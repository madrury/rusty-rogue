use specs::prelude::*;
use super::{CombatStats, WantsToMeleeAttack, Name, ApplyMeleeDamage, GameLog};

pub struct MeleeCombatSystem {}

// A system for processing melee combat.
//
// This system scans all entities for a WantsToMelee component, which signals
// that the entity has requested to enter melee combat against some target.
// If the combat is successful, we attaach a SufferDamage component to the
// target, which is processed by the DamageSystem.
impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, WantsToMeleeAttack>,
        WriteStorage<'a, ApplyMeleeDamage>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, names, combat_stats, mut melee_attack, mut damagees) = data;
        let iter = (&entities, &melee_attack, &names, &combat_stats).join();
        for (entity, melee, name, stats) in iter {
            let target = melee.target;
            // As a rule, entities cannot target themselves in melee combat.
            // This happens if, for example, the player passes a turn.
            if entity == target {continue;}
            let target_stats = combat_stats.get(target).unwrap();
            if target_stats.hp > 0 {
                let target_name = names.get(target).unwrap();
                let damage = i32::max(0, stats.power - target_stats.defense);
                if damage == 0 {
                    // println!("{} is unable to damage {}.", &name.name, &target_name.name);
                    log.entries.push(
                        format!("{} is unable to damage {}.", &name.name, &target_name.name)
                    );
                } else {
                    log.entries.push(
                        format!("{} hits {} for {} hp.", &name.name, &target_name.name, damage)
                    );
                    ApplyMeleeDamage::new_damage(&mut damagees, melee.target, damage);
                }
            }
        }
        melee_attack.clear();
    }
}