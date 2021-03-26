
use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};

//------------------------------------------------------------------
// Equipment System Components
//------------------------------------------------------------------
// An entity with this component can be equipped.
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee, Armor
}
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

// Component for a equipped item. Points to the entity that has it equipped.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot
}

// Signals that the owning entity wants to equip an item.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToEquipItem {
    pub item: Entity,
    pub slot: EquipmentSlot,
}

// Signals that the owning entity wants to remove an equipped an item.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToRemoveItem {
    pub item: Entity,
    pub slot: EquipmentSlot,
}