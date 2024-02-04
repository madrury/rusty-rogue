use crate::CombatStats;

use super::GameLog;
use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs_derive::*;


//------------------------------------------------------------------
// Generic functions for creating and ticking status effects.
//
// These functions are generic over types implementing the StatusEffect trait,
// and are used to create new status effects, and tick already existing status
// effects. There are two types;
//
//   - new_status and tick_status are the basic implementations.
//   - new_status_with_immunity and tick_status_with_immunity are used for status
//     for which immunity can be granted, and we need to check if the targeted
//     entity is immune before applying the status.
//   - new_combat_stats_status is used for buffs and debuffs affecting combat
//     statistics, which should only be applied to entities that can engage in
//     combat, i.e., have a CombatStats cmponent.
//------------------------------------------------------------------
pub fn new_status<Status: Component + StatusEffect>(
    store: &mut WriteStorage<Status>,
    e: Entity,
    turns: i32,
    render_glyph: bool
) -> bool {
    if let Some(status) = store.get_mut(e) {
        status.set_remaining_turns(i32::max(status.remaining_turns(), turns));
    } else {
        let status = Status::new(turns, render_glyph);
        store
            .insert(e, status)
            .expect("Unable to insert new status component.");
        return true
    }
    return false
}

pub fn new_status_with_immunity<Status, StatusImmune>(
    store: &mut WriteStorage<Status>,
    immune: &WriteStorage<StatusImmune>,
    e: Entity,
    turns: i32,
    render_glyph: bool
) -> bool where
    Status: Component + StatusEffect,
    StatusImmune: Component + StatusEffect,
{
    if let Some(status) = store.get_mut(e) {
        status.set_remaining_turns(i32::max(status.remaining_turns(), turns));
    } else {
        let is_immune = immune.get(e).is_some();
        if !is_immune {
            let status = Status::new(turns, render_glyph);
            store
                .insert(e, status)
                .expect("Unable to insert new status component.");
            return true
        }
    }
    return false
}

pub fn new_combat_stats_status<Status>(
    store: &mut WriteStorage<Status>,
    stats: &WriteStorage<CombatStats>,
    e: Entity,
    turns: i32,
    render_glyph: bool
) -> bool where
    Status: Component + StatusEffect,
{
    if let Some(status) = store.get_mut(e) {
        status.set_remaining_turns(i32::max(status.remaining_turns(), turns));
    } else {
        let is_immune = stats.get(e).is_none();
        if !is_immune {
            let status = Status::new(turns, render_glyph);
            store
                .insert(e, status)
                .expect("Unable to insert new status component.");
            return true
        }
    }
    return false
}

pub fn tick_status<Status: Component + StatusEffect>(
    store: &mut WriteStorage<Status>,
    log: &mut GameLog,
    entity: Entity,
    msg: Option<String>,
) {
    let status = store.get_mut(entity);
    if let Some(status) = status {
        if status.remaining_turns() <= 0 {
            store.remove(entity);
            if let Some(msg) = msg {
                log.entries.push(msg);
            }
        } else {
            status.set_remaining_turns(status.remaining_turns() - 1);
        }
    }
}

pub fn tick_status_with_immunity<Status, StatusImmune>(
    store: &mut WriteStorage<Status>,
    immune: &WriteStorage<StatusImmune>,
    log: &mut GameLog,
    entity: Entity,
    msg: Option<String>,
) where
    Status: Component + StatusEffect,
    StatusImmune: Component + StatusEffect,
{
    let status = store.get_mut(entity);
    let is_immune = immune.get(entity).is_some();
    if let Some(status) = status {
        if status.remaining_turns() <= 0 || is_immune {
            store.remove(entity);
            if let Some(msg) = msg {
                log.entries.push(msg);
            }
        } else {
            status.set_remaining_turns(status.remaining_turns() - 1);
        }
    }
}

pub fn remove_status<Status: Component + StatusEffect>(
    store: &mut WriteStorage<Status>,
    entity: Entity,
) {
    let status = store.get_mut(entity);
    if let Some(_) = status {
        store.remove(entity);
    }
}


//------------------------------------------------------------------
// Status Effect Components.
//------------------------------------------------------------------
pub trait StatusEffect {
    fn new(turns: i32, render_glyph: bool) -> Self;
    fn remaining_turns(&self) -> i32;
    fn set_remaining_turns(&mut self, turns: i32);
    fn do_render(&self) -> bool;
}

// Component indicating the entity is frozen. A frozen entity cannot take any
// action until the status has expired, or is removed.
#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsFrozen {
    pub remaining_turns: i32,
    pub render_glyph: bool
}
impl StatusEffect for StatusIsFrozen {
    fn new(turns: i32, render_glyph: bool) -> StatusIsFrozen {
        StatusIsFrozen {
            remaining_turns: turns,
            render_glyph: render_glyph
        }
    }
    fn remaining_turns(&self) -> i32 {
        self.remaining_turns
    }
    fn set_remaining_turns(&mut self, turns: i32) {
        self.remaining_turns = turns
    }
    fn do_render(&self) -> bool {
        self.render_glyph
    }
}

// Component indicating the entity is burning. A burning entity takes a small
// amount of damage each turn.
pub const BURNING_TICK_DAMAGE: i32 = 3;

#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsBurning {
    pub remaining_turns: i32,
    pub render_glyph: bool
}
impl StatusEffect for StatusIsBurning {
    fn new(turns: i32, render_glyph: bool) -> StatusIsBurning {
        StatusIsBurning {
            remaining_turns: turns,
            render_glyph: render_glyph
        }
    }
    fn remaining_turns(&self) -> i32 {
        self.remaining_turns
    }
    fn set_remaining_turns(&mut self, turns: i32) {
        self.remaining_turns = turns
    }
    fn do_render(&self) -> bool {
        self.render_glyph
    }
}

// Component indicating that the entity's melee attacks are buffed (do double
// damage).
#[derive(Component, ConvertSaveload, Clone)]

pub struct StatusIsMeleeAttackBuffed {
    pub remaining_turns: i32,
    pub render_glyph: bool
}
impl StatusEffect for StatusIsMeleeAttackBuffed {
    fn new(turns: i32, render_glyph: bool) -> StatusIsMeleeAttackBuffed {
        StatusIsMeleeAttackBuffed {
            remaining_turns: turns,
            render_glyph: render_glyph
        }
    }
    fn remaining_turns(&self) -> i32 {
        self.remaining_turns
    }
    fn set_remaining_turns(&mut self, turns: i32) {
        self.remaining_turns = turns
    }
    fn do_render(&self) -> bool {
        self.render_glyph
    }
}

// Component indicating that the entity's pysical defense is (damage is halved)/
#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsPhysicalDefenseBuffed {
    pub remaining_turns: i32,
    pub render_glyph: bool
}
impl StatusEffect for StatusIsPhysicalDefenseBuffed {
    fn new(turns: i32, render_glyph: bool) -> StatusIsPhysicalDefenseBuffed {
        StatusIsPhysicalDefenseBuffed {
            remaining_turns: turns,
            render_glyph: render_glyph
        }
    }
    fn remaining_turns(&self) -> i32 {
        self.remaining_turns
    }
    fn set_remaining_turns(&mut self, turns: i32) {
        self.remaining_turns = turns
    }
    fn do_render(&self) -> bool {
        self.render_glyph
    }
}

//------------------------------------------------------------------
// Status Immunity Components.
//------------------------------------------------------------------
// Component indicating the entity is immune to damage from Fire elemntal
// sources.
#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsImmuneToFire {
    pub remaining_turns: i32,
    pub render_glyph: bool
}
impl StatusEffect for StatusIsImmuneToFire {
    fn new(turns: i32, render_glyph: bool) -> StatusIsImmuneToFire {
        StatusIsImmuneToFire {
            remaining_turns: turns,
            render_glyph: render_glyph
        }
    }
    fn remaining_turns(&self) -> i32 {
        self.remaining_turns
    }
    fn set_remaining_turns(&mut self, turns: i32) {
        self.remaining_turns = turns
    }
    fn do_render(&self) -> bool {
        self.render_glyph
    }
}

// Component indicating the entity is immune to damage from Chill elemntal
// sources.
#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusIsImmuneToChill {
    pub remaining_turns: i32,
    pub render_glyph: bool
}
impl StatusEffect for StatusIsImmuneToChill {
    fn new(turns: i32, render_glyph: bool) -> StatusIsImmuneToChill {
        StatusIsImmuneToChill {
            remaining_turns: turns,
            render_glyph: render_glyph
        }
    }
    fn remaining_turns(&self) -> i32 {
        self.remaining_turns
    }
    fn set_remaining_turns(&mut self, turns: i32) {
        self.remaining_turns = turns
    }
    fn do_render(&self) -> bool {
        self.render_glyph
    }
}