use rltk::{ RGB, Rltk };

const XWIDTH: i32 = 80;
const YWIDTH: i32 = 50;
// const PLAYER_START: (i32, i32) = (40, 25);
const PLAYER_START_IDX: usize = (40 as usize) * 80 + (25 as usize);


#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

pub fn new_map() -> Vec<TileType> {
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
