use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB, Point};


// Binary Components:
// These components indicate something, the type of entity or some binary
// behaviour, by their presence or absence.
//------------------------------------------------------------------

// The singular entity with this component is the player.
#[derive(Component)]
pub struct Player {}

// An entity with this component is a monster.
#[derive(Component)]
pub struct Monster {}

// An entity with this component can be picked up.
#[derive(Component)]
pub struct PickUpable {}

// An entity with this component can be picked up.
#[derive(Component)]
pub struct Useable {}

// An entity with this component can be thrown.
#[derive(Component)]
pub struct Throwable {}

// An entity with this component blocks the tile that it occupies.
#[derive(Component)]
pub struct BlocksTile {}

// An entity with this component is consumed upon use.
#[derive(Component)]
pub struct Consumable {}

// An entity with this component, when used, restores all of the users hp.
#[derive(Component)]
pub struct ProvidesHealing {}


// Data Components:
// These components have some data associated with them.
//------------------------------------------------------------------

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
    pub order: i32,
}

// Component for a held item. Points to the entity that owns it.
#[derive(Component)]
pub struct InBackpack {
    pub owner: Entity
}

// Component for items or spells that inflict damage when thrown or cast.
#[derive(Component)]
pub struct InflictsDamageWhenThrown {
    pub damage: i32
}

// Component for items or spells with an area of effect.
#[derive(Component)]
pub struct AreaOfEffectWhenThrown {
    pub radius: i32
}

// Component for items that inflict the frozen status.
#[derive(Component)]
pub struct InflictsFreezingWhenThrown {
    pub turns: i32
}

// Component for items that inflict the burning status.
#[derive(Component)]
pub struct InflictsBurningWhenThrown {
    pub turns: i32,
    pub tick_damage: i32
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
impl CombatStats {
    pub fn take_damage(&mut self, damage: i32) {
        self.hp = i32::max(0, self.hp - damage)
    }
}

// Status Effect Components
//------------------------------------------------------------------

// Component indicating the entity is frozen.
#[derive(Component)]
pub struct StatusIsFrozen {
    pub remaining_turns: i32
}
impl StatusIsFrozen {
    pub fn new_status(store: &mut WriteStorage<StatusIsFrozen>, victim: Entity, turns: i32) {
        if let Some(frozen) = store.get_mut(victim) {
            frozen.remaining_turns = turns;
        } else {
            let frozen = StatusIsFrozen{remaining_turns: turns};
            store.insert(victim, frozen)
                .expect("Unable to insert StatusIsFrozen component.");
        }
    }
    pub fn is_frozen(self) -> bool {
        self.remaining_turns > 0
    }
    pub fn tick(&mut self) {
        self.remaining_turns -= 1
    }
}

// Component indicating the entity is burning.
#[derive(Component)]
pub struct StatusIsBurning {
    pub remaining_turns: i32,
    pub tick_damage: i32,
}
impl StatusIsBurning {
    pub fn new_status(store: &mut WriteStorage<StatusIsBurning>, victim: Entity, turns: i32, dmg: i32) {
        if let Some(burning) = store.get_mut(victim) {
            burning.remaining_turns = turns;
            burning.tick_damage = i32::max(dmg, burning.tick_damage);
        } else {
            let burning = StatusIsBurning{remaining_turns: turns, tick_damage: dmg};
            store.insert(victim, burning)
                .expect("Unable to insert StatusIsBurning component.");
        }
    }
    pub fn is_burning(self) -> bool {
        self.remaining_turns > 0
    }
    pub fn tick(&mut self) {
        self.remaining_turns -= 1
    }
}


// Signaling Components
//------------------------------------------------------------------
// These components are used when processing changes to game state to signal
// that some change needs to occur.

// Signals that the entity has entered into melee combat with a chosen target.
#[derive(Component)]
pub struct WantsToMeleeAttack {
    pub target: Entity
}

// Signals that an entity wants to pick up an item.
#[derive(Component)]
pub struct WantsToPickupItem {
    pub by: Entity,
    pub item: Entity
}

// Signals that the owning entity wants to use an item.
#[derive(Component)]
pub struct WantsToUseItem {
    pub item: Entity,
}

// Signals that the owning entity wants to throw an item.
#[derive(Component)]
pub struct WantsToThrowItem {
    pub item: Entity,
    pub target: Point,
}

// Signals that the entity has damage queued, but not applied.
#[derive(Component)]
pub struct ApplyDamage {
    pub amounts: Vec<i32>
}
impl ApplyDamage {
    // Since the ApplyMeleeDamage component can contain *multiple* instances of
    // damage, we need to distinguish the case of the first instance of damage
    // from the subsequent. This function encapsulates this switch.
    pub fn new_damage(store: &mut WriteStorage<ApplyDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amounts.push(amount);
        } else {
            let dmg = ApplyDamage{ amounts: vec![amount] };
            store.insert(victim, dmg)
                .expect("Unable to insert SufferDamage component.");
        }
    }
}