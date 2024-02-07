
use specs::prelude::*;
use super::{
    SwimStamina, RunState, Map, Position, WantsToTakeDamage, GameLog,
    ElementalDamageKind
};


pub struct SwimmingSystem {}

#[derive(SystemData)]
pub struct SwimmingSystemData<'a> {
    entities: Entities<'a>,
    player: ReadExpect<'a, Entity>,
    runstate: ReadExpect<'a, RunState>,
    map: ReadExpect<'a, Map>,
    log: WriteExpect<'a, GameLog>,
    positions: WriteStorage<'a, Position>,
    wants_damages: WriteStorage<'a, WantsToTakeDamage>,
    swim_staminas: WriteStorage<'a, SwimStamina>,
}

//----------------------------------------------------------------------------
// Tick the player's swim stamina, and apply drowning damage if needed.
//----------------------------------------------------------------------------
impl<'a> System<'a> for SwimmingSystem {

    type SystemData = SwimmingSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let SwimmingSystemData {
            entities, player, runstate, map, mut log, positions, mut wants_damages, mut swim_staminas
        } = data;

        for (entity, pos, stamina) in (&entities, &positions, &mut swim_staminas).join() {

            let proceed =
                (*runstate == RunState::PlayerTurn && entity == *player)
                || (*runstate == RunState::MonsterTurn && entity != *player);
            if !proceed {
                return
            }

            let idx = map.xy_idx(pos.x, pos.y);
            let in_water = map.water[idx];

            let log_message: Option<String>;
            if in_water {
                if stamina.stamina >= 1 {
                    log_message = None;
                    stamina.stamina -= 1;
                } else {
                    log_message = Some("You are drowning.".to_string());
                    WantsToTakeDamage::new_damage(
                        &mut wants_damages,
                        entity,
                        stamina.drowning_damage,
                        ElementalDamageKind::Drowning
                    );
                }
            } else {
                log_message = None;
                stamina.stamina = i32::min(stamina.stamina + 1, stamina.max_stamina);
            }

            if let Some(msg) = log_message {
                log.entries.push(msg);
            }
        }
    }
}