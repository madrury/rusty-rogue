use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::{RGB, Point};


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

// An entity with this component can be picked up.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PickUpable {}

// An entity with this component can be picked up.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Useable {}

// An entity with this component can be thrown.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Throwable {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Castable {}

// An entity with this component can be used as a targeted effect.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Targeted {pub verb: String}

// An entity with this component can be used as an untargeted effect.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Untargeted {pub verb: String}

// An entity with this component blocks the tile that it occupies.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

// An entity with this component is consumed upon use.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Consumable {}

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

//------------------------------------------------------------------
// Location components.
// These components tag an entity as in some (abstract, non-physical) location.
// In someone's inventory or spellbook, for example.
//------------------------------------------------------------------
// Component for a held item. Points to the entity that owns it.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity
}

//------------------------------------------------------------------
// Animation System Components
//------------------------------------------------------------------
// Represents an atomic piece of game animation.
#[derive(Component)]
pub struct ParticleLifetime {
    pub lifetime : f32,
    // How many milliseconds after the animation starts should this particle be
    // displayed?
    pub delay: f32,
    // For how long should this particle be displayed?
    pub displayed: bool,
    pub x: i32,
    pub y: i32,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType
}

// Component for effects that create an area of effect animation when
// thrown.
#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffectAnimationWhenTargeted {
    pub radius: i32,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType
}

//------------------------------------------------------------------
// Entity Stats Components
//------------------------------------------------------------------
// Component holding the combat statistics of an entity.
// TODO: This should probably be broken into two comonents? HealthStats and
// MeleeStats?
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
    pub fn increase_max_hp(&mut self, amount: i32) {
        self.max_hp += amount;
    }
}

//------------------------------------------------------------------
// Hunger System Components
//------------------------------------------------------------------
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    pub state_duration: i32,
    pub time: i32,
    pub tick_damage: i32
}
impl HungerClock {
    pub fn satiate(&mut self) {
        self.state = HungerState::WellFed;
        self.time = self.state_duration;
    }
}

//------------------------------------------------------------------
// Magic System Components
//------------------------------------------------------------------
#[derive(Component, ConvertSaveload, Clone)]
pub struct InSpellBook {
    pub owner: Entity
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpellCharges {
    pub max_charges: i32,
    pub charges: i32,
    pub regen_time: i32,
    pub time: i32
}
impl SpellCharges {
    pub fn expend_charge(&mut self) {
        self.charges = i32::max(0, self.charges - 1);
        self.time = 0;
    }
    // Return value indicated if a cast has recharged.
    pub fn tick(&mut self) -> bool {
        self.time = i32::min(self.time + 1, self.regen_time);
        if self.time == self.regen_time && self.charges < self.max_charges {
            self.charges += 1;
            return true
        }
        false
    }
}


//------------------------------------------------------------------
// Monster AI Components
//------------------------------------------------------------------
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

//------------------------------------------------------------------
// Game effects components
//------------------------------------------------------------------
// Component for effects that increase the user's maximum hp.
#[derive(Component, ConvertSaveload, Clone)]
pub struct IncreasesMaxHpWhenUsed {
    pub amount: i32
}

// Component for effects that inflict damage when thrown or cast.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamageWhenTargeted {
    pub damage: i32
}

// Component for effects with an area of effect.
#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffectWhenTargeted {
    pub radius: i32
}

// Component for effects that inflict the frozen status.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsFreezingWhenTargeted {
    pub turns: i32
}

// Component for effects that inflict the burning status.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsBurningWhenTargeted {
    pub turns: i32,
    pub tick_damage: i32
}

// Component for effects that grant a MeleeAttackBonus
#[derive(Component, ConvertSaveload, Clone)]
pub struct GrantsMeleeAttackBonus {
    pub bonus: i32
}

// Component for effects that grant a MeleeAttackBonus
#[derive(Component, ConvertSaveload, Clone)]
pub struct GrantsMeleeDefenseBonus {
    pub bonus: i32
}

// An entity with this component, when used, restores all of the users hp.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesFullHealing {}

// An entity with this component, when used, restores the user to well fed.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesFullFood {}

// An entity with this component, when used, teleports the user to a random
// position.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MovesToRandomPosition {}

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

// The entiity has requested to teleport to a random map position.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToMoveToRandomPosition {}

// Signals that the entity has damage queued, but not applied.
#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToTakeDamage {
    pub amounts: Vec<i32>
}
impl WantsToTakeDamage {
    // Since the ApplyMeleeDamage component can contain *multiple* instances of
    // damage, we need to distinguish the case of the first instance of damage
    // from the subsequent. This function encapsulates this switch.
    pub fn new_damage(store: &mut WriteStorage<WantsToTakeDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amounts.push(amount);
        } else {
            let dmg = WantsToTakeDamage{ amounts: vec![amount] };
            store.insert(victim, dmg)
                .expect("Unable to insert SufferDamage component.");
        }
    }
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