use rltk::{RGB, Rltk, RandomNumberGenerator, Algorithm2D, BaseMap, Point};
use serde::{Serialize, Deserialize};
use specs::prelude::*;
use std::cmp::{min, max};
use std::iter::Iterator;
use super::{DEBUG_DRAW_ALL_MAP};


pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;
pub const MAP_SIZE: usize = (MAP_WIDTH as usize) * (MAP_HEIGHT as usize);


#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    BloodStain,
    DownStairs
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {

    pub fn new(depth : i32) -> Map {
        Map{
            width : MAP_WIDTH as i32,
            height: MAP_HEIGHT as i32,
            depth: depth,
            tiles : vec![TileType::Wall; MAP_SIZE],
            revealed_tiles : vec![false; MAP_SIZE],
            visible_tiles : vec![false; MAP_SIZE],
            blocked : vec![false; MAP_SIZE],
            tile_content : vec![Vec::new(); MAP_SIZE],
        }
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        ((y * self.width) as usize) + x as usize
    }

    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
    }

    pub fn random_point(&self, n_tries: i32, rng: &mut RandomNumberGenerator) -> Option<(i32, i32)> {
        for _ in 0..n_tries {
            let x = rng.roll_dice(1, self.width) - 1;
            let y = rng.roll_dice(1, self.height) - 1;
            let idx = self.xy_idx(x, y);
            let tile = self.tiles[idx];
            if tile == TileType::Floor || tile == TileType::BloodStain {
                return Some((x, y))
            }
        }
        return None
    }

    pub fn random_unblocked_point(&self, n_tries: i32, rng: &mut RandomNumberGenerator) -> Option<(i32, i32)> {
        for _ in 0..n_tries {
            let pt = self.random_point(n_tries, rng);
            if let Some(pt) = pt {
                let idx = self.xy_idx(pt.0, pt.1);
                if !self.blocked[idx] {
                    return Some(pt);
                }
            }
        }
        return None;
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

pub fn draw_map(map: &Map, ctx: &mut Rltk) {
    for (idx, tile) in map.tiles.iter().enumerate() {
        let pt = Point::new(idx as i32 % MAP_WIDTH, idx as i32 / MAP_WIDTH);
        if map.revealed_tiles[idx] || DEBUG_DRAW_ALL_MAP {
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
            fg = RGB::from_f32(0.66, 0.66, 0.25);
        }
        TileType::BloodStain => {
            glyph = rltk::to_cp437('.');
            fg = RGB::from_f32(0.5, 0.0, 0.0);
        }
        TileType::Wall => {
            glyph = rltk::to_cp437('#');
            fg = RGB::from_f32(1.00, 1.00, 0.50);
        }
        TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(1.0, 1.0, 0.0);
        }
    }
    if !visible {fg = RGB::from_f32(0.5, 0.5, 0.5);}
    ctx.set(x, y, fg, RGB::from_f32(0.0, 0.0, 0.0), glyph);
}
