use rltk::{ RGB, Rltk, RandomNumberGenerator };
use std::cmp::{min, max};
use std::iter::Iterator;
use super::{Rectangle};


const XWIDTH: i32 = 80;
const YWIDTH: i32 = 50;
const MAX_IDX: usize = (XWIDTH as usize) * (YWIDTH as usize);


#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rectangle>,
    pub width : i32,
    pub height : i32
}

impl Map {

    pub fn new_rooms_and_corridors() -> Map {

        let mut map = Map{
           tiles: vec![TileType::Wall; (XWIDTH * YWIDTH) as usize],
           rooms: Vec::new(),
           width: XWIDTH,
           height: YWIDTH
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_ROOM_SIZE: i32 = 6;
        const MAX_ROOM_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let h = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let x = rng.roll_dice(1, XWIDTH - w - 1) - 1;
            let y = rng.roll_dice(1, YWIDTH - h - 1) - 1;
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
        map
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        ((y * self.width) as usize) + x as usize
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
            if idx > 0 && idx < MAX_IDX {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAX_IDX {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

}


pub fn draw_map(map: &Map, ctx: &mut Rltk) {
    let mut x = 0;
    let mut y = 0;
    for tile in map.tiles.iter() {
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
            }
        }
        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}