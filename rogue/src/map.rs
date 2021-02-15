use rltk::{ RGB, Rltk, RandomNumberGenerator };
use std::cmp::{min, max};
use std::iter::Iterator;
use super::{Rectangle};


const XWIDTH: i32 = 80;
const YWIDTH: i32 = 50;
// const PLAYER_START: (i32, i32) = (40, 25);
const MAX_IDX: usize = (XWIDTH as usize) * (YWIDTH as usize);
const PLAYER_START_IDX: usize = (40 as usize) * 80 + (25 as usize);


#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn new_map_rooms_and_corridors() -> (Vec<Rectangle>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; (XWIDTH * YWIDTH) as usize];
    let mut rooms: Vec<Rectangle> = Vec::new();

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
        let ok_to_place = rooms.iter().all(|other| !new_room.intersect(other));
        if ok_to_place {
            apply_room_to_map(&new_room, &mut map);

            if !rooms.is_empty() {
                let (cxnew, cynew) = new_room.center();
                let (cxprev, cyprev) = rooms[rooms.len() - 1].center();
                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut map, cxprev, cxnew, cyprev);
                    apply_vertical_tunnel(&mut map, cyprev, cynew, cxnew);
                } else {
                    apply_vertical_tunnel(&mut map, cyprev, cynew, cxprev);
                    apply_horizontal_tunnel(&mut map, cxprev, cxnew, cynew);
                }
            }

            rooms.push(new_room)
        }
    }

    (rooms, map)
}

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut x = 0;
    let mut y = 0;
    for tile in map.iter() {
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

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

fn apply_room_to_map(room: &Rectangle, map: &mut[TileType]) {
    for x in (room.x1 + 1)..=room.x2 {
        for y in (room.y1 + 1)..=room.y2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut[TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < MAX_IDX {
            map[idx] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut[TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < MAX_IDX {
            map[idx] = TileType::Floor;
        }
    }
}

/// Deprecated.
pub fn new_map_random_placement() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; (XWIDTH * YWIDTH) as usize];
    // Make boundary walls
    for x in 0..XWIDTH {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, YWIDTH - 1)] = TileType::Wall;
    }
    for y in 0..YWIDTH {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(XWIDTH - 1, y)] = TileType::Wall;
    }
    // Randomly splat a bunch of walls.
    let mut rng = rltk::RandomNumberGenerator::new();
    for _i in 0..400 {
        let x = rng.roll_dice(1, XWIDTH - 1);
        let y = rng.roll_dice(1, YWIDTH - 1);
        let idx = xy_idx(x, y);
        if idx != PLAYER_START_IDX {
            map[idx] = TileType::Wall;
        }
    }
    map
}
