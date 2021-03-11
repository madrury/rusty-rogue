use rltk::{RandomNumberGenerator};
use specs::prelude::*;

use super::MapBuilder;
use super::{
    Map, TileType, Position, DEBUG_VISUALIZE_MAPGEN, spawner
};

//const MIN_ROOM_SIZE : i32 = 8;
const N_ITERATIONS: i32 = 4;

//----------------------------------------------------------------------------
// Celular Automata Map
//
// This map generator creates organic looking caves using a cellular automata
// algorithm.
//----------------------------------------------------------------------------
pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>
}

impl MapBuilder for CellularAutomataBuilder {

    fn map(&self) -> Map {
        self.map.clone()
    }

    fn starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn take_snapshot(&mut self) {
        if DEBUG_VISUALIZE_MAPGEN {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }

    fn snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        self.ititialize(&mut rng);
        self.take_snapshot();
        for _i in 0..N_ITERATIONS {
            self.update();
            self.take_snapshot();
        }
        // We use a Dijkstra map to figure out the furthest point from the
        // player's spawn position. To do so, we need the blocked array
        // populated correctly, as we rely on this for the is_exit_valid method
        // on Map.
        self.map.populate_blocked();
        self.compute_starting_position();
        self.place_stairs();
        self.take_snapshot();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
    }

}

impl CellularAutomataBuilder {

    pub fn new(depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder{
            map: Map::new(depth),
            starting_position: Position{x: 0, y: 0},
            depth: depth,
            history: Vec::new()
        }
    }

    // Populate the map with independent random Wall tiles.
    fn ititialize(&mut self, rng: &mut RandomNumberGenerator) {
        for x in 1..self.map.width - 1 {
            for y in 1..self.map.height - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);
                if roll > 48 {
                    self.map.tiles[idx] = TileType::Floor
                } else {
                    self.map.tiles[idx] = TileType::Wall
                }
            }
        }
    }

    // Run one pass of the following cellular automata update algorithm:
    //   - If a tile is a wall, and is surrounded by <= 3 adjacent walls, then it
    //     becomes a floor.
    //   - If a tile is a floor, and is surrounded by >= adjacent walls, then it
    //     becomes a wall.
    fn update(&mut self) {
        for _i in 0..15 {
            let mut newtiles = self.map.tiles.clone();
            for y in 1..self.map.height-1 {
                for x in 1..self.map.width-1 {
                    let idx = self.map.xy_idx(x, y);
                    let center = self.map.tiles[idx] == TileType::Wall;
                    let neighbours = [
                        self.map.tiles[idx - 1] == TileType::Wall,
                        self.map.tiles[idx + 1] == TileType::Wall,
                        self.map.tiles[idx - self.map.width as usize] == TileType::Wall,
                        self.map.tiles[idx + self.map.width as usize] == TileType::Wall,
                        self.map.tiles[idx - (self.map.width as usize - 1)] == TileType::Wall,
                        self.map.tiles[idx - (self.map.width as usize + 1)] == TileType::Wall,
                        self.map.tiles[idx + (self.map.width as usize - 1)] == TileType::Wall,
                        self.map.tiles[idx + (self.map.width as usize + 1)] == TileType::Wall
                    ];
                    let n_neighbors: i32 = neighbours.iter().map(|x| if *x {1} else {0}).sum();

                    if center && n_neighbors <= 3 {
                        newtiles[idx] = TileType::Floor;
                    }
                    else if !center && n_neighbors >= 5 {
                        newtiles[idx] = TileType::Wall;
                    }
                }
            }
            self.map.tiles = newtiles.clone();
        }
    }

    // Get a starting position by sampling a single unblocked point from the
    // map.
    fn compute_starting_position(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        let pt = self.map.random_unblocked_point(250, &mut rng);
        // TODO: If we get None, we'll spawn at (0, 0), which will softlock
        // the game. We probably want to deal with this at some point.
        if let Some(pt) = pt {
            self.starting_position = Position {x: pt.0, y: pt.1};
        }
    }

    // Place the stairs as far away from the player as possible.
    fn place_stairs(&mut self) {
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.map.width, self.map.height, &map_starts , &self.map, 200.0
        );
        let mut exit_tile = (0, 0.0f32);
        for (idx, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[idx];
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                } else {
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = idx;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }
        self.map.tiles[exit_tile.0] = TileType::DownStairs;
    }
}