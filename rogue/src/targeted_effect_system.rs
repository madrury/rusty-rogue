use log::{info, warn};
use specs::prelude::*;

use crate::{
    Map, Point, GameLog, AnimationSequenceBuffer, AnimationBlock,
    EntitySpawnRequestBuffer, EntitySpawnRequest, new_status_with_immunity,
    new_combat_stats_status
};
use crate::components::*;
use crate::components::animation::*;
use crate::components::game_effects::*;
use crate::components::magic::*;
use crate::components::melee::*;
use crate::components::spawn_despawn::*;
use crate::components::signaling::*;
use crate::components::status_effects::*;
use crate::components::targeting::*;


pub struct TargetedSystem {}

// Searches for WantsToUseTargeted compoents and processes the results by:
//   - Finding targets in the selected position, or the area of effect.
//   - Looking for vatious effect encoding components on the thing.
#[derive(SystemData)]
pub struct TargetedSystemData<'a> {
        entities: Entities<'a>,
        map: ReadExpect<'a, Map>,
        log: WriteExpect<'a, GameLog>,
        animation_buffer: WriteExpect<'a, AnimationSequenceBuffer>,
        spawn_buffer: WriteExpect<'a, EntitySpawnRequestBuffer>,
        names: ReadStorage<'a, Name>,
        renderables: ReadStorage<'a, Renderable>,
        positions: WriteStorage<'a, Position>,
        consumables: ReadStorage<'a, Consumable>,
        spell_charges: WriteStorage<'a, SpellCharges>,
        single_casts: WriteStorage<'a, RemovedFromSpellBookWhenCast>,
        in_spellbooks: WriteStorage<'a, InSpellBook>,
        specials: WriteStorage<'a, WeaponSpecial>,
        expend_special_when_thrown: ReadStorage<'a, ExpendWeaponSpecialWhenThrown>,
        expend_special_when_cast: ReadStorage<'a, ExpendWeaponSpecialWhenCast>,
        wants_target: WriteStorage<'a, WantsToUseTargeted>,
        targeted_when_thrown: ReadStorage<'a, TargetedWhenThrown>,
        targeted_when_cast: ReadStorage<'a, TargetedWhenCast>,
        combat_stats: WriteStorage<'a, CombatStats>,
        healing: ReadStorage<'a, ProvidesFullHealing>,
        damage_when_thrown: ReadStorage<'a, InflictsDamageWhenThrown>,
        damage_when_cast: ReadStorage<'a, InflictsDamageWhenCast>,
        freeze_when_thrown: WriteStorage<'a, InflictsFreezingWhenThrown>,
        freeze_when_cast: WriteStorage<'a, InflictsFreezingWhenCast>,
        burning_when_thrown: WriteStorage<'a, InflictsBurningWhenThrown>,
        burning_when_cast: WriteStorage<'a, InflictsBurningWhenCast>,
        buff_attack_when_cast: WriteStorage<'a, BuffsMeleeAttackWhenCast>,
        buff_defense_when_cast: WriteStorage<'a, BuffsPhysicalDefenseWhenCast>,
        move_to_position_when_cast: ReadStorage<'a, MoveToPositionWhenCast>,
        animation_when_thrown: ReadStorage<'a, AnimationWhenThrown>,
        animation_when_cast: ReadStorage<'a, AnimationWhenCast>,
        spawns_entity_in_area_when_thrown: ReadStorage<'a, SpawnsEntityInAreaWhenThrown>,
        spawns_entity_in_area_when_cast: ReadStorage<'a, SpawnsEntityInAreaWhenCast>,
        apply_damages: WriteStorage<'a, WantsToTakeDamage>,
        teleports: ReadStorage<'a, MovesToRandomPosition>,
        wants_to_move: WriteStorage<'a, WantsToMoveToPosition>,
        wants_to_teleport: WriteStorage<'a, WantsToMoveToRandomPosition>,
        is_frozen: WriteStorage<'a, StatusIsFrozen>,
        is_burning: WriteStorage<'a, StatusIsBurning>,
        is_fire_immune: WriteStorage<'a, StatusIsImmuneToFire>,
        is_chill_immune: WriteStorage<'a, StatusIsImmuneToChill>,
        has_buffed_attack: WriteStorage<'a, StatusIsMeleeAttackBuffed>,
        has_buffed_defense: WriteStorage<'a, StatusIsPhysicalDefenseBuffed>,
}

impl<'a> System<'a> for TargetedSystem {

    type SystemData = TargetedSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let TargetedSystemData {
            entities,
            map,
            mut log,
            mut animation_buffer,
            mut spawn_buffer,
            names,
            renderables,
            mut positions,
            consumables,
            mut spell_charges,
            mut single_casts,
            mut in_spellbooks,
            mut specials,
            expend_special_when_thrown,
            expend_special_when_cast,
            targeted_when_thrown,
            targeted_when_cast,
            mut wants_target,
            mut combat_stats,
            healing,
            damage_when_thrown,
            damage_when_cast,
            freeze_when_thrown,
            freeze_when_cast,
            burning_when_thrown,
            burning_when_cast,
            buff_attack_when_cast,
            buff_defense_when_cast,
            move_to_position_when_cast,
            animation_when_thrown,
            animation_when_cast,
            spawns_entity_in_area_when_thrown,
            spawns_entity_in_area_when_cast,
            mut apply_damages,
            teleports,
            mut wants_to_move,
            mut wants_to_teleport,
            mut is_frozen,
            mut is_burning,
            is_fire_immune,
            is_chill_immune,
            mut has_buffed_attack,
            mut has_buffed_defense,
        } = data;

        //--------------------------------------------------------------------
        // Main loop through all the targets.
        //--------------------------------------------------------------------
        for (user, want_target) in (&entities, &wants_target).join() {

            // Determine if we a throwing an item or casting a spell.
            let verb = want_target.verb;

            // In the case we are casting a spell, we check here that the spell
            // is castable.
            let has_charges = spell_charges
                .get(want_target.thing)
                .map_or(true, |sc| sc.charges > 0);
            let is_single_cast = single_casts.get(want_target.thing).is_some();
            if verb == TargetingVerb::Cast && !(has_charges || is_single_cast)  {
                warn!("Attempted to cast an impossible spell.");
                continue
            }

            // Stuff needed to construct log messages.
            let log_thing_name = names.get(want_target.thing);
            let log_verb = match verb {
                TargetingVerb::Throw => "throws",
                TargetingVerb::Cast => "casts",
            };

            // Gather up all the entities that are either at the targeted
            // position or within the area of effect.
            let user_position = positions.get(user).unwrap_or(&Position {x: 0, y: 0});
            let user_point = Point {x: user_position.x, y: user_position.y};
            let target_point = want_target.target;
            let targeting_kind = match verb {
                TargetingVerb::Throw => targeted_when_thrown.get(want_target.thing).map(|t| t.kind),
                TargetingVerb::Cast => targeted_when_cast.get(want_target.thing).map(|t| t.kind),
            };
            if targeting_kind.is_none() {
                info!("Failed to find targeting kind.");
                continue
            }

            let targets: Vec<(&Entity, Point)> = find_targets(
                &*map,
                user_point,
                target_point,
                targeting_kind.unwrap() // Safe because of earlier guard.
            )
                .into_iter()
                .filter(|(&e, _pt)| e != user)
                .collect();

            // Apply the effect to each target in turn. This is essentially a
            // big switch statement over the possible types of effects.
            for (target, target_point) in targets {

                // Component: ProvidesHealing.
                // Note: This component does not yet have a cast and thrown version.
                let thing_heals = healing.get(want_target.thing);
                let stats = combat_stats.get_mut(*target);
                if let (Some(_), Some(stats)) = (thing_heals, stats) {
                    stats.full_heal();
                }

                // Compontnet: MovesToRandomPosition
                // Note: This component does not yet have a cast and thrown version.
                let thing_teleports = teleports.get(want_target.thing);
                let target_pos = positions.get_mut(*target);
                if let (Some(_), Some(_)) = (thing_teleports, target_pos) {
                    wants_to_teleport.insert(*target, WantsToMoveToRandomPosition {})
                        .expect("Failed to insert WantsToMoveToRandomPostion.");
                }

                // Components: InflictsDamageWhen{Thrown|Cast}
                let target_stats = combat_stats.get_mut(*target);
                let damage_data = match verb {
                    TargetingVerb::Throw => damage_when_thrown.get(want_target.thing).map(|d| &d.0),
                    TargetingVerb::Cast => damage_when_cast.get(want_target.thing).map(|d| &d.0),
                };
                if let (Some(dd), Some(_stats)) = (damage_data, target_stats) {
                    WantsToTakeDamage::new_damage(
                        &mut apply_damages,
                        *target, dd.damage, dd.element
                    );
                }

                // Components: InflictsFreezingWhen{Thrown|Cast}
                let freezing_data = match verb {
                    TargetingVerb::Throw => freeze_when_thrown.get(want_target.thing).map(|d| &d.0),
                    TargetingVerb::Cast => freeze_when_cast.get(want_target.thing).map(|d| &d.0),
                };
                if let Some(fd) = freezing_data {
                    new_status_with_immunity::<StatusIsFrozen, StatusIsImmuneToChill>(
                        &mut is_frozen,
                        &is_chill_immune,
                        *target,
                        fd.turns,
                        true
                    );
                }

                // Components: InflictsBurningWhen{Thrown|Cast}
                let burning_data = match verb {
                    TargetingVerb::Throw => burning_when_thrown.get(want_target.thing).map(|d| &d.0),
                    TargetingVerb::Cast => burning_when_cast.get(want_target.thing).map(|d| &d.0),
                };
                if let Some(bd) = burning_data {
                    let _play_message = new_status_with_immunity::<StatusIsBurning, StatusIsImmuneToFire>(
                        &mut is_burning,
                        &is_fire_immune,
                        *target,
                        bd.turns,
                        true,
                    );
                }

                // Component: BuffsMeleeAttackWhenCast
                // Note: This component does not yet have both a thorwn and cast version.
                let thing_buffs_attack = buff_attack_when_cast.get(want_target.thing);
                if let Some(thing_buffs_attack) = thing_buffs_attack {
                    let _play_message = new_combat_stats_status::<StatusIsMeleeAttackBuffed>(
                        &mut has_buffed_attack,
                        &mut combat_stats,
                        *target,
                        thing_buffs_attack.turns,
                        true
                    );
                }

                // Component: BuffsPhysicalDefenseWhenTargeted
                // Note: This component does not yet have both a thorwn and cast version.
                let thing_buffs_defense = buff_defense_when_cast.get(want_target.thing);
                if let Some(thing_buffs_defense) = thing_buffs_defense {
                    let _play_message = new_combat_stats_status::<StatusIsPhysicalDefenseBuffed>(
                        &mut has_buffed_defense,
                        &mut combat_stats,
                        *target,
                        thing_buffs_defense.turns,
                        true
                    );
                }
            }

            // Components: SpawnsEntityInAreaWhen{Thrown|Cast}
            let spawning_data = match verb {
                TargetingVerb::Throw => spawns_entity_in_area_when_thrown.get(want_target.thing).map(|d| &d.0),
                TargetingVerb::Cast => spawns_entity_in_area_when_cast.get(want_target.thing).map(|d| &d.0),
            };
            if let Some(spawns) = spawning_data {
                let points = map.get_euclidean_disk_around(target_point, spawns.radius);
                for pt in points.iter() {
                    spawn_buffer.request(EntitySpawnRequest {
                        x: pt.x,
                        y: pt.y,
                        kind: spawns.kind
                    })
                }
            }

            // Component: MoveToPositionWhenCast
            let moves_to_point = move_to_position_when_cast.get(want_target.thing);
            if let Some(_) = moves_to_point {
                wants_to_move.insert(user, WantsToMoveToPosition {
                    pt: target_point.clone(),
                    // Maybe: Switching this to false. I dunno if this will have
                    // a bad effect. We'll see.
                    force: false
                })
                .expect("Could not insert WantsToMoveToPosition.");
            }

            //----------------------------------------------------------------
            // Animations.
            //----------------------------------------------------------------
            let animation_sequence = match verb {
                TargetingVerb::Throw => animation_when_thrown.get(want_target.thing).map(|d| &d.sequence),
                TargetingVerb::Cast => animation_when_cast.get(want_target.thing).map(|d| &d.sequence),
            };
            if let Some(aseq) = animation_sequence {
                let localized: Vec<AnimationBlock> = aseq.iter()
                    .map(|data| data.localize_targeted_or_cast(Some(user_point), Some(target_point)))
                    .filter_map(|data| data)
                    .collect();
                animation_buffer.request_sequence(AnimationSequence::from_blocks(localized));
            }

            //----------------------------------------------------------------
            // Cleanup and resource expnditure.
            //----------------------------------------------------------------
            // If the thing was single use, clean it up. Weapon specials
            // sometimes give a free throw of a consumable thrown weapon.
            let consumable = consumables.get(want_target.thing).is_some();
            let singlecast = single_casts.get(want_target.thing).is_some();
            let throwexpend = expend_special_when_thrown.get(want_target.thing).is_some();
            let castexpend = expend_special_when_cast.get(want_target.thing).is_some();

            match verb {
                TargetingVerb::Throw => {
                    let freethrow = specials.get(want_target.thing)
                        .map(|s| matches!(s.kind, WeaponSpecialKind::ThrowWithoutExpending) && s.is_charged())
                        .map_or(false, |b| b);
                    if freethrow && throwexpend {
                        let special = specials.get_mut(want_target.thing);
                        special.expect("Failure attempting to clear weapon special charge.").expend();
                    } else if consumable && !freethrow {
                        entities.delete(want_target.thing)
                            .expect("Failed to delete consumed entity when thrown.");
                    };
                }
                TargetingVerb::Cast => {
                    if singlecast {
                        in_spellbooks.remove(want_target.thing)
                            .expect("Failed to remove weapon from spellbook on special resolution.");
                        single_casts.remove(want_target.thing)
                            .expect("Failed to single cast compoenent on special resolution.");
                    }
                    if castexpend {
                        let special = specials.get_mut(want_target.thing);
                        special.expect("Failure attempting to clear weapon special charge.").expend();
                    }
                    let sc = spell_charges.get_mut(want_target.thing);
                    if let Some(sc) = sc { sc.expend_charge() }
                }
            }

        } // Loop over wants_targets.

        wants_target.clear();
    }
}

// Helper function to find all targets of a given thing.
//   - Base Case: Find all entites at the given position.
//   - AOE Case: Find all entities within a given viewshed (defined by a radius)
//     of a given position.
//   - Along Ray Case: Draw a ray from the source to the target point, stopping
//     when the ray encounters a blocked tile (if requested). Target all entities in
//     the final tile.
fn find_targets<'a>(map: &'a Map, user_pos: Point, target_pos: Point, kind: TargetingKind) -> Vec<(&'a Entity, Point)> {
    let mut targets: Vec<(&Entity, Point)> = Vec::new();
    let idx = map.xy_idx(target_pos.x, target_pos.y);
    match kind {
        TargetingKind::Simple => {
            for target in map.tile_content[idx].iter() {
                targets.push((target, target_pos));
            }
        }
        TargetingKind::AreaOfEffect {radius} => {
            let mut blast_tiles = map.get_euclidean_disk_around(target_pos, radius);
            blast_tiles.retain(
                |p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
            );
            for tile in blast_tiles.iter() {
                let idx = map.xy_idx(tile.x, tile.y);
                for target in map.tile_content[idx].iter() {
                    targets.push((target, *tile));
                }
            }
        }
        TargetingKind::AlongRay {until_blocked} => {
            let tiles = map.get_ray_tiles(user_pos, target_pos, until_blocked);
            let last_tile = tiles.last();
            if let Some(tile) = last_tile {
                let idx = map.xy_idx(tile.x, tile.y);
                for target in map.tile_content[idx].iter() {
                    targets.push((target, *tile));
                }
            }
        }
    }
    targets
}