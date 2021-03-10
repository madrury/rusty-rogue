use rltk::{RandomNumberGenerator};
use specs::prelude::*;

use super::MapBuilder;
use super::{
    Map, Rectangle, TileType, Position,
    DEBUG_VISUALIZE_MAPGEN, spawner,
};

//const MIN_ROOM_SIZE : i32 = 8;
const N_ITERATIONS: i32 = 14;

// A map with some rectangular rooms connected with L shaped corridors.
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
                // So the snapshot will render everything.
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
            self.update(&mut rng);
            self.take_snapshot();
        }
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

    fn update(&mut self, rng: &mut RandomNumberGenerator) {
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
}