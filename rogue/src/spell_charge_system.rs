
use specs::prelude::*;
use crate::{ElementalDamageKind, IncresesSpellRechargeRateWhenEquipped};

use super::{GameLog, SpellCharges, Name, InSpellBook, Equipped};


pub struct SpellChargeSystem {}

#[derive(SystemData)]
pub struct SpellChargeSystemData<'a> {
    entities: Entities<'a>,
    player: ReadExpect<'a, Entity>,
    log: WriteExpect<'a, GameLog>,
    names: ReadStorage<'a, Name>,
    in_spellbooks: ReadStorage<'a, InSpellBook>,
    spell_charges: WriteStorage<'a, SpellCharges>,
    equipped: ReadStorage<'a, Equipped>,
    increases_spell_recharge_rate: ReadStorage<'a, IncresesSpellRechargeRateWhenEquipped>,
}

impl<'a> System<'a> for SpellChargeSystem {

    type SystemData = SpellChargeSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let SpellChargeSystemData {
            entities,
            player,
            mut log,
            names,
            in_spellbooks,
            mut spell_charges,
            equipped,
            increases_spell_recharge_rate
        } = data;

        for (spell, in_spellbook, charges) in (&entities, &in_spellbooks, &mut spell_charges).join() {
            let spellowner = in_spellbook.owner;
            // See if any of the equipped weapons modify the spell recharge
            // rate.
            let charge_bonus_element = (&equipped, &increases_spell_recharge_rate).join()
                .filter(|(eq, _)| eq.owner == spellowner)
                .map(|(_, inc)| inc.element)
                .next()
                .unwrap_or(ElementalDamageKind::None);
            // Tick the timer and bail out if there is nothing else to do.
            let mut did_recharge = charges.tick();
            // Tick the timer again if an equipped weapon grants a bonus
            // charge.
            did_recharge |= if charge_bonus_element == charges.element {
                charges.tick()
            } else {
                false
            };
            // If any of our ticks recharged a spell, log a message.
            if did_recharge && spellowner == *player {
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
