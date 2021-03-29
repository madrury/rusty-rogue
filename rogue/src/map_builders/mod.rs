use super::{
    World, Map, TileType, Position, entity_spawners, terrain_spawners,
    DEBUG_VISUALIZE_MAPGEN, MAP_WIDTH, MAP_HEIGHT, MAP_SIZE
};
mod simple_map;
use simple_map::{SimpleMapBuilder};
mod cellular_automata_map;
use cellular_automata_map::{CellularAutomataBuilder};
mod common;
use common::*;

/*
Various apgorithms for map building.

  Simple Map Builder: PLaces some rectangular rooms and then connects them
    with L shaped corridors.
  Cellular Automota Builder: Fills the rap with binary random noise, then
    smoothes it out with a cellular automata.
*/

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_terrain(&mut self, ecs: &mut World);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn map(&self) -> Map;
    fn starting_position(&self) -> Position;

    fn take_snapshot(&mut self);
    fn snapshot_history(&self) -> Vec<Map>;
}

pub fn random_builder(depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(depth))
    // Box::new(CellularAutomataBuilder::new(depth))
}