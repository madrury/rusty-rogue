use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::{RGB, Point};


// Marker for entities serialized when saving the game.
pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map
}


// Binary Components:
// These components indicate something, the type of entity or some binary
// behaviour, by their presence or absence.
//------------------------------------------------------------------

// The singular entity with this component is the player.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

// An entity with this component is a monster.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Monster {}

// An entity with this component can be picked up.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PickUpable {}

// An entity with this component can be picked up.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Useable {}

// An entity with this component can be thrown.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Throwable {}

// An entity with this component blocks the tile that it occupies.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

// An entity with this component is consumed upon use.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Consumable {}

// An entity with this component, when used, restores all of the users hp.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesHealing {}



//------------------------------------------------------------------
// Data Components:
// These components have some data associated with them.
//------------------------------------------------------------------

// Component for all entities that have a position within the map.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
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
}

// An entity with this component can be equipped.
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee, Armor
}
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

// Component for a held item. Points to the entity that owns it.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity
}

// Component for a equipped item. Points to the entity that has it equipped.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot
}

// Component for items or spells that inflict damage when thrown or cast.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamageWhenThrown {
    pub damage: i32
}

// Component for items or spells with an area of effect.
#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffectWhenThrown {
    pub radius: i32
}

// Component for items that inflict the frozen status.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsFreezingWhenThrown {
    pub turns: i32
}

// Component for items that inflict the burning status.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsBurningWhenThrown {
    pub turns: i32,
    pub tick_damage: i32
}

// Component for items that create an area of effect animation when
// thrown.
#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffectAnimationWhenThrown {
    pub radius: i32,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType
}

// Comonent holding data determining a monster's movement behaviour.
#[derive(Component, ConvertSaveload, Clone)]
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
#[derive(Component, ConvertSaveload, Clone)]
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
    pub fn full_heal(&mut self) {
        self.hp = self.max_hp
    }
    pub fn heal_amount(&mut self, amount: i32) {
        self.hp = i32::min(self.max_hp, self.hp + amount)
    }
}

#[derive(Component)]
pub struct ParticleLifetime {
    pub lifetime : f32,
    pub delay: f32,
    pub displayed: bool,
    pub x: i32,
    pub y: i32,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType
}

//------------------------------------------------------------------
// Status Effect Components
//------------------------------------------------------------------

// Component indicating the entity is frozen.
#[derive(Component, ConvertSaveload, Clone)]
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
#[derive(Component, ConvertSaveload, Clone)]
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

// Signals that the owning entity wants to use an item.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item: Entity,
}

// Signals that the owning entity wants to throw an item.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToThrowItem {
    pub item: Entity,
    pub target: Point,
}

// Signals that the owning entity wants to throw an item.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToEquipItem {
    pub item: Entity,
    pub slot: EquipmentSlot,
}

// Signals that the entity has damage queued, but not applied.
#[derive(Component, ConvertSaveload, Clone)]
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