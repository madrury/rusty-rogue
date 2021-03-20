use specs::prelude::*;
use super::{
    Map, TileType, CombatStats, WantsToMeleeAttack, Name, WantsToTakeDamage,
    GameLog, Renderable, Position, AnimationBuilder, AnimationRequest,
    Equipped, GrantsMeleeAttackBonus,
    ElementalDamageKind
};

pub struct MeleeCombatSystem {}

//----------------------------------------------------------------------------
// A system for processing melee combat.
//
// This system scans all entities for a WantsToMeleeAttack component, which
// signals that the entity has requested to enter melee combat against some
// target. If the combat is successful, we attaach a SufferDamage component to
// the target, which is processed by the DamageSystem.
//----------------------------------------------------------------------------
impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, AnimationBuilder>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, GrantsMeleeAttackBonus>,
        WriteStorage<'a, WantsToMeleeAttack>,
        WriteStorage<'a, WantsToTakeDamage>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut map,
            mut log,
            mut animation_builder,
            names,
            combat_stats,
            equipped,
            attack_bonuses,
            mut melee_attacks,
            mut damagees,
            positions,
            renderables
        ) = data;

        let iter = (&entities, &melee_attacks, &names, &combat_stats).join();
        for (attacker, melee, name, stats) in iter {
            let target = melee.target;
            // As a rule, entities cannot target themselves in melee combat.
            // This happens if, for example, the player passes a turn.
            if attacker == target {continue;}
            let target_stats = combat_stats.get(target).unwrap();
            if target_stats.hp > 0 {

                let target_name = names.get(target).unwrap();
                let attack_bonus: i32 = (&entities, &attack_bonuses, &equipped)
                    .join()
                    .filter(|(_e, _ab, eq)| eq.owner == attacker)
                    .map(|(_e, ab, _eq)| ab.bonus)
                    .sum();
                let damage = i32::max(0, stats.power + attack_bonus);
                if damage == 0 {
                    log.entries.push(
                        format!("{} is unable to damage {}.", &name.name, &target_name.name)
                    );
                } else {
                    log.entries.push(
                        format!("{} hits {} for {} hp.", &name.name, &target_name.name, damage)
                    );
                    WantsToTakeDamage::new_damage(
                        &mut damagees,
                        melee.target,
                        damage,
                        ElementalDamageKind::Physical
                    );
                    // Animate the damage with a flash, and render a bloodstain
                    // where the damage was inflicted.
                    let pos = positions.get(melee.target);
                    let render = renderables.get(melee.target);
                    if let(Some(pos), Some(render)) = (pos, render) {
                        animation_builder.request(
                            AnimationRequest::MeleeAttack {
                                x: pos.x,
                                y: pos.y,
                                bg: render.bg,
                                glyph: render.glyph,
                            }
                        );
                        let idx = map.xy_idx(pos.x, pos.y);
                        map.tiles[idx] = TileType::BloodStain
                    }
                }
            }
        }
        melee_attacks.clear();
    }
}