use specs::prelude::*;
use super::{
    Map, TileType, CombatStats, WantsToMeleeAttack, Name, WantsToTakeDamage,
    GameLog, Renderable, Position, AnimationRequestBuffer, AnimationRequest,
    Equipped, GrantsMeleeAttackBonus, StatusIsMeleeAttackBuffed,
    ElementalDamageKind, SpawnEntityWhenMeleeAttacked, EntitySpawnKind,
    EntitySpawnRequestBuffer, EntitySpawnRequest
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
// TODO: Move this into a struct instead of a tuple.
impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, AnimationRequestBuffer>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, GrantsMeleeAttackBonus>,
        ReadStorage<'a, StatusIsMeleeAttackBuffed>,
        WriteStorage<'a, WantsToMeleeAttack>,
        WriteStorage<'a, WantsToTakeDamage>,
        ReadStorage<'a, SpawnEntityWhenMeleeAttacked>,
        WriteExpect<'a, EntitySpawnRequestBuffer>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut map,
            mut log,
            mut animation_builder,
            names,
            combat_stats,
            positions,
            renderables,
            equipped,
            weapon_attack_bonuses,
            is_melee_buffs,
            mut melee_attacks,
            mut damagees,
            spawn_when_melee,
            mut entity_spawn_buffer
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
                    continue;
                }

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

                // If entity splits or spawn on a melee attack, send the signal
                // to spawn a new entity. We're probably smacking a jelly here.
                let spawns = spawn_when_melee.get(target);
                if let (Some(spawns), Some(pos)) = (spawns, pos) {
                    match spawns.kind {
                        EntitySpawnKind::PinkJelly {..} => {
                            let spawn_position = map.random_adjacent_unblocked_point(pos.x, pos.y);
                            // TODO: Guard against spawning in the player position.
                            if let Some(spawn_position) = spawn_position {
                                entity_spawn_buffer.request(EntitySpawnRequest {
                                    x: spawn_position.0,
                                    y: spawn_position.1,
                                    kind: EntitySpawnKind::PinkJelly {max_hp: 1, hp: 1}
                                })
                            }
                        }
                        _ => {}
                    }
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
        melee_attacks.clear();
    }
}