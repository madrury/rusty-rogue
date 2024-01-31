
use super::{
    Map, Position, Renderable, BlocksTile, SetsBgColor, Name, SimpleMarker,
    SerializeMe, MarkedBuilder, DissipateWhenBurning,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    ChanceToSpawnEntityWhenBurning, RemoveBurningWhenEncroachedUpon,
    DissipateFireWhenEncroachedUpon, RemoveBurningOnUpkeep, EntitySpawnKind,
    IsEntityKind, Hazard, Opaque, BlessingSelectionTile, TileType, color, noise
};
use rltk::{RandomNumberGenerator};
use specs::prelude::*;

pub mod foliage;
pub mod water;
pub mod statues;
pub mod magic;


pub fn spawn_terrain(ecs: &mut World, _depth: i32) {
    let grass_roll;
    let water_roll;
    let statue_roll;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        grass_roll = rng.roll_dice(1, 3);
        water_roll = rng.roll_dice(1, 3);
        statue_roll = rng.roll_dice(1, 3)
    }
    match water_roll {
        1 => {},
        2 => water::spawn_large_lakes(ecs),
        3 => water::spawn_small_lakes(ecs),
        _ => panic!("Rolled to high on water spawning.")
    }
    magic::spawn_blessing_tile_aparatus(ecs);
    match statue_roll {
        1 => statues::spawn_statues(ecs),
        _ => {}
    }

    match grass_roll {
        1 => foliage::spawn_short_grass(ecs),
        2 => foliage::spawn_sporadic_grass(ecs),
        3 => foliage::spawn_grove_grass(ecs),
        _ => panic!("Rolled to high on foliage spawning.")
    }
}