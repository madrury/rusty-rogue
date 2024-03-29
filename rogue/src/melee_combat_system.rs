use specs::prelude::*;
use crate::{
    AnimationSequenceBuffer, AnimationBlock, EntitySpawnRequestBuffer,
    EntitySpawnRequest, MeeleAttackRequestBuffer, Map, Point, TileType
};
use crate::components::*;
use crate::components::equipment::*;
use crate::components::melee::*;
use crate::components::signaling::*;
use crate::components::spawn_despawn::*;
use crate::components::status_effects::*;


//----------------------------------------------------------------------------
// A system for processing melee combat.
//
// Processes the ECS MeeleAttackRequestBuffer.
//----------------------------------------------------------------------------
pub struct MeleeCombatSystem {}

// TODO: Move this into a struct instead of a tuple.
impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, AnimationSequenceBuffer>,
        WriteExpect<'a, MeeleAttackRequestBuffer>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeeleAttackWepon>,
        ReadStorage<'a, StatusIsMeleeAttackBuffed>,
        WriteStorage<'a, WantsToTakeDamage>,
        WriteStorage<'a, Bloodied>,
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
            mut damages,
            mut bloodieds,
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
                &mut damages,
                attack.target,
                damage,
                attack.element
            );

            // Animate the damage with a flash
            // TODO: Same here. This should be created after damage is actually created.
            // to avoid triggering animations when all damage is nullified.
            let targetpos = positions.get(attack.target);
            let targetrender = renderables.get(attack.target);
            if let(Some(pos), Some(render)) = (targetpos, targetrender) {
                animation_buffer.request_block(
                    AnimationBlock::MeleeAttack {
                        pt: pos.to_point(),
                        bg: render.bg,
                        glyph: render.glyph,
                    }
                );
            }

            // 🩸 🩸 🩸 🩸
            if let Some(pos) = targetpos {
                let idx = map.xy_idx(pos.x, pos.y);
                map.tiles[idx] = TileType::BloodStain;
                for &e in map.tile_content[idx].iter() {
                    bloodieds.insert(e, Bloodied {})
                        .expect("Failed to insert Bloodied component.");
                }
            }

            // If entity splits or spawn on a melee attack, send the signal
            // to spawn a new entity. We're probably smacking a jelly here.
            let spawns = spawn_when_melee.get(attack.target);
            if let (Some(spawns), Some(pos)) = (spawns, targetpos) {
                let spawn_position = map.random_adjacent_point(pos.x, pos.y);
                let mut can_spawn: bool = false;
                if let Some(spawn_position) = spawn_position {
                    let spawn_idx = map.xy_idx(spawn_position.x, spawn_position.y);
                    let in_player_position = (spawn_position.x == ppos.x)
                        && (spawn_position.y == ppos.y);
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
                                x: spawn_position.x,
                                y: spawn_position.y,
                                kind: spawn_request_kind
                            });
                        }
                    }
                }
            } // Swan Requests.
        } // Meele Buffer.
    }
}
