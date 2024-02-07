use std::cmp::{max, min};
use specs::prelude::*;
use serde::{Serialize, Deserialize};
use rltk::RandomNumberGenerator;
use crate::terrain_spawners::water::WaterSpawnTable;

use super::{Map, Point, TileType, MAP_SIZE};

const PLACE_STAIRS_N_TRYS: i32 = 250;

pub fn get_stairs_position(ecs: &World) -> Option<Point> {
    for _ in 0..PLACE_STAIRS_N_TRYS {
        if let Some(pt) = try_get_stairs_position(ecs) {
            return Some(pt)
        }
    }
    return None
}
// Search for a position to place stairs in the map. Our strategy is to either
// do horizontal or vertical scans of the map in a single row or column of
// tiles. A valid place will consist of a sequence of open-tile, wall-tile,
// wall-tile. Once one is found, we spawn the stairs in the middle wall tile.
fn try_get_stairs_position(ecs: &World) -> Option<Point> {
    let mut rng = RandomNumberGenerator::new();
    let map = ecs.fetch::<Map>();
    let mut start: bool;
    let mut next: bool;
    let mut last: bool;
    let scan_type = rng.roll_dice(1, 4);
    match scan_type {
        // Scan in a row, right to left.
        1 => {
            let y: i32 = rng.roll_dice(1, map.height) - 1;
            for x in 2..map.width {
                start = map.blocked[map.xy_idx(x, y)];
                next = map.blocked[map.xy_idx(x - 1, y)];
                last = map.blocked[map.xy_idx(x - 2, y)];
                if (start, next, last) == (false, true, true) {
                    return Some(Point {x: x - 1, y: y})
                }
            }
        },
        // Scan in a row, left to right.
        2 => {
            let y: i32 = rng.roll_dice(1, map.height) - 1;
            for x in (2..map.width).rev() {
                start = map.blocked[map.xy_idx(x, y)];
                next = map.blocked[map.xy_idx(x - 1, y)];
                last = map.blocked[map.xy_idx(x - 2, y)];
                if (start, next, last) == (false, true, true) {
                    return Some(Point {x: x - 1, y: y})
                }
            }
        },
        // Scan in a column, top to bottom.
        3 => {
            let x: i32 = rng.roll_dice(1, map.width) - 1;
            for y in 2..map.height {
                start = map.blocked[map.xy_idx(x, y)];
                next = map.blocked[map.xy_idx(x, y - 1)];
                last = map.blocked[map.xy_idx(x, y - 2)];
                if (start, next, last) == (false, true, true) {
                    return Some(Point {x: x, y: y - 1})
                }
            }
        },
        // Scan in a row, left to right.
        4 => {
            let x: i32 = rng.roll_dice(1, map.width) - 1;
            for y in (2..map.height).rev() {
                start = map.blocked[map.xy_idx(x, y)];
                next = map.blocked[map.xy_idx(x, y - 1)];
                last = map.blocked[map.xy_idx(x, y - 2)];
                if (start, next, last) == (false, true, true) {
                    return Some(Point {x: x, y: y - 1})
                }
            }
        },
        _ => {}
    }
    return None
}

// Carving functions
// "Carving" refers to replacing TileType::Wall tiles in a map with
// Tiletype::Floor tiles during generation.
pub fn carve_out_rectangular_room(map: &mut Map, room: &Rectangle) {
    for x in (room.x1 + 1)..=room.x2 {
        for y in (room.y1 + 1)..=room.y2 {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn carve_out_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < MAP_SIZE {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn carve_out_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < MAP_SIZE {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn carve_out_water_spawn_table(map: &mut Map, water_spawn_table: &WaterSpawnTable) {
    for elem in &water_spawn_table.shallow {
        let idx = map.xy_idx(elem.x, elem.y);
        map.tiles[idx] = TileType::Floor
    }
    for elem in &water_spawn_table.deep {
        let idx = map.xy_idx(elem.x, elem.y);
        map.tiles[idx] = TileType::Floor
    }
}

#[derive(PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32
}

impl Rectangle {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rectangle {
        Rectangle{x1: x, x2: x + w, y1: y, y2: y + h}
    }
    pub fn intersect(&self, other: &Rectangle) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }
    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2)/2, (self.y1 + self.y2)/2)
    }
}