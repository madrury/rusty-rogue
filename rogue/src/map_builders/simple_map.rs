use rltk::{RandomNumberGenerator};
use specs::prelude::*;

use super::MapBuilder;
use super::{
    Map, TileType, Point, Rectangle, MAP_WIDTH, MAP_HEIGHT,
    DEBUG_VISUALIZE_MAPGEN, entity_spawners, terrain_spawners, apply_room,
    apply_horizontal_tunnel, apply_vertical_tunnel, get_stairs_position
};

//----------------------------------------------------------------------------
// Simple Map
//
// This is the simplest map genration algorithm, A map with some rectangular
// rooms connected with L shaped corridors.
//----------------------------------------------------------------------------
pub struct SimpleMapBuilder {
    map: Map,
    depth: i32,
    rooms: Vec<Rectangle>,
    history: Vec<Map>
}

impl MapBuilder for SimpleMapBuilder {

    fn map(&self) -> Map {
        self.map.clone()
    }


    fn build_map(&mut self) {
        self.rooms_and_corridors();
        self.map.intitialize_blocked();
        self.map.intitialize_opaque();
        self.map.intitialize_ok_to_spawn();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        entity_spawners::spawn_monsters(ecs, self.depth);
        for room in self.rooms.iter().skip(1) {
            let mut region: Vec<usize> = Vec::new();
            for y in room.y1 + 1 .. room.y2 {
                for x in room.x1 + 1 .. room.x2 {
                    let idx = self.map.xy_idx(x, y);
                    if self.map.tiles[idx] == TileType::Floor {
                        region.push(idx);
                    }
                }
            }
            entity_spawners::spawn_items_in_region(ecs, &region, self.depth);
        }
    }

    fn spawn_terrain(&mut self, ecs: &mut World) {
        terrain_spawners::spawn_terrain(ecs, self.depth);
    }

    fn starting_position(&self, ecs: &World) -> Point {
        let start = entity_spawners::player::get_spawn_position(ecs);
        match start {
            Some(start) => start,
            None => panic!("Failed to find a starting position.")
        }
    }

    fn stairs_position(&self, ecs: &World) -> Point {
        let start = get_stairs_position(ecs);
        match start {
            Some(start) => start,
            None => panic!("Failed to find a stairs position.")
        }
    }

    fn take_snapshot(&mut self) {
        if DEBUG_VISUALIZE_MAPGEN {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                // So the snapshot will render everything.
                *v = true;
            }
            self.history.push(snapshot);
        }
    }

    fn snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

}

impl SimpleMapBuilder {

    pub fn new(depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder{
            map: Map::new(depth),
            depth: depth,
            rooms: Vec::new(),
            history: Vec::new()
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
            let ok_to_place = self.rooms
                .iter()
                .all(|other| !new_room.intersect(other));
            if ok_to_place {
                apply_room(&mut self.map, &new_room);
                self.take_snapshot();
                if !self.rooms.is_empty() {
                    let (cxnew, cynew) = new_room.center();
                    let (cxprev, cyprev) =self.rooms[self.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(&mut self.map, cxprev, cxnew, cyprev);
                        apply_vertical_tunnel(&mut self.map, cyprev, cynew, cxnew);
                    } else {
                        apply_vertical_tunnel(&mut self.map, cyprev, cynew, cxprev);
                        apply_horizontal_tunnel(&mut self.map, cxprev, cxnew, cynew);
                    }
                }
                self.rooms.push(new_room);
                self.take_snapshot();
            }
        }
    }
}