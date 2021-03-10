use super::{World, Map, Rectangle, TileType, Position, spawner, MAP_WIDTH, MAP_HEIGHT, MAP_SIZE};
mod simple_map;
use simple_map::{SimpleMapBuilder};
mod common;
use common::*;

/*
Various apgorithms for map building.

  Simple Map Builder: PLaces some rectangular rooms and then connects them
    with L shaped corridors.
*/

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn map(&self) -> Map;
    fn starting_position(&self) -> Position;

    fn take_snapshot(&mut self);
    fn snapshot_history(&self) -> Vec<Map>;
}

pub fn random_builder(depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(depth))
}