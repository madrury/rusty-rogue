
use specs::prelude::*;
use super::{GameLog, RunState, SpellCharges, Name, InSpellBook};


pub struct SpellChargeSystem {}

#[derive(SystemData)]
pub struct SpellChargeSystemData<'a> {
    entities: Entities<'a>,
    player: ReadExpect<'a, Entity>,
    runstate: ReadExpect<'a, RunState>,
    log: WriteExpect<'a, GameLog>,
    names: ReadStorage<'a, Name>,
    in_spellbooks: ReadStorage<'a, InSpellBook>,
    spell_charges: WriteStorage<'a, SpellCharges>
}

impl<'a> System<'a> for SpellChargeSystem {

    type SystemData = SpellChargeSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let SpellChargeSystemData {
            entities, player, runstate, mut log, names, in_spellbooks, mut spell_charges
        } = data;


        for (spell, in_spellbook, charges) in (&entities, &in_spellbooks, &mut spell_charges).join() {

            // Only tick the player's spells on their turn and any monster's
            // spells on monster's turns.
            let spellcaster = in_spellbook.owner;
            let proceed =
                (*runstate == RunState::PlayerTurn && spellcaster == *player)
                || (*runstate == RunState::MonsterTurn && spellcaster != *player);
            if !proceed {
                return
            }

            // Tick the timer and bail out if there is nothing else to do.
            let did_recharge = charges.tick();
            if did_recharge && spellcaster == *player {
                let spell_name = names.get(spell);
                if let Some(spell_name) = spell_name {
                    log.entries.push(format!(
                        "Player's {} recharged.", spell_name.name
                    ))
                }
            }
        }
    }
}
