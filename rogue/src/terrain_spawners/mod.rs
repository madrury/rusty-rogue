use super::{
    Map, Position, Renderable, BlocksTile, SetsBgColor, Name, SimpleMarker,
    SerializeMe, MarkedBuilder, DissipateWhenBurning,
    ChanceToSpawnEntityWhenBurning, RemoveBurningWhenEncroachedUpon,
    DissipateFireWhenEncroachedUpon, RemoveBurningOnUpkeep, EntitySpawnKind,
    StatusIsImmuneToChill, StatusIsImmuneToFire, IsEntityKind, Hazard, Opaque,
    BlessingSelectionTile, TileType
};

pub mod foliage;
pub mod water;
pub mod statues;
pub mod magic;