use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
// use super::{ElementalDamageKind, EntitySpawnKind};


//------------------------------------------------------------------
// Spawn Components
//
// These components influence entity spawning behaviour.
//------------------------------------------------------------------

// Enumerates the various types of entities that can spawn. Used to tag a spawn
// request to lookup the appropriate function used to spawn the entity.
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum EntitySpawnKind {
    Fire {spread_chance: i32, dissipate_chance: i32},
    Chill {spread_chance: i32, dissipate_chance: i32},
}

// Tags entitys with a specific type. This is used to lookup the apropriate
// destructor.
#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct IsEntityKind {
    pub kind: EntitySpawnKind
}

// Component indicates that a targeted effect spawns new entities when resolved.
#[derive(Component, ConvertSaveload, Clone)]
pub struct SpawnsEntityInAreaWhenTargeted {
    pub radius: i32,
    pub kind: EntitySpawnKind
}

// Component indicates that the entity has a random chance to spawn entities in
// an adjacent space.
#[derive(Component, ConvertSaveload, Clone)]
pub struct ChanceToSpawnAdjacentEntity {
    pub chance: i32,
    pub kind: EntitySpawnKind
}

// Component indicates that an entity will spawn another entity in the same
// space when burning (so, for example, grass will spawn fire when burned).
#[derive(Component, ConvertSaveload, Clone)]
pub struct ChanceToSpawnEntityWhenBurning {
    pub kind: EntitySpawnKind,
    pub chance: i32
}

// An entity with this component has a chance to dissipate every turn.
#[derive(Component, ConvertSaveload, Clone)]
pub struct ChanceToDissipate {
    pub chance: i32,

}
// Component indicates that an entity dissipates (i.e. is destroyed) when it is
// burning. I.e., grass is destroyed when burned.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DissipateWhenBurning {}

// An entity with this component has dissipated and should be removed from the
// ECS at the end of the current turn.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToDissipate {}