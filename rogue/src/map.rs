use rltk::{Algorithm2D, BaseMap, Bresenham, Point, RGB, RandomNumberGenerator, Rltk};
use serde::{Serialize, Deserialize};
use specs::prelude::*;
use std::iter::Iterator;
use take_until::*;
use super::{DEBUG_DRAW_ALL_MAP, MovementRoutingOptions};


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
    // The current depth into the dungeon. This will effect which types of map
    // geometries can spawn, along with enemies and items.
    pub depth: i32,
    pub tiles: Vec<TileType>,
    // Has the player seen this tile? If so, we render it even when it is out of
    // visibility, but darkened.
    pub revealed_tiles: Vec<bool>,
    // Is this tile currently visible to the player. This array is updated in
    // visibility_system every turn.
    pub visible_tiles: Vec<bool>,
    // Is the tile currently blocked for movement? This array needs to be kept
    // synced with game state.
    pub blocked: Vec<bool>,
    // Is the tile currently opaque, i.e., does it block visibility?
    pub opaque: Vec<bool>,
    // Is the tile currently occuped by fire? We only want to spawn one fire
    // entity in each tile, so we need a source of truth for this.
    pub fire: Vec<bool>,
    // Is the tile currently occuped by chill? We only want to spawn one chill
    // entity in each tile, so we need a source of truth for this.
    pub chill: Vec<bool>,
    // Is the tile currently occupied by steam?
    pub steam: Vec<bool>,
    // Is the tile currently occuped by water?
    pub water: Vec<bool>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    // Convenience vector of all the entities occupying a tile. This array is
    // updated each turn in map_indexing_system.rs.
    pub tile_content: Vec<Vec<Entity>>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    // Helper vector used ONLY when generating a new map (so, this has no
    // gameplay application).
    pub ok_to_spawn: Vec<bool>
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
            opaque : vec![false; MAP_SIZE],
            fire: vec![false; MAP_SIZE],
            chill: vec![false; MAP_SIZE],
            steam: vec![false; MAP_SIZE],
            water: vec![false; MAP_SIZE],
            tile_content : vec![Vec::new(); MAP_SIZE],
            ok_to_spawn: vec![true; MAP_SIZE],
        }
    }

    pub fn intitialize_blocked(&mut self) {
        for (i, tile) in self.tiles.iter().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn intitialize_opaque(&mut self) {
        for (i, tile) in self.tiles.iter().enumerate() {
            self.opaque[i] = *tile == TileType::Wall;
        }
    }

    pub fn intitialize_ok_to_spawn(&mut self) {
        for (i, tile) in self.tiles.iter().enumerate() {
            self.ok_to_spawn[i] = *tile != TileType::Wall && *tile != TileType::DownStairs;
        }
    }

    pub fn clear_tile_content(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if !self.within_bounds(x, y) {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        ((y * self.width) as usize) + x as usize
    }

    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
    }

    pub fn within_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x <= self.width - 1 && y >= 0 && y <= self.height - 1
    }

    pub fn is_edge_tile(&self, x: i32, y: i32) -> bool {
        x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1
    }

    pub fn is_reachable(&self, start: Point, end: Point) -> bool {
        let start_idx = self.xy_idx(start.x, start.y);
        let end_idx = self.xy_idx(end.x, end.y);
        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.width, self.height, &map_starts , self, 200.0
        );
        dijkstra_map.map[end_idx] != std::f32::MAX
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

    pub fn get_adjacent_tiles(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        let ds = vec![
            (-1, -1),
            (-1,  0),
            (-1,  1),
            (0,  -1),
            (0,   1),
            (1,  -1),
            (1,   0),
            (1,   1),
        ];
        ds.into_iter()
            .map(|(dx, dy)| (x + dx, y + dy))
            .filter(|(x, y)| self.within_bounds(*x, *y))
            .collect()
    }

    pub fn random_adjacent_point(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        // TODO: This should use the game's internal RNG.
        let mut rng = RandomNumberGenerator::new();
        let adjacent_tiles: Vec<(i32, i32)> = self.get_adjacent_tiles(x, y)
            .into_iter()
            .map(|(x, y)| self.xy_idx(x, y))
            .filter(|idx| self.tiles[*idx] != TileType::Wall)
            .map(|idx| self.idx_xy(idx))
            .collect();
        let n_adjacent_tiles = adjacent_tiles.len() as i32;
        match n_adjacent_tiles {
            0 => None,
            _ => {
                let ridx = (rng.roll_dice(1, n_adjacent_tiles) - 1) as usize;
                Some(adjacent_tiles[ridx])
            }
        }
    }

    pub fn random_adjacent_unblocked_point(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        // TODO: This should use the game's internal RNG.
        let mut rng = RandomNumberGenerator::new();
        let adjacent_unblocked_tiles: Vec<(i32, i32)> = self.get_adjacent_tiles(x, y)
            .into_iter()
            .map(|(x, y)| self.xy_idx(x, y))
            .filter(|idx| !self.blocked[*idx])
            .map(|idx| self.idx_xy(idx))
            .collect();
        let n_adjacent_unblocked_tiles = adjacent_unblocked_tiles.len() as i32;
        match n_adjacent_unblocked_tiles {
            0 => None,
            _ => {
                let ridx = (rng.roll_dice(1, n_adjacent_unblocked_tiles) - 1) as usize;
                Some(adjacent_unblocked_tiles[ridx])
            }
        }
    }

    pub fn get_aoe_tiles(&self, pt: Point, radius: f32) -> Vec<Point> {
        let in_viewshed = rltk::field_of_view(pt, f32::ceil(radius) as i32, self);
        in_viewshed.into_iter()
            .filter(|p| rltk::DistanceAlg::Pythagoras.distance2d(*p, pt) < radius)
            .collect()
    }

    pub fn get_ray_tiles(&self, source: Point, target: Point, take_until_blocked: bool) -> Vec<Point> {
        let mut tiles: Vec<Point> = Bresenham::new(source, target).collect();
        tiles.push(target);
        tiles.into_iter()
            .skip(1)
            .take_until(|p| {
                let idx = self.xy_idx(p.x, p.y);
                take_until_blocked && self.blocked[idx]
            })
            .collect()
    }

    pub fn get_l_infinity_circle_around(&self, source: Point, radius: i32) -> Vec<Point> {
        let diameter = 2*radius;
        let mut circle: Vec<Point> = vec![
            Point {x: source.x - radius, y: source.y - radius},
            Point {x: source.x + radius, y: source.y - radius},
            Point {x: source.x - radius, y: source.y + radius},
            Point {x: source.x + radius, y: source.y + radius},
        ];
        for i in 1..diameter {
            circle.push(Point {x: source.x - radius + i, y: source.y - radius});
            circle.push(Point {x: source.x + radius, y: source.y - radius + i});
            circle.push(Point {x: source.x + radius - i, y: source.y + radius});
            circle.push(Point {x: source.x - radius, y: source.y + radius - i});
        }
        circle.into_iter().filter(|pt| self.within_bounds(pt.x, pt.y)).collect()
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.opaque[idx]
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

//----------------------------------------------------------------------------
// RoutingMap.
// Supports options for how to route Monsters around hazardous terrain.
//
// This struct is meant to be used with the rltk pathfinding functions
// a_star_search and dijkstra_map. It can be created from a Map using the
// from_map function, which consumes a struct of RoutingOptions defining how a
// monster treats movement through hazardous terrain.
//----------------------------------------------------------------------------
pub struct RoutingMap {
    pub width: i32,
    pub height: i32,
    pub avoid: Vec<bool>
}

impl RoutingMap {
    pub fn from_map(map: &Map, options: &MovementRoutingOptions) -> RoutingMap {
        let mut route = RoutingMap {
            width: map.width,
            height: map.height,
            avoid: vec![false; map.width as usize * map.height as usize]
        };
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                route.avoid[idx] =
                    map.tiles[idx] == TileType::Wall
                    || (options.avoid_blocked && map.blocked[idx])
                    || (options.avoid_fire && map.fire[idx])
                    || (options.avoid_chill && map.chill[idx])
                    || (options.avoid_water && map.water[idx]);
            }
        }
        route
    }

    fn xy_idx(&self, x: i32, y: i32) -> usize {
        ((y * self.width) as usize) + x as usize
    }

    fn within_bounds(&self, x: i32, y: i32) -> bool {
        x >= 1 && x <= self.width - 1 && y >= 1 && y <= self.height - 1
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if !self.within_bounds(x, y) {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.avoid[idx]
    }

    fn is_reachable(&self, start: Point, end: Point) -> bool {
        let start_idx = self.xy_idx(start.x, start.y);
        let end_idx = self.xy_idx(end.x, end.y);
        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.width, self.height, &map_starts , self, 200.0
        );
        dijkstra_map.map[end_idx] != std::f32::MAX
    }
}

impl BaseMap for RoutingMap {
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

}