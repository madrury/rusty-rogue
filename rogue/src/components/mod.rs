use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::{RGB, Point};

use crate::map_builders::{BgColorMap, FgColorMap};

use super::{GameLog};

pub mod animation;
pub mod hunger;
pub mod swimming;
pub mod magic;
pub mod equipment;
pub mod ai;
pub mod game_effects;
pub mod spawn_despawn;
pub mod status_effects;
pub mod signaling;
pub mod melee;
pub mod targeting;


//----------------------------------------------------------------------------
// Tagging Components
// These components tag an entity as some type.
//----------------------------------------------------------------------------
// The singular entity with this component is the player.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

// An entity with this component is a monster.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Monster {}

// An entity with this component is a hazard.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Hazard {}

// An entity with this component can be picked up.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum PickupDestination {
    Backpack, Spellbook, BlessingOrbBag
}
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PickUpable {
    pub destination: PickupDestination
}

// An entity with this component can be used from the item menu.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Useable {}

// An entity with this component can be thrown.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Throwable {}

// An entity with this component can be used as an untargeted effect.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Untargeted {
    pub verb: String
}

// An entity with this component blocks the tile that it occupies.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

// An entity with this component is consumed upon use.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Consumable {}

// An entity with this component blocks vision.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Opaque {}

// An entity with this component tramples the contents of any tile it enters.
// For example, entities with this component will encroach on tall grass and
// trample it into short grass.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Tramples {}

// Component tags an entity as bloodies. Has various visual and game effects.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Bloodied {}

//------------------------------------------------------------------
// Core Data Components:
// These components have some data associated with them core to gameplay.
//------------------------------------------------------------------
// Component for all entities that have a position within the map.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl Position {
    pub fn to_point(&self) -> Point {
        Point {x: self.x, y: self.y}
    }
}

// Component for all named enetities.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String
}

// Component for all entities that have a field of view.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    // Flag indicating if the entities field of view needs to be recomputed.
    pub dirty: bool,
}

// Component for all entities that need to be rendered on the console.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType,
    pub order: i32,
    pub visible_out_of_fov: bool,
}

// Sets a colormap used to lookup fg colors for entity..
#[derive(Component, ConvertSaveload, Clone)]
pub struct UseFgColorMap {
    pub cmap: FgColorMap
}
impl UseFgColorMap {
    pub fn set(&mut self, cmap: FgColorMap) {
        self.cmap = cmap
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct UseFgColorMapWhenBloodied {
    pub cmap: FgColorMap
}

// Sets a colormap used to lookup bg colors for entity.
#[derive(Component, ConvertSaveload, Clone)]
pub struct UseBgColorMap {
    pub cmap: BgColorMap
}
impl UseBgColorMap {
    pub fn set(&mut self, cmap: BgColorMap) {
        self.cmap = cmap
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct UseBgColorMapWhenBloodied {
    pub cmap: BgColorMap
}

// An entity with this component sets the bg-color for their tile. It has no
// gameplay application, it's only a visual effect.
#[derive(Component, ConvertSaveload, Clone)]
pub struct SetsBgColor {
    pub order: i32
}

// Component for a held item. Points to the entity that owns it.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity
}

//----------------------------------------------------------------------------
// Serialization Components.
// Non-gameplay components used to help game saving and loading.
//----------------------------------------------------------------------------
// Marker for entities serialized when saving the game.
pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map
}