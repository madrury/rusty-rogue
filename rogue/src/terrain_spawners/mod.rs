
use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, DissipateWhenEnchroachedUpon,
    SpawnEntityWhenEncroachedUpon, ChanceToSpawnEntityWhenBurning,
    EntitySpawnKind, Hazard, Opaque, color, noise
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

pub mod foliage;


pub fn spawn_terrain(ecs: &mut World, map: &Map, depth: i32) {
    foliage::spawn_grove_grass(ecs, map);
}