
use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};

use crate::ElementalDamageKind;

//------------------------------------------------------------------
// Components and data structures for the magic system
//
// Magic uses a spell charge system: an entity that knows a spell has some
// number of charges of the spell to extend (each charge grants the ability to
// cast the spell one time). Charges are replenished over time, with a timer
// ticking each game turn.
//------------------------------------------------------------------

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlessingOrb {}

const ORBS_NEEDED_FOR_BLESSING: i32 = 4;
#[derive(Component, ConvertSaveload, Clone)]
pub struct BlessingOrbBag {
    pub count: i32
}
impl BlessingOrbBag {
    pub fn enough_orbs_for_blessing(&self) -> bool {
        self.count >= ORBS_NEEDED_FOR_BLESSING
    }
    pub fn cash_in_orbs_for_blessing(&mut self) {
        self.count = i32::max(0, self.count - ORBS_NEEDED_FOR_BLESSING);
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlessingSelectionTile {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct OfferedBlessing {}


#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum BlessingSlot {
    None,
    Movement,
    Assist,
    NonElementalAttack,
    FireAttackLevel1,
    FireAttackLevel2,
    ChillAttackLevel1,
    ChillAttackLevel2
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Castable {
   pub slot: BlessingSlot
}

// Tags a spell component as in the spellbook of some other entity. I.e., the
// referenced entity can cast the spell.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InSpellBook {
    pub owner: Entity,
    pub slot: BlessingSlot
}

// Tracks teh number of charges of a spell that are available, and how far we
// are into recharging a use of that spell.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpellCharges {
    // The maximum number of charges of this spell that can be available.
    pub max_charges: i32,
    // The current number of available charges of this spell.
    pub charges: i32,
    // The number of turns to recharge one use of this spell, assuming the base
    // recharge rate of one tick per turn.
    pub regen_ticks: i32,
    // The number of turns we are into recharging another use of this spell. The
    // spell_charge_system is responsible for incrementing this each turn.
    pub ticks: i32,
    // Optional element. Used to modify the recharge rate under certain circumstances.
    pub element: ElementalDamageKind
}
impl SpellCharges {
    pub fn expend_charge(&mut self) {
        self.charges = i32::max(0, self.charges - 1);
        self.ticks = 0;
    }
    // Returns value indicating if a cast has recharged.
    pub fn tick(&mut self) -> bool {
        self.ticks = i32::min(self.ticks + 1, self.regen_ticks);
        if self.ticks == self.regen_ticks && self.charges < self.max_charges {
            self.charges += 1;
            self.ticks = 0;
            return true
        }
        // We don't want to let the time fill when we're at max charges, since
        // then we could spam spells at max charge each turn., and the spell
        // would recharge the next turn.
        if self.charges == self.max_charges {
            self.ticks = 0;
        }
        false
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct IncresesSpellRechargeRate {
    pub element: ElementalDamageKind
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SingleCast {}