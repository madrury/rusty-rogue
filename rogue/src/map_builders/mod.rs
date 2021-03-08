use super::{World, Map, Rectangle, TileType, Position, spawner, MAP_WIDTH, MAP_HEIGHT, MAP_SIZE};
mod simple_map;
use simple_map::{SimpleMapBuilder};
mod common;
use common::*;


trait MapBuilder {
    fn build(depth: i32) -> (Map, Position);
    fn spawn(map: &Map, ecs: &mut World, depth: i32);
}

pub fn build_random_map(depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(depth)
}

pub fn spawn(map: &Map, ecs: &mut World, depth: i32) {
    SimpleMapBuilder::spawn(map, ecs, depth)
}