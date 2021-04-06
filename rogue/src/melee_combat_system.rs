use specs::prelude::*;
use super::{
    Map, TileType, CombatStats, WantsToMeleeAttack, Name, WantsToTakeDamage,
    GameLog, Renderable, Position, AnimationRequestBuffer, AnimationRequest,
    Equipped, GrantsMeleeAttackBonus, StatusIsMeleeAttackBuffed,
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
        WriteExpect<'a, AnimationRequestBuffer>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, GrantsMeleeAttackBonus>,
        ReadStorage<'a, StatusIsMeleeAttackBuffed>,
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
            weapon_attack_bonuses,
            is_melee_buffs,
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
            // TODO: this unwrap is dodgy. Can we really not get here if the
            // target does not have combat stats? If we have a blocking entity
            // without combat stats, are we gonna be ok here?
            let target_stats = combat_stats.get(target).unwrap();
            if target_stats.hp > 0 {
                let target_name = names.get(target).unwrap();
                let weapon_attack_bonus: i32 = (&entities, &weapon_attack_bonuses, &equipped)
                    .join()
                    .filter(|(_e, _ab, eq)| eq.owner == attacker)
                    .map(|(_e, ab, _eq)| ab.bonus)
                    .sum();
                // Factor is 2 if the attacker is buffed, 1 otherwise.
                let attack_buff_factor: i32 = is_melee_buffs.get(attacker)
                    .map_or(1, |_b| 2);
                let damage = i32::max(0, attack_buff_factor * (stats.power + weapon_attack_bonus));
                // TODO: This message should be created further down the turn
                // pipeline. Probably where damage is actually applied.
                if damage == 0 {
                    log.entries.push(
                        format!("{} is unable to damage {}.", &name.name, &target_name.name)
                    );
                } else {
                    // TODO: This is not right. This message needs to happen AFTER defense buffs are applied.
                    log.entries.push(
                        format!("{} hits {} for {} hp.", &name.name, &target_name.name, damage)
                    );
                    WantsToTakeDamage::new_damage(
                        &mut damagees,
                        melee.target,
                        damage,
                        ElementalDamageKind::Physical
                    );
                    // Animate the damage with a flash
                    // TODO: Same here. This should be created after damage is actually created.
                    // to avoid triggering animations when all damage is nullified.
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
                    }
                    // Create a bloodstain where the damage was inflicted.
                    if let Some(pos) = pos {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.tiles[idx] != TileType::DownStairs {
                            map.tiles[idx] = TileType::BloodStain
                        }
                    }
                }
            }
        }
        melee_attacks.clear();
    }
}