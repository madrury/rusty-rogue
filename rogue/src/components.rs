use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB};


// Binary Components:
// These components indicate something by their presence or absence.

// The singular entity with this component is the player.
#[derive(Component, Debug)]
pub struct Player {}

// An entity with this component is a monster.
#[derive(Component, Debug)]
pub struct Monster {}

// Component whose presence indicates that the entity blocks the tile they
// occupy.
#[derive(Component)]
pub struct BlocksTile {}


// Data Components:
// These components have some data associated with them.

// Component for all entities that have a position within the map.
#[derive(Component, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// Component for all named enetities.
#[derive(Component)]
pub struct Name {
    pub name: String
}

// Component for all entities that have a field of view.
#[derive(Component, Debug)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    // Flag indicating if the entities field of view needs to be recomputed.
    pub dirty: bool,
}

// Component for all entities that need to be rendered on the console.
#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

// Comonent holding data determining a monster's movement behaviour.
#[derive(Component)]
pub struct MonsterMovementAI {
    pub only_follow_within_viewshed: bool,
    pub no_visibility_wander: bool,
    pub lost_visibility_keep_following_turns_max: i32,
    pub lost_visibility_keep_following_turns_remaining: i32
}
impl MonsterMovementAI {
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

// Component holding the combat statistics of an entity.
#[derive(Component)]
pub struct CombatStats {
    // Health.
    pub max_hp: i32,
    pub hp: i32,
    // Raw melee stats.
    pub defense: i32,
    pub power: i32
}

// Signaling Components
// These components are used when processing changes to game state to signal
// that some change needs to occur.

// Signals that the entity has entered into melee combat with a chosen target.
#[derive(Component)]
pub struct MeleeAttack {
    pub target: Entity
}

// Signals that the entity has damage queued, but not applied.
#[derive(Component)]
pub struct ApplyMeleeDamage {
    pub amounts: Vec<i32>
}
impl ApplyMeleeDamage {
    pub fn new_damage(store: &mut WriteStorage<ApplyMeleeDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amounts.push(amount);
        } else {
            let dmg = ApplyMeleeDamage{ amounts: vec![amount] };
            store.insert(victim, dmg)
                .expect("Unable to insert SufferDamage component.");
        }
    }
}