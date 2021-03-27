use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};


//------------------------------------------------------------------
// Monster AI Components
//
// These components hold data needed to cunfigure aand track the state of enemy
// monster AI behaviour.
//------------------------------------------------------------------

// tags monsters if they can act in a given turn.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct CanAct {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MovementRoutingOptions {
    pub avoid_blocked: bool,
    pub avoid_fire: bool,
    pub avoid_chill: bool
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MonsterBasicAI {
    pub only_follow_within_viewshed: bool,
    pub no_visibility_wander: bool,
    pub lost_visibility_keep_following_turns_max: i32,
    pub lost_visibility_keep_following_turns_remaining: i32,
    pub routing_options: MovementRoutingOptions
}
impl MonsterBasicAI {
    pub fn reset_keep_following(&mut self) {
        self.lost_visibility_keep_following_turns_remaining = self.lost_visibility_keep_following_turns_max
    }
    pub fn do_keep_following(&self) -> bool {
        self.lost_visibility_keep_following_turns_remaining > 0
    }
    pub fn decrement_keep_following(&mut self) {
        self.lost_visibility_keep_following_turns_remaining -= 1
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MonsterAttackSpellcasterAI {
    pub distance_to_keep_away: i32,
    pub routing_options: MovementRoutingOptions
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MonsterClericAI {
    pub distance_to_keep_away: i32,
    pub routing_options: MovementRoutingOptions
}