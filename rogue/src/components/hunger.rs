use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};

//------------------------------------------------------------------
// Components and data structures for the hunger system.
//
// To incentivize that the player keep pace moving through the game, we have a
// hunger system, which will inflict some punishment for dilly-dallying. The
// player will progress through various hunger states as game turns pass, with
// progressive states inflicing worse effects on the player.
//------------------------------------------------------------------

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum HungerState {
    WellFed,
    Normal,
    // When Hungry the player cannot wait heal.
    Hungry,
    // When Starving, the player takes a small amount of damage each tick.
    Starving
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    // How many turns does each state last?
    pub state_duration: i32,
    // How many turns into the current state is the player.
    pub time: i32,
    // How many damage does that player take each tick when in Starving state.
    pub tick_damage: i32
}
impl HungerClock {
    // Restore the player to WellFed.
    pub fn satiate(&mut self) {
        self.state = HungerState::WellFed;
        self.time = self.state_duration;
    }
}
