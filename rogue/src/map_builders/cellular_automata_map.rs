use std::collections::HashMap;
use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::terrain_spawners::water::WaterSpawnTable;
use super::{MapBuilder, enumerate_connected_components, fill_all_but_largest_component};
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

    fn build_map(&mut self) -> terrain_spawners::water::WaterSpawnTable {
        let mut rng = RandomNumberGenerator::new();
        self.ititialize(&mut rng);
        self.take_snapshot();
        for _i in 0..N_ITERATIONS {
            self.update();
            self.take_snapshot();
        }
        // The use of the map.blocked array here is confusing. We need to call
        // initialize_blocked *twice*.
        //
        //   - The *initial calls* to initialize_blocked are to support our use
        //   of the rltk::Dijkstra map when finding the connected components of
        //   our map. This API uses the blocked array to determine the open
        //   neighbours of a tile.
        //  - The *final* call to initialize_blocked is after we have filled in
        //  the smaller connected components with TileType::Wall, meaning our
        //  map has a single connected compoenent. We need to re-compute the
        //  blocked array so that it can be used to determine free squares when
        //  spawning entities.
        self.map.intitialize_blocked();
        let components = enumerate_connected_components(&self.map);
        fill_all_but_largest_component(&mut self.map, components);
        self.map.intitialize_blocked();

        // Now we carve out space to spawn water.
        let water_spawn_table = terrain_spawners::random_water_spawn_table(&mut rng, &self.map);
        for elem in &water_spawn_table.shallow {
            let idx = self.map.xy_idx(elem.x, elem.y);
            self.map.tiles[idx] = TileType::Floor
        }
        for elem in &water_spawn_table.deep {
            let idx = self.map.xy_idx(elem.x, elem.y);
            self.map.tiles[idx] = TileType::Floor
        }
        // Carving out water maybe creates new connected components, so reduce
        // back to a single connected component.
        self.map.intitialize_blocked();
        let components = enumerate_connected_components(&self.map);
        fill_all_but_largest_component(&mut self.map, components);
        self.map.intitialize_blocked();

        self.populate_noise_areas();
        self.map.intitialize_opaque();
        self.map.intitialize_ok_to_spawn();

        water_spawn_table
    }

    fn spawn_water(&mut self, ecs: &mut World, water_spawn_table: &WaterSpawnTable) {
        for e in &water_spawn_table.shallow {
            {
                let mut map = ecs.write_resource::<Map>();
                let idx = map.xy_idx(e.x, e.y);
                if map.blocked[idx] {
                    continue;
                }
                map.ok_to_spawn[idx] = true;
            }
            terrain_spawners::water::shallow_water(ecs, e.x, e.y, e.fgcolor, e.bgcolor);
        }
        for e in &water_spawn_table.deep {
            {
                let map = ecs.read_resource::<Map>();
                let idx = map.xy_idx(e.x, e.y);
                if map.blocked[idx] {
                    continue;
                }
            }
            terrain_spawners::water::deep_water(ecs, e.x, e.y, e.fgcolor, e.bgcolor);
        }
    }

    fn spawn_terrain(&mut self, ecs: &mut World) {
        terrain_spawners::spawn_terrain(ecs, self.depth);
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        entity_spawners::spawn_monsters(ecs, self.depth);
        for area in self.noise_areas.iter() {
            entity_spawners::spawn_items_in_region(ecs, area.1, self.depth);
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

    // Populate the map with independent Wall tiles samples as random,
    // independent coin flips.
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