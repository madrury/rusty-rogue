use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::{RGB, Point};
use super::{GameLog};

pub mod animation;
pub mod hunger;
pub mod magic;
pub mod ai;
pub mod game_effects;
pub mod spawn_despawn;


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

// An entity with this component is a hazard.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Hazard {}

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
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TargetingKind {
    Simple,
    AreaOfEffect {radius: f32},
    AlongRay {until_blocked: bool}
}
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Targeted {
    pub verb: String,
    pub range: f32,
    pub kind: TargetingKind
}

// An entity with this component can be used as an untargeted effect.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Untargeted {
    pub verb: String
}

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
impl Position {
    pub fn to_point(&self) -> Point {
        Point {x: self.x, y: self.y}
    }
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

// Tags entitys with a specific type. This is used to lookup the apropriate
// destructor.
#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct IsEntityKind {
    pub kind: EntitySpawnKind
}

// Component for a held item. Points to the entity that owns it.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity
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
// Status Effect Components
//------------------------------------------------------------------
// TODO: Figure out how to write a generalized tick function.

// Component indicating the entity is frozen.
#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsFrozen {
    pub remaining_turns: i32
}
impl StatusIsFrozen {
    // Attempt to add the burning status to an enetity and return a boolean
    // indicating if this was successful.
    pub fn new_status(
        store: &mut WriteStorage<StatusIsFrozen>,
        immune: &ReadStorage<StatusIsImmuneToChill>,
        victim: Entity,
        turns: i32) -> bool
    {
        if immune.get(victim).is_some() {
            return false
        }
        if let Some(frozen) = store.get_mut(victim) {
            frozen.remaining_turns = i32::max(frozen.remaining_turns, turns);
            return false
        } else {
            let frozen = StatusIsFrozen{remaining_turns: turns};
            store.insert(victim, frozen)
                .expect("Unable to insert StatusIsFrozen component.");
            return true
        }
    }

    pub fn tick(
        store: &mut WriteStorage<StatusIsFrozen>,
        immune: &mut WriteStorage<StatusIsImmuneToChill>,
        log: &mut GameLog,
        entity: Entity,
        msg: Option<String>)
    {
        let frozen = store.get_mut(entity);
        let is_immune = immune.get(entity).is_some();
        if let Some(frozen) = frozen {
            if frozen.remaining_turns <= 0 || is_immune {
                store.remove(entity);
                if let Some(msg) = msg {
                    log.entries.push(msg);
                }
            } else {
                frozen.remaining_turns -= 1;
            }
        }
    }
}

// Component indicating the entity is burning.
#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsBurning {
    pub remaining_turns: i32,
    pub tick_damage: i32,
}
impl StatusIsBurning {
    // Attempt to add the burning status to an enetity and return a boolean
    // indicating if this was successful.
    pub fn new_status(
        store: &mut WriteStorage<StatusIsBurning>,
        immune: &ReadStorage<StatusIsImmuneToFire>,
        victim: Entity,
        turns: i32,
        dmg: i32) -> bool
    {
        if immune.get(victim).is_some() {
            return false
        }
        if let Some(burning) = store.get_mut(victim) {
            burning.remaining_turns = i32::max(burning.remaining_turns, turns);
            burning.tick_damage = i32::max(dmg, burning.tick_damage);
            return false
        } else {
            let burning = StatusIsBurning{remaining_turns: turns, tick_damage: dmg};
            store.insert(victim, burning)
                .expect("Unable to insert StatusIsBurning component.");
            return true
        }
    }
    pub fn tick(
        store: &mut WriteStorage<StatusIsBurning>,
        immune: &mut WriteStorage<StatusIsImmuneToFire>,
        log: &mut GameLog,
        entity: Entity,
        msg: Option<String>)
    {
        let burning = store.get_mut(entity);
        let is_immune = immune.get(entity).is_some();
        if let Some(burning) = burning {
            if burning.remaining_turns <= 0 || is_immune {
                store.remove(entity);
                if let Some(msg) = msg {
                    log.entries.push(msg);
                }
            } else {
                burning.remaining_turns -= 1;
            }
        }
    }
}

// Component indicating the entity is immune to damage from Fire elemntal
// sources.
#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsImmuneToFire {
    pub remaining_turns: i32,
}
impl StatusIsImmuneToFire {
    pub fn new_status(store: &mut WriteStorage<StatusIsImmuneToFire>, e: Entity, turns: i32) {
        if let Some(immune) = store.get_mut(e) {
            immune.remaining_turns = i32::max(immune.remaining_turns, turns);
        } else {
            let immune = StatusIsImmuneToFire{remaining_turns: turns};
            store.insert(e, immune)
                .expect("Unable to insert StatusIsImmuneToFire component.");
        }
    }
    pub fn tick(
        store: &mut WriteStorage<StatusIsImmuneToFire>,
        log: &mut GameLog,
        entity: Entity,
        msg: Option<String>)
    {
        let is_immune = store.get_mut(entity);
        if let Some(is_immune) = is_immune {
            if is_immune.remaining_turns <= 0 {
                store.remove(entity);
                if let Some(msg) = msg {
                    log.entries.push(msg);
                }
            } else {
                is_immune.remaining_turns -= 1;
            }
        }
    }
}

// Component indicating the entity is immune to damage from Chill elemntal
// sources.
#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsImmuneToChill {
    pub remaining_turns: i32,
}
impl StatusIsImmuneToChill {
    pub fn new_status(store: &mut WriteStorage<StatusIsImmuneToChill>, e: Entity, turns: i32) {
        if let Some(immune) = store.get_mut(e) {
            immune.remaining_turns = i32::max(immune.remaining_turns, turns);
        } else {
            let immune = StatusIsImmuneToChill{remaining_turns: turns};
            store.insert(e, immune)
                .expect("Unable to insert StatusIsImmuneToChill component.");
        }
    }
    pub fn tick(
        store: &mut WriteStorage<StatusIsImmuneToChill>,
        log: &mut GameLog,
        entity: Entity,
        msg: Option<String>)
    {
        let is_immune = store.get_mut(entity);
        if let Some(is_immune) = is_immune {
            if is_immune.remaining_turns <= 0 {
                store.remove(entity);
                if let Some(msg) = msg {
                    log.entries.push(msg);
                }
            } else {
                is_immune.remaining_turns -= 1;
            }
        }
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

// The entiity has requested to move to a specific map position.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToMoveToPosition {
    pub pt: Point
}

// The entiity has requested to teleport to a random map position.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToMoveToRandomPosition {}

// Signals that the entity has damage queued, but not applied.
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ElementalDamageKind {
    Physical, Fire, Chill, Hunger
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

// An entity with this component has dissipated and should be removed from the
// ECS at the end of the current turn.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToDissipate {}

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