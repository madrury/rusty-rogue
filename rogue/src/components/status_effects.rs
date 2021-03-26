use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use super::{GameLog};

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