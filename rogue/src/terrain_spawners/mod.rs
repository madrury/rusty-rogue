
use self::water::WaterSpawnTable;

use super::{
    Map, Position, Renderable, BlocksTile, SetsBgColor, Name, SimpleMarker,
    SerializeMe, MarkedBuilder, DissipateWhenBurning,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    ChanceToSpawnEntityWhenBurning, RemoveBurningWhenEncroachedUpon,
    DissipateFireWhenEncroachedUpon, RemoveBurningOnUpkeep, EntitySpawnKind,
    StatusIsImmuneToChill, StatusIsImmuneToFire, IsEntityKind, Hazard, Opaque,
    BlessingSelectionTile,
    TileType, color, noise
};
use rltk::{RandomNumberGenerator};
use specs::prelude::*;

pub mod foliage;
pub mod water;
pub mod statues;
pub mod magic;


pub fn random_water_spawn_table(rng: &mut RandomNumberGenerator, map: &Map) -> water::WaterSpawnTable {
    let water_roll = rng.roll_dice(1, 3);
    match water_roll {
        1 => water::WaterSpawnTable::new(),
        2 => water::large_lakes_spawn_table(map),
        3 => water::small_lakes_spawn_table(map),
        _ => panic!("Rolled to high on water spawning.")
    }
}

pub fn spawn_terrain(ecs: &mut World, _depth: i32) {
    let grass_roll;
    let statue_roll;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        grass_roll = rng.roll_dice(1, 3);
        statue_roll = rng.roll_dice(1, 3)
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