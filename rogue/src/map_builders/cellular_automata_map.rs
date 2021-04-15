use std::collections::HashMap;
use rltk::{RandomNumberGenerator};
use specs::prelude::*;

use super::MapBuilder;
use super::{
    Map, Point, TileType, entity_spawners, terrain_spawners, get_stairs_position,
    DEBUG_VISUALIZE_MAPGEN
};

const N_ITERATIONS: i32 = 4;

//----------------------------------------------------------------------------
// Celular Automata Map
//
// This map generator creates organic looking caves using a cellular automata
// algorithm.
//----------------------------------------------------------------------------
pub struct CellularAutomataBuilder {
    map: Map,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>
}

impl MapBuilder for CellularAutomataBuilder {

    fn map(&self) -> Map {
        self.map.clone()
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
        // The use of the map.blocked array here is confusing. We need to call initialize_blocked *twice*.
        //   - The *first* call to initialize_blocked is to support out use of the
        //     rltk::Dijkstra map when finding the connected components of our map.
        //     This uses the blocked array to determine the open neighbours of a
        //     tile.
        //  - The *second* call to initialize_blocked is after we have filled in
        //    the smaller connected components with TileType::Wall. We need to
        //    re-compute the blocked array so that it can be used when spawning
        //    entities.
        self.map.intitialize_blocked();
        let components = self.enumerate_connected_components();
        self.fill_all_but_largest_component(components);
        self.map.intitialize_blocked();
        self.populate_noise_areas();
        self.map.intitialize_opaque();
        self.map.intitialize_ok_to_spawn();
    }

    fn spawn_terrain(&mut self, ecs: &mut World) {
        terrain_spawners::spawn_terrain(ecs, self.depth);
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            entity_spawners::spawn_region(ecs, area.1, self.depth);
        }
    }

    fn starting_position(&self, ecs: &World) -> Point {
        let start = entity_spawners::player::get_spawn_position(ecs);
        match start {
            Some(start) => start,
            None => panic!("Failed to find a starting position.")
        }
    }

    fn stairs_position(&self, ecs: &World) -> Point {
        let start = get_stairs_position(ecs);
        match start {
            Some(start) => start,
            None => panic!("Failed to find a stairs position.")
        }
    }

}

impl CellularAutomataBuilder {

    pub fn new(depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder{
            map: Map::new(depth),
            depth: depth,
            history: Vec::new(),
            noise_areas: HashMap::new()
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

    fn enumerate_connected_components(&self) -> HashMap<usize, Vec<usize>> {
        let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut floor: Vec<bool> = self.map.tiles
            .iter()
            .map(|tt| *tt == TileType::Floor)
            .collect();
        loop {
            let nextidx = self.get_next_idx(&floor);
            match nextidx {
                None => {break;}
                Some(idx) => {
                    let component = self.get_connected_component(idx);
                    for cidx in component.iter() {
                        floor[*cidx] = false;
                    }
                    components.insert(idx, component);
                }
            }
        }
        components
    }

    fn get_next_idx(&self, floor: &Vec<bool>) -> Option<usize> {
        for idx in 0..(self.map.width * self.map.height) {
            if floor[idx as usize] {return Some(idx as usize)}
        }
        None
    }

    fn get_connected_component(&self, start_idx: usize) -> Vec<usize> {
        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.map.width, self.map.height, &map_starts , &self.map, 200.0
        );
        let mut component: Vec<usize> = Vec::new();
        for (idx, _) in self.map.tiles.iter().enumerate() {
            if dijkstra_map.map[idx] != std::f32::MAX {
                component.push(idx)
            }
        }
        component
    }

    fn fill_all_but_largest_component(&mut self, components: HashMap<usize, Vec<usize>>) {
        let component_sizes: HashMap<usize, usize> = components.iter()
            .map(|(k, v)| (*k, v.len()))
            .collect();
        let largest_component_size = component_sizes.iter()
            .map(|(k, v)| v)
            .max()
            .expect("Found no connected components when building cellular automota map.");
        let largest_components_idxs: Vec<usize> = component_sizes.iter()
            .filter(|(k, v)| *v == largest_component_size)
            .map(|(k, v)| *k)
            .collect();
        let non_largest_components_idxs: Vec<usize> = component_sizes.iter()
            .filter(|(k, v)| *v != largest_component_size)
            .map(|(k, v)| *k)
            .collect();
        println!("{:?}", component_sizes);
        println!("{:?}", largest_components_idxs);
        println!("{:?}", non_largest_components_idxs);
        // Fill all the components that are not one of the largest, and all but
        // one if there are more than one components tied for the largest.
        for idx in non_largest_components_idxs {
            for cidx in components[&idx].iter() {
                self.map.tiles[*cidx] = TileType::Wall;
            }
        }
    }
    // Place the stairs as far away from the player as possible.
    // fn place_stairs(&mut self) {
    //     let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
    //     let map_starts : Vec<usize> = vec![start_idx];
    //     let dijkstra_map = rltk::DijkstraMap::new(
    //         self.map.width, self.map.height, &map_starts , &self.map, 200.0
    //     );
    //     let mut exit_tile = (0, 0.0f32);
    //     for (idx, tile) in self.map.tiles.iter_mut().enumerate() {
    //         if *tile == TileType::Floor {
    //             let distance_to_start = dijkstra_map.map[idx];
    //             if distance_to_start == std::f32::MAX {
    //                 *tile = TileType::Wall;
    //             } else {
    //                 if distance_to_start > exit_tile.1 {
    //                     exit_tile.0 = idx;
    //                     exit_tile.1 = distance_to_start;
    //                 }
    //             }
    //         }
    //     }
    //     self.map.tiles[exit_tile.0] = TileType::DownStairs;
    // }

    // Use cellular noise to create some random contiguous reigons of the map
    // that can be used as pseudo-rooms to spawn entities in.
    fn populate_noise_areas(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);
        for y in 1 .. self.map.height-1 {
            for x in 1 .. self.map.width-1 {
                let idx = self.map.xy_idx(x, y);
                if self.map.tiles[idx] == TileType::Floor {
                    let cell_value = (noise.get_noise(x as f32, y as f32) * 10240.0) as i32;
                    if self.noise_areas.contains_key(&cell_value) {
                        self.noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        self.noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }
    }
}