
use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};

//------------------------------------------------------------------
// Components and data structures for the magic system
//
// Magic uses a spell charge system: an entity that knows a spell has some
// number of charges of the spell to extend (each charge grants the ability to
// cast the spell one time). Charges are replenished over time, with a timer
// ticking each game turn.
//------------------------------------------------------------------

// Tags a spell component as in the spellbook of some other entity. I.e., the
// referenced entity can cast the spell.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InSpellBook {
    pub owner: Entity
}

// Tracks teh number of charges of a spell that are available, and how far we
// are into recharging a use of that spell.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpellCharges {
    // The maximum number of charges of this spell that can be available.
    pub max_charges: i32,
    // The current number of available charges of this spell.
    pub charges: i32,
    // The number of turns to recharge one use of this spell.
    pub regen_time: i32,
    // The number of turns we are into recharging another use of this spell. The
    // spell_charge_system is responsible for incrementing this each turn.
    pub time: i32
}
impl SpellCharges {
    pub fn expend_charge(&mut self) {
        self.charges = i32::max(0, self.charges - 1);
        self.time = 0;
    }
    // Returns value indicating if a cast has recharged.
    pub fn tick(&mut self) -> bool {
        self.time = i32::min(self.time + 1, self.regen_time);
        if self.time == self.regen_time && self.charges < self.max_charges {
            self.charges += 1;
            self.time = 0;
            return true
        }
        // We don't want to let the time fill when we're at max charges, since
        // then we could spam spells at max charge each turn., and the spell
        // would recharge the next turn.
        if self.charges == self.max_charges {
            self.time = 0;
        }
        false
    }
}