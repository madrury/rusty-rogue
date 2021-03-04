use rltk::{RGB, Rltk, RandomNumberGenerator, Algorithm2D, BaseMap, Point};
use serde::{Serialize, Deserialize};
use specs::prelude::*;
use std::cmp::{min, max};
use std::iter::Iterator;
// use super::{Rectangle};


pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;
pub const MAP_SIZE: usize = (MAP_WIDTH as usize) * (MAP_HEIGHT as usize);
const DEBUG_DRAW_ALL: bool = false;


#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rectangle>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {

    pub fn new_rooms_and_corridors(depth: i32) -> Map {

        let mut map = Map{
           width: MAP_WIDTH,
           height: MAP_HEIGHT,
           depth: depth,
           tiles: vec![TileType::Wall; MAP_SIZE],
           rooms: Vec::new(),
           revealed_tiles: vec![false; MAP_SIZE],
           visible_tiles: vec![false; MAP_SIZE],
           blocked: vec![false; MAP_SIZE],
           tile_content: vec![Vec::new(); MAP_SIZE],
        };

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
                map.apply_room(&new_room);
                if !map.rooms.is_empty() {
                    let (cxnew, cynew) = new_room.center();
                    let (cxprev, cyprev) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(cxprev, cxnew, cyprev);
                        map.apply_vertical_tunnel(cyprev, cynew, cxnew);
                    } else {
                        map.apply_vertical_tunnel(cyprev, cynew, cxprev);
                        map.apply_horizontal_tunnel(cxprev, cxnew, cynew);
                    }
                }
                map.rooms.push(new_room)
            }
        }
        // Place downward stairs.
        let stairs_position = map.rooms[map.rooms.len()-1].center();
        let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStairs;
        map
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        ((y * self.width) as usize) + x as usize
    }

    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_tile_content(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    fn apply_room(&mut self, room: &Rectangle) {
        for x in (room.x1 + 1)..=room.x2 {
            for y in (room.y1 + 1)..=room.y2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAP_SIZE {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAP_SIZE {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;
        if self.is_exit_valid(x - 1, y) { exits.push((idx - 1, 1.0));}
        if self.is_exit_valid(x + 1, y) { exits.push((idx + 1, 1.0));}
        if self.is_exit_valid(x, y - 1) { exits.push((idx - w, 1.0));}
        if self.is_exit_valid(x, y + 1) { exits.push((idx + w, 1.0));}
        // 1.45 is approximately sqrt(2).
        if self.is_exit_valid(x - 1, y - 1) { exits.push((idx - w - 1, 1.45));}
        if self.is_exit_valid(x + 1, y - 1) { exits.push((idx - w + 1, 1.45));}
        if self.is_exit_valid(x - 1, y + 1) { exits.push((idx + w - 1, 1.45));}
        if self.is_exit_valid(x + 1, y + 1) { exits.push((idx + w + 1, 1.45));}
        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
   fn dimensions(&self) -> Point {
       Point::new(self.width, self.height)
   }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    for (idx, tile) in map.tiles.iter().enumerate() {
        let pt = Point::new(idx as i32 % MAP_WIDTH, idx as i32 / MAP_WIDTH);
        if map.revealed_tiles[idx] || DEBUG_DRAW_ALL {
            let visible = map.visible_tiles[idx];
            draw_tile(pt.x, pt.y, tile, visible, ctx);
        }
    }
}

fn draw_tile(x: i32, y: i32, tile: &TileType, visible: bool, ctx: &mut Rltk) {
    let glyph;
    let mut fg;
    match tile {
        TileType::Floor => {
            glyph = rltk::to_cp437('.');
            fg = RGB::from_f32(0.0, 0.5, 0.5);
        }
        TileType::Wall => {
            glyph = rltk::to_cp437('#');
            fg = RGB::from_f32(0.0, 1.0, 0.5);
        }
        TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(1.0, 1.0, 0.0);
        }
    }
    if !visible {fg = fg.to_greyscale();}
    ctx.set(x, y, fg, RGB::from_f32(0.0, 0.0, 0.0), glyph);
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
    pub fn random_point(&self, ecs: &mut World) -> (i32, i32) {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let room_width = i32::abs(self.x1 - self.x2);
        let room_height = i32::abs(self.y1 - self.y2);
        let x = self.x1 + rng.roll_dice(1, room_width);
        let y = self.y1 + rng.roll_dice(1, room_height);
        (x, y)
    }
}