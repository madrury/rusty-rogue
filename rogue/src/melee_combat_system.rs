use specs::prelude::*;
use crate::MeeleAttackRequestBuffer;

use super::{
    Map, Point, TileType, CombatStats, WantsToTakeDamage, Renderable, Position,
    AnimationRequestBuffer, AnimationRequest, Equipped, MeeleAttackWepon,
    StatusIsMeleeAttackBuffed, ElementalDamageKind,
    SpawnEntityWhenMeleeAttacked, EntitySpawnKind, EntitySpawnRequestBuffer,
    EntitySpawnRequest
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
        WriteExpect<'a, AnimationRequestBuffer>,
        WriteExpect<'a, MeeleAttackRequestBuffer>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeeleAttackWepon>,
        ReadStorage<'a, StatusIsMeleeAttackBuffed>,
        WriteStorage<'a, WantsToTakeDamage>,
        ReadStorage<'a, SpawnEntityWhenMeleeAttacked>,
        WriteExpect<'a, EntitySpawnRequestBuffer>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut map,
            mut animation_buffer,
            mut meele_buffer,
            ppos,
            combat_stats,
            positions,
            renderables,
            equipped,
            weapon_attack_bonuses,
            is_melee_buffs,
            mut damagees,
            spawn_when_melee,
            mut entity_spawn_buffer
        ) = data;

        while !meele_buffer.is_empty() {
            let attack = meele_buffer.pop().expect("Meele attack buffer pop failed.");
            if attack.source == attack.target {continue;}

            let target_stats = combat_stats.get(attack.target);
            if target_stats.is_none() { continue; }

            let tstats = target_stats.expect("Failed to unwrap combat stats for meele target.");
            if tstats.hp <= 0 { continue; }

            let source_stats = combat_stats.get(attack.source);
            let sstats = source_stats.expect("Failed to unwrap combat stats for meele target.");

            // Add any modifiers to the meele attack. As of now, there are two sources:
            //   - Weapons add a fixed additive amount to meele attack damage.
            //   - Buffs double attack damage.
            let weapon_attack_bonus: i32 = (&entities, &weapon_attack_bonuses, &equipped)
                .join()
                .filter(|(_e, _ab, eq)| eq.owner == attack.source)
                .map(|(_e, ab, _eq)| ab.bonus)
                .sum();
            let attack_buff_factor: i32 = is_melee_buffs.get(attack.source)
                .map_or(1, |_b| 2);
            let critical_hit_factor: i32 = if attack.critical {2} else {1};
            let damage = i32::max(0, critical_hit_factor * attack_buff_factor * (sstats.power + weapon_attack_bonus));

            // Actually push the damage :D
            WantsToTakeDamage::new_damage(
                &mut damagees,
                attack.target,
                damage,
                ElementalDamageKind::Physical
            );

            // Animate the damage with a flash
            // TODO: Same here. This should be created after damage is actually created.
            // to avoid triggering animations when all damage is nullified.
            let pos = positions.get(attack.target);
            let render = renderables.get(attack.target);
            if let(Some(pos), Some(render)) = (pos, render) {
                animation_buffer.request(
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
            let spawns = spawn_when_melee.get(attack.target);
            if let (Some(spawns), Some(pos)) = (spawns, pos) {
                let spawn_position = map.random_adjacent_point(pos.x, pos.y);
                let mut can_spawn: bool = false;
                if let Some(spawn_position) = spawn_position {
                    let spawn_idx = map.xy_idx(spawn_position.0, spawn_position.1);
                    let in_player_position = (spawn_position.0 == ppos.x)
                        && (spawn_position.1 == ppos.y);
                    let spawn_request_kind = match spawns.kind {
                        EntitySpawnKind::PinkJelly {..} => {
                            can_spawn = !map.blocked[spawn_idx] && !in_player_position;
                            Some(EntitySpawnKind::PinkJelly {
                                max_hp: tstats.hp / 2,
                                hp: tstats.hp / 2
                            })
                        },
                        EntitySpawnKind::Fire {spread_chance, dissipate_chance} => {
                            can_spawn = true;
                            Some(EntitySpawnKind::Fire {
                                spread_chance, dissipate_chance
                            })
                        },
                        EntitySpawnKind::Chill {spread_chance, dissipate_chance} => {
                            can_spawn = true;
                            Some(EntitySpawnKind::Chill {
                                spread_chance, dissipate_chance
                            })
                        },
                        _ => None,
                    };
                    if let Some(spawn_request_kind) = spawn_request_kind {
                        if can_spawn {
                            entity_spawn_buffer.request(EntitySpawnRequest {
                                x: spawn_position.0,
                                y: spawn_position.1,
                                kind: spawn_request_kind
                            });
                        }
                    }
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
}
