use super::{
    Map, Point, CombatStats, GameLog, AnimationBuilder, AnimationRequest,
    EntitySpawnRequestBuffer, EntitySpawnRequest, Name, Renderable,
    Consumable, SpellCharges, Position, WantsToUseTargeted, Targeted,
    TargetingKind, ProvidesFullHealing, MovesToRandomPosition,
    InflictsDamageWhenTargeted, InflictsFreezingWhenTargeted,
    InflictsBurningWhenTargeted, AreaOfEffectAnimationWhenTargeted,
    AlongRayAnimationWhenTargeted, WantsToTakeDamage,
    WantsToMoveToRandomPosition, StatusIsFrozen, StatusIsBurning,
    SpawnsEntityInAreaWhenTargeted, StatusIsImmuneToFire,
    StatusIsImmuneToChill
};
use specs::prelude::*;

pub struct TargetedSystem {}

// Searches for WantsToUseTargeted compoents and processes the results by:
//   - Finding targets in the selected position, or the area of effect.
//   - Looking for vatious effect encoding components on the thing.
#[derive(SystemData)]
pub struct TargetedSystemData<'a> {
        entities: Entities<'a>,
        map: ReadExpect<'a, Map>,
        log: WriteExpect<'a, GameLog>,
        animation_builder: WriteExpect<'a, AnimationBuilder>,
        spawn_buffer: WriteExpect<'a, EntitySpawnRequestBuffer>,
        names: ReadStorage<'a, Name>,
        renderables: ReadStorage<'a, Renderable>,
        positions: WriteStorage<'a, Position>,
        consumables: ReadStorage<'a, Consumable>,
        spell_charges: WriteStorage<'a, SpellCharges>,
        targeteds: ReadStorage<'a, Targeted>,
        wants_target: WriteStorage<'a, WantsToUseTargeted>,
        combat_stats: WriteStorage<'a, CombatStats>,
        healing: ReadStorage<'a, ProvidesFullHealing>,
        does_damage: ReadStorage<'a, InflictsDamageWhenTargeted>,
        does_freeze: WriteStorage<'a, InflictsFreezingWhenTargeted>,
        does_burn: WriteStorage<'a, InflictsBurningWhenTargeted>,
        aoe_animations: ReadStorage<'a, AreaOfEffectAnimationWhenTargeted>,
        along_ray_animations: ReadStorage<'a, AlongRayAnimationWhenTargeted>,
        spawns_entity_in_area: ReadStorage<'a, SpawnsEntityInAreaWhenTargeted>,
        apply_damages: WriteStorage<'a, WantsToTakeDamage>,
        teleports: ReadStorage<'a, MovesToRandomPosition>,
        wants_to_teleport: WriteStorage<'a, WantsToMoveToRandomPosition>,
        is_frozen: WriteStorage<'a, StatusIsFrozen>,
        is_burning: WriteStorage<'a, StatusIsBurning>,
        is_fire_immune: ReadStorage<'a, StatusIsImmuneToFire>,
        is_chill_immune: ReadStorage<'a, StatusIsImmuneToChill>,
}

impl<'a> System<'a> for TargetedSystem {

    type SystemData = TargetedSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let TargetedSystemData {
            entities,
            map,
            mut log,
            mut animation_builder,
            mut spawn_buffer,
            names,
            renderables,
            mut positions,
            consumables,
            mut spell_charges,
            targeteds,
            mut wants_target,
            mut combat_stats,
            healing,
            does_damage,
            does_freeze,
            does_burn,
            aoe_animations,
            along_ray_animations,
            mut apply_damages,
            teleports,
            mut wants_to_teleport,
            mut is_frozen,
            mut is_burning,
            spawns_entity_in_area,
            is_fire_immune,
            is_chill_immune
        } = data;

        //--------------------------------------------------------------------
        // Main loop through all the targets.
        //--------------------------------------------------------------------
        for (user, want_target) in (&entities, &wants_target).join() {

            // In the case we are casting a spell, we guard against the case
            // that we have no spell charges left.
            let proceed = spell_charges
                .get(want_target.thing)
                .map_or(true, |sc| sc.charges > 0);
            if !proceed {continue}

            // Stuff needed to construct log messages.
            let thing_name = names.get(want_target.thing);
            let default_verb = "target".to_string();
            let verb = targeteds
                .get(want_target.thing)
                .map(|t| t.verb.clone())
                .unwrap_or(default_verb);

            // Gather up all the entities that are either at the targeted
            // position or within the area of effect.
            let user_position = positions.get(user).unwrap_or(&Position {x: 0, y: 0});
            let user_point = Point {x: user_position.x, y: user_position.y};
            let target_point = want_target.target;
            let targeting_kind = targeteds
                .get(want_target.thing)
                .map(|t| t.kind.clone())
                .expect("Tried to target but no Targeted component.");
            let targets: Vec<(&Entity, Point)> = find_targets(&*map, user_point, target_point, targeting_kind)
                .into_iter()
                .filter(|(&e, _pt)| e != user)
                .collect();

            // Apply the effect to each target in turn. This is essentially a
            // bit switch over the possible types of effects.
            for (target, target_point) in targets {

                // Component: ProvidesHealing.
                let thing_heals = healing.get(want_target.thing);
                let stats = combat_stats.get_mut(*target);
                if let (Some(_), Some(stats)) = (thing_heals, stats) {
                    stats.full_heal();
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        log.entries.push(format!(
                            "You {} the {}, healing {}.",
                            verb,
                            thing_name.name,
                            target_name.name
                        ));
                    }
                    let render = renderables.get(*target);
                    if let Some(render) = render {
                        animation_builder.request(AnimationRequest::Healing {
                            x: target_point.x,
                            y: target_point.y,
                            fg: render.fg,
                            bg: render.bg,
                            glyph: render.glyph,
                        })
                    }
                }

                // Compontnet: MovesToRandomPosition
                let thing_teleports = teleports.get(want_target.thing);
                let target_pos = positions.get_mut(*target);
                if let (Some(_), Some(_)) = (thing_teleports, target_pos) {
                    wants_to_teleport.insert(*target, WantsToMoveToRandomPosition {})
                        .expect("Failed to insert WantsToMoveToRandomPostion.");
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        log.entries.push(format!(
                            "You {} the {}, and {} disappears.",
                            verb,
                            thing_name.name,
                            target_name.name
                        ));
                    }
                }

                // Component: InflictsDamageWhenTargeted
                let stats = combat_stats.get_mut(*target);
                let thing_damages = does_damage.get(want_target.thing);
                if let (Some(thing_damages), Some(_stats)) = (thing_damages, stats) {
                    WantsToTakeDamage::new_damage(
                        &mut apply_damages,
                        *target,
                        thing_damages.damage,
                        thing_damages.kind
                    );
                    let thing_name = names.get(want_target.thing);
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        log.entries.push(format!(
                            "You {} the {}, dealing {} {} damage.",
                            verb,
                            thing_name.name,
                            target_name.name,
                            thing_damages.damage
                        ))
                    }
                }

                // Component: InflictsFreezingWhenTargeted
                let thing_freezes = does_freeze.get(want_target.thing);
                if let Some(thing_freezes) = thing_freezes {
                    let play_message = StatusIsFrozen::new_status(
                        &mut is_frozen,
                        &is_chill_immune,
                        *target,
                        thing_freezes.turns
                    );
                    let thing_name = names.get(want_target.thing);
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        if play_message {
                            log.entries.push(format!(
                                "You {} the {}, freezing {} in place.",
                                verb,
                                thing_name.name,
                                target_name.name,
                            ))
                        }
                    }
                }

                // Component: InflictsBurningWhenTargeted
                let thing_burns = does_burn.get(want_target.thing);
                if let Some(thing_burns) = thing_burns {
                    let play_message = StatusIsBurning::new_status(
                        &mut is_burning,
                        &is_fire_immune,
                        *target,
                        thing_burns.turns,
                        thing_burns.tick_damage
                    );
                    let thing_name = names.get(want_target.thing);
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        if play_message {
                            log.entries.push(format!(
                                "You {} the {}, stting {} ablaze.",
                                verb,
                                thing_name.name,
                                target_name.name,
                            ))
                        }
                    }
                }
            }

            // Component: AreaOfEffectAnimationWhenTargeted
            let has_aoe_animation = aoe_animations.get(want_target.thing);
            if let Some(has_aoe_animation) = has_aoe_animation {
                animation_builder.request(AnimationRequest::AreaOfEffect {
                    x: target_point.x,
                    y: target_point.y,
                    fg: has_aoe_animation.fg,
                    bg: has_aoe_animation.bg,
                    glyph: has_aoe_animation.glyph,
                    radius: has_aoe_animation.radius
                })
            }

            // Component: AlongRayAnimationWhenTargeted
            let has_ray_animation = along_ray_animations.get(want_target.thing);
            if let Some(has_ray_animation) = has_ray_animation {
                animation_builder.request(AnimationRequest::AlongRay {
                    source_x: user_point.x,
                    source_y: user_point.y,
                    target_x: target_point.x,
                    target_y: target_point.y,
                    fg: has_ray_animation.fg,
                    bg: has_ray_animation.bg,
                    glyph: has_ray_animation.glyph
                })
            }

            // Component: SpawnsEntityInAreaWhenTargeted
            let spawns_entities_when_targeted = spawns_entity_in_area.get(want_target.thing);
            if let Some(spawns) = spawns_entities_when_targeted {
                let points = rltk::field_of_view(target_point, spawns.radius, &*map);
                for pt in points.iter() {
                    spawn_buffer.request(EntitySpawnRequest {
                        x: pt.x,
                        y: pt.y,
                        kind: spawns.kind
                    })
                }
            }

            // If the thing was single use, clean it up.
            let consumable = consumables.get(want_target.thing);
            if let Some(_) = consumable {
                entities.delete(want_target.thing).expect("Potion delete failed.");
            }
            // If the thing is a spell, we've used up one of the spell charges.
            let sc = spell_charges.get_mut(want_target.thing);
            if let Some(sc) = sc {
                sc.expend_charge()
            }

        }
        wants_target.clear();
    }
}

// Helper function to find all targets of a given thing.
//   - Base Case: Find all entites at the given position.
//   - AOE Case: Find all entities within a given viewshed (defined by a radius)
//     of a given position.
//   - Along Ray Case: Draw a ray from the source to the target point, stopping
//     when the ray encounters a blocked tile. Target all entities in the final
//     tile.
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
            let mut blast_tiles = map.get_aoe_tiles(target_pos, radius);
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
        TargetingKind::AlongRay => {
            let tiles = map.get_ray_tiles(user_pos, target_pos);
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