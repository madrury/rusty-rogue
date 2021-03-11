use std::cmp::{max, min};
use serde::{Serialize, Deserialize};
use rltk::{RandomNumberGenerator};
use super::{Map, TileType, MAP_SIZE};


pub fn apply_room(map: &mut Map, room: &Rectangle) {
    for x in (room.x1 + 1)..=room.x2 {
        for y in (room.y1 + 1)..=room.y2 {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < MAP_SIZE {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < MAP_SIZE {
            map.tiles[idx] = TileType::Floor;
        }
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
    pub fn random_point(&self) -> (i32, i32) {
        let mut rng = RandomNumberGenerator::new();
        let room_width = i32::abs(self.x1 - self.x2);
        let room_height = i32::abs(self.y1 - self.y2);
        let x = self.x1 + rng.roll_dice(1, room_width);
        let y = self.y1 + rng.roll_dice(1, room_height);
        (x, y)
    }
}