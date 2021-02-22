use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB};


#[derive(Component, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Name {
    pub name: String
}

#[derive(Component, Debug)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component, Debug)]
pub struct Monster {}

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

#[derive(Component)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32
}

// Signals that the entity has entered into melee combat with a chosen target.
#[derive(Component)]
pub struct WantsToMelee {
    pub target: Entity
}

// Signals that the entity has damage queued, but not applied.
#[derive(Component)]
pub struct SufferDamage {
    pub amounts: Vec<i32>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amounts.push(amount);
        } else {
            let dmg = SufferDamage{ amounts: vec![amount] };
            store.insert(victim, dmg)
                .expect("Unable to insert SufferDamage component.");
        }
    }
}