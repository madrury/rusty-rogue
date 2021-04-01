
use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, DissipateWhenEnchroachedUpon,
    SpawnEntityWhenEncroachedUpon, ChanceToSpawnEntityWhenBurning,
    EntitySpawnKind, Hazard, Opaque, color, noise
};
use rltk::{RandomNumberGenerator};
use specs::prelude::*;

pub mod foliage;


pub fn spawn_terrain(ecs: &mut World, map: &Map, _depth: i32) {
    let roll;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 3)
    }
    match roll {
        1 => foliage::spawn_short_grass(ecs, map),
        2 => foliage::spawn_sporadic_grass(ecs, map),
        3 => foliage::spawn_grove_grass(ecs, map),
        _ => panic!("Rolled to high on terrain spawning.")
    }
}