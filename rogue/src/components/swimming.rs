use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};

//------------------------------------------------------------------
// Components and data structures for the swimming system.
//------------------------------------------------------------------
#[derive(Component, ConvertSaveload, Clone)]
pub struct SwimStamina {
    pub max_stamina: i32,
    pub stamina: i32,
    pub drowning_damage: i32
}