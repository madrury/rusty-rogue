use rltk::{RGB, Rltk, RandomNumberGenerator};
use specs::prelude::*;

use super::MapBuilder;
use super::{
    Map, Rectangle, TileType, Position, MAP_WIDTH, MAP_HEIGHT, spawner,
    apply_room, apply_horizontal_tunnel, apply_vertical_tunnel
};


// A map with some rectangular rooms connected with L shaped corridors.
pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32
}

impl MapBuilder for SimpleMapBuilder {

    fn map(&self) -> Map {
        self.map.clone()
    }

    fn starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for room in self.map.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
    }

}

impl SimpleMapBuilder {

    pub fn new(depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder{
            map: Map::new(depth),
            starting_position: Position{x: 0, y: 0},
            depth: depth
        }
    }

    fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS: i32 = 30;
        const MIN_ROOM_SIZE: i32 = 6;
        const MAX_ROOM_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let h = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let x = rng.roll_dice(1, MAP_WIDTH - w - 1) - 1;
            let y = rng.roll_dice(1, MAP_HEIGHT - h - 1) - 1;
            let new_room = Rectangle::new(x, y, w, h);
            // Try to place our new room on the map.
            let ok_to_place = self.map.rooms
                .iter()
                .all(|other| !new_room.intersect(other));
            if ok_to_place {
                apply_room(&mut self.map, &new_room);
                if !self.map.rooms.is_empty() {
                    let (cxnew, cynew) = new_room.center();
                    let (cxprev, cyprev) =self.map.rooms[self.map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(&mut self.map, cxprev, cxnew, cyprev);
                        apply_vertical_tunnel(&mut self.map, cyprev, cynew, cxnew);
                    } else {
                        apply_vertical_tunnel(&mut self.map, cyprev, cynew, cxprev);
                        apply_horizontal_tunnel(&mut self.map, cxprev, cxnew, cynew);
                    }
                }
                self.map.rooms.push(new_room)
            }
        }
        // Place downward stairs.
        let stairs_position = self.map.rooms[self.map.rooms.len()-1].center();
        let stairs_idx = self.map.xy_idx(stairs_position.0, stairs_position.1);
        self.map.tiles[stairs_idx] = TileType::DownStairs;
        // Compute the player's starting position in this map.
        let start_position = self.map.rooms[0].center();
        self.starting_position = Position{x: start_position.0, y: start_position.1}
    }
}