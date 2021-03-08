use rltk::{RGB, Rltk, RandomNumberGenerator};
use specs::prelude::*;

use super::MapBuilder;
use super::{
    Map, Rectangle, TileType, Position, MAP_WIDTH, MAP_HEIGHT, spawner,
    apply_room, apply_horizontal_tunnel, apply_vertical_tunnel
};


pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {

    fn build(depth: i32) -> (Map, Position) {
       let mut map =  Map::new(depth);
       let ppos = SimpleMapBuilder::rooms_and_corridors(&mut map);
       (map, ppos)
    }

    fn spawn(map: &Map, ecs: &mut World, depth: i32) {
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, depth);
        }
    }
}

impl SimpleMapBuilder {

    fn rooms_and_corridors(map: &mut Map) -> Position {
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
            let ok_to_place = map.rooms.iter().all(|other| !new_room.intersect(other));
            if ok_to_place {
                apply_room(map, &new_room);
                if !map.rooms.is_empty() {
                    let (cxnew, cynew) = new_room.center();
                    let (cxprev, cyprev) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(map, cxprev, cxnew, cyprev);
                        apply_vertical_tunnel(map, cyprev, cynew, cxnew);
                    } else {
                        apply_vertical_tunnel(map, cyprev, cynew, cxprev);
                        apply_horizontal_tunnel(map, cxprev, cxnew, cynew);
                    }
                }
                map.rooms.push(new_room)
            }
        }
        // Place downward stairs.
        let stairs_position = map.rooms[map.rooms.len()-1].center();
        let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStairs;
        // Compute the player's starting position in this map.
        let start_position = map.rooms[0].center();
        Position {x: start_position.0, y: start_position.1}
    }
}