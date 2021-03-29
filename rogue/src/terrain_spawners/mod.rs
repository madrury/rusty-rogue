
use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    EntitySpawnKind, Hazard
};
use::rand;
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

mod foliage;


pub fn spawn_region(ecs: &mut World, map: &Map, region: &[usize], depth: i32) {
    foliage::spawn_short_grass_patch(ecs, map, region);
}