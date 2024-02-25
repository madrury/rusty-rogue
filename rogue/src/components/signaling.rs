use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::Point;

use crate::TargetingVerb;

//------------------------------------------------------------------
// Signaling Components
// These components are used when processing changes to game state to signal
// that some change needs to occur or effect needs to be applied.
//
// The naming convention "WantsToXYZ" indicated that the owning entity wants to
// apply some effect.
//------------------------------------------------------------------

// Signals that the entity has entered into melee combat with a chosen target.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToMeleeAttack {
    pub target: Entity
}

// Signals that an entity wants to pick up an item.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToPickupItem {
    pub by: Entity,
    pub item: Entity
}

// Signals that the owning entity wants to use an untargeted effect.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToUseUntargeted {
    pub thing: Entity,
}

// Signals that the owning entity wants to use a targeted effect.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToUseTargeted {
    pub thing: Entity,
    pub target: Point,
    pub verb: TargetingVerb,
}

// The entiity has requested to move to a specific map position.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToMoveToPosition {
    pub pt: Point,
    // Overide any movement options check - force the monster into the position.
    // Useful for stuff like teleportation, that we want to be somwhat
    // dangerous.
    pub force: bool
}

// The entiity has requested to teleport to a random map position.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToMoveToRandomPosition {}

// Signals that the entity has damage queued, but not applied.
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ElementalDamageKind {
    Physical, Fire, Chill, Hunger, Drowning
}
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToTakeDamage {
    pub amounts: Vec<i32>,
    pub kinds: Vec<ElementalDamageKind>
}
impl WantsToTakeDamage {
    // Since the WantsToTakeDamage component can contain *multiple* instances of
    // damage, we need to distinguish the case of the first instance of damage
    // from the subsequent. This function encapsulates this switch.
    pub fn new_damage(
        store: &mut WriteStorage<WantsToTakeDamage>,
        victim: Entity,
        amount: i32,
        kind: ElementalDamageKind
    ) {
        if let Some(wants_damage) = store.get_mut(victim) {
            wants_damage.amounts.push(amount);
            wants_damage.kinds.push(kind);
        } else {
            let dmg = WantsToTakeDamage{
                amounts: vec![amount],
                kinds: vec![kind]
            };
            store.insert(victim, dmg)
                .expect("Unable to insert WantsToTakeDamage component.");
        }
    }
}
