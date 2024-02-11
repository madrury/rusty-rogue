use crate::map_builders::NoiseMaps;

use super::{
    Map, Position, Renderable, BlocksTile, SetsBgColor, Name, SimpleMarker,
    SerializeMe, MarkedBuilder, DissipateWhenBurning,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    ChanceToSpawnEntityWhenBurning, RemoveBurningWhenEncroachedUpon,
    DissipateFireWhenEncroachedUpon, RemoveBurningOnUpkeep, EntitySpawnKind,
    StatusIsImmuneToChill, StatusIsImmuneToFire, IsEntityKind, Hazard, Opaque,
    BlessingSelectionTile,
    TileType, color
};

pub mod foliage;
pub mod water;
pub mod statues;
pub mod magic;