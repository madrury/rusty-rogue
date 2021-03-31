
use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    EntitySpawnKind, Hazard, color, noise
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

mod foliage;


pub fn spawn_terrain(ecs: &mut World, map: &Map, depth: i32) {
    foliage::spawn_sporadic_grass(ecs, map);
}