
use specs::prelude::*;
use super::{HungerClock, RunState, HungerState, WantsToTakeDamage, GameLog};


pub struct HungerSystem {}

#[derive(SystemData)]
pub struct HungerSystemData<'a> {
    entities: Entities<'a>,
    player: ReadExpect<'a, Entity>,
    runstate: ReadExpect<'a, RunState>,
    log: WriteExpect<'a, GameLog>,
    wants_damages: WriteStorage<'a, WantsToTakeDamage>,
    hungers: WriteStorage<'a, HungerClock>,
}

impl<'a> System<'a> for HungerSystem {

    type SystemData = HungerSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let HungerSystemData {
            entities, player, runstate, mut log, mut wants_damages, mut hungers
        } = data;

        for (entity, mut clock) in (&entities, &mut hungers).join() {

            // Only tick the player's clock on their turn and any monster clock
            // on monster's turns.
            let proceed =
                (*runstate == RunState::PlayerTurn && entity == *player)
                || (*runstate == RunState::MonsterTurn && entity != *player);
            if !proceed {
                return
            }

            // Tick the timer and bail out if there is nothing else to do.
            clock.time -= 1;
            if clock.time > 0 {
                return
            }

            // Bump hunger state up by one unless we're starving, in which case
            // take damage.
            let log_message: Option<String>;
            match clock.state {
                HungerState::WellFed => {
                    clock.state = HungerState::Normal;
                    clock.time = clock.state_duration;
                    log_message = None;
                },
                HungerState::Normal => {
                    clock.state = HungerState::Hungry;
                    clock.time = clock.state_duration;
                    log_message = Some("You begin to feel hungry.".to_string());
                },
                HungerState::Hungry => {
                    clock.state = HungerState::Starving;
                    log_message = Some("You are starving.".to_string());
                }
                HungerState::Starving => {
                    WantsToTakeDamage::new_damage(&mut wants_damages, entity, clock.tick_damage);
                    log_message = None;
                }
            }

            if let Some(msg) = log_message {
                log.entries.push(msg);
            }
        }
    }
}
