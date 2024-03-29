use super::{
    World, Map, TileType, Point, entity_spawners, terrain_spawners,
    DEBUG_VISUALIZE_MAPGEN, MAP_WIDTH, MAP_HEIGHT, MAP_SIZE
};
mod noise;
pub use noise::*;
mod simple_map;
use simple_map::SimpleMapBuilder;
mod cellular_automata_map;
use cellular_automata_map::CellularAutomataBuilder;
mod common;
use common::*;
mod components;
use components::*;
mod colormaps;
pub use colormaps::{ColorMaps, FgColorMap, BgColorMap};


pub trait MapBuilder {
    fn map(&self) -> Map;
    fn build_map(&mut self) -> (NoiseMaps, ColorMaps);
    fn spawn_blessing_tile(&mut self, ecs: &mut World);
    fn spawn_water(&mut self, ecs: &mut World);
    fn spawn_terrain(&mut self, ecs: &mut World);
    fn spawn_entities(&mut self, ecs: &mut World);
    // Compute a position to place the player.
    fn starting_position(&self, ecs: &World) -> Point;
    // Compute a position to place the stairs.
    fn stairs_position(&self, ecs: &World) -> Point;

    fn take_snapshot(&mut self);
    fn snapshot_history(&self) -> Vec<Map>;
}

pub fn random_builder(depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let roll = rng.roll_dice(1, 2);
    match roll {
        1 => Box::new(SimpleMapBuilder::new(depth)),
        2 => Box::new(CellularAutomataBuilder::new(depth)),
        _ => panic!("Rolled too high when choosing a map builder.")
    }
}