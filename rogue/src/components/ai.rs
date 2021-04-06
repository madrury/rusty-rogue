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
    pub avoid_chill: bool,
    pub avoid_water: bool,
    pub avoid_steam: bool,
    pub avoid_smoke: bool,
    pub avoid_lava: bool,
    pub avoid_brimstone: bool,
    pub avoid_ice: bool,
}
#[derive(Component, ConvertSaveload, Clone)]
pub struct MonsterMovementRoutingOptions {
    pub options: MovementRoutingOptions
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MonsterBasicAI {
    pub only_follow_within_viewshed: bool,
    // Should the monster wander about when they cannot see the player.
    pub no_visibility_wander: bool,
    // The chance for the entity to disregard any other action, and move to a
    // random adjacent tile. Useful for entities like bats that take an erratic
    // path towards their target.
    pub chance_to_move_to_random_adjacent_tile: i32,
    // Should the monster try to escape when at low health?
    pub escape_when_at_low_health: bool,
    // These attributes allow a monster to continue following the plyer even
    // when the player moves out of the monsters viewrange.
    pub lost_visibility_keep_following_turns_max: i32,
    pub lost_visibility_keep_following_turns_remaining: i32,
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
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum SupportSpellcasterKind {
    Cleric,
    EnchanterAttack,
    EnchanterDefense,
}
#[derive(Component, ConvertSaveload, Clone)]
pub struct MonsterSupportSpellcasterAI {
    pub support_kind: SupportSpellcasterKind,
    pub distance_to_keep_away_from_player: i32,
    pub distance_to_keep_away_from_monsters: i32,
}