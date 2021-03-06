use super::{
    Map, Point, CombatStats, GameLog, AnimationBuilder, AnimationRequest,
    Name, Renderable, Consumable, Position, Viewshed,
    WantsToUseTargeted,
    ProvidesFullHealing, MovesToRandomPosition,
    InflictsDamageWhenTargeted,
    InflictsFreezingWhenTargeted, InflictsBurningWhenTargeted,
    AreaOfEffectWhenTargeted, AreaOfEffectAnimationWhenTargeted, WantsToTakeDamage,
    StatusIsFrozen, StatusIsBurning
};
use specs::prelude::*;
use rltk::RandomNumberGenerator;

pub struct TargetedSystem {}

// Searches for WantsTotargetthing compoents and then processes the results by
// finding targets in the selected position, and then looking for vatious effect
// encoding components on the thing:
//    ProvidesHealing: Restores all of the using entities hp.
#[derive(SystemData)]
pub struct TargetedSystemData<'a> {
        entities: Entities<'a>,
        map: ReadExpect<'a, Map>,
        log: WriteExpect<'a, GameLog>,
        animation_builder: WriteExpect<'a, AnimationBuilder>,
        rng: WriteExpect<'a, RandomNumberGenerator>,
        names: ReadStorage<'a, Name>,
        renderables: ReadStorage<'a, Renderable>,
        positions: WriteStorage<'a, Position>,
        viewsheds: WriteStorage<'a, Viewshed>,
        consumables: ReadStorage<'a, Consumable>,
        wants_target: WriteStorage<'a, WantsToUseTargeted>,
        combat_stats: WriteStorage<'a, CombatStats>,
        healing: ReadStorage<'a, ProvidesFullHealing>,
        does_damage: ReadStorage<'a, InflictsDamageWhenTargeted>,
        does_freeze: WriteStorage<'a, InflictsFreezingWhenTargeted>,
        does_burn: WriteStorage<'a, InflictsBurningWhenTargeted>,
        aoes: ReadStorage<'a, AreaOfEffectWhenTargeted>,
        aoe_animations: ReadStorage<'a, AreaOfEffectAnimationWhenTargeted>,
        apply_damages: WriteStorage<'a, WantsToTakeDamage>,
        teleports: ReadStorage<'a, MovesToRandomPosition>,
        is_frozen: WriteStorage<'a, StatusIsFrozen>,
        is_burning: WriteStorage<'a, StatusIsBurning>,
}

impl<'a> System<'a> for TargetedSystem {

    type SystemData = TargetedSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let TargetedSystemData {
            entities,
            map,
            mut log,
            mut animation_builder,
            mut rng,
            names,
            renderables,
            mut positions,
            mut viewsheds,
            consumables,
            mut wants_target,
            mut combat_stats,
            healing,
            does_damage,
            does_freeze,
            does_burn,
            aoes,
            aoe_animations,
            mut apply_damages,
            teleports,
            mut is_frozen,
            mut is_burning,
        } = data;

        // The WantsTotargetthing object (do_target below), has references to the
        // targeted position and the targetn thing.
        for (user, want_target) in (&entities, &wants_target).join() {
            let target_point = want_target.target;
            let aoe = aoes.get(want_target.thing);

            let targets: Vec<&Entity> = find_targets(&*map, target_point, aoe)
                .into_iter()
                .filter(|&e| *e != user)
                .collect();

            for target in targets {
                // Component: ProvidesHealing.
                let thing_heals = healing.get(want_target.thing);
                let stats = combat_stats.get_mut(*target);
                if let (Some(_), Some(stats)) = (thing_heals, stats) {
                    stats.full_heal();
                    let thing_name = names.get(want_target.thing);
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        log.entries.push(format!(
                            "You target the {}, healing {}.",
                            thing_name.name,
                            target_name.name
                        ));
                    }
                    let pos = positions.get(*target);
                    let render = renderables.get(*target);
                    if let(Some(pos), Some(render)) = (pos, render) {
                        animation_builder.request(AnimationRequest::Healing {
                            x: pos.x,
                            y: pos.y,
                            fg: render.fg,
                            bg: render.bg,
                            glyph: render.glyph,
                        })
                    }
                }
                // Compontnet: MovesToRandomPosition
                let thing_teleports = teleports.get(want_target.thing);
                let target_pos = positions.get_mut(*target);
                if let (Some(_), Some(tpos)) = (thing_teleports, target_pos) {
                    let new_pos = map.random_unblocked_point(10, &mut *rng);
                    if let Some(new_pos) = new_pos {
                        tpos.x = new_pos.0;
                        tpos.y = new_pos.1;
                        let target_viewshed = viewsheds.get_mut(*target);
                        if let Some(tviewshed) = target_viewshed {
                            tviewshed.dirty = true;
                        }
                        let target_name = names.get(*target);
                        if let Some(tname) = target_name {
                            log.entries.push(format!("The {} vanishes!", tname.name));
                        }
                    }
                }
                // Component: InflictsDamageWhentargetn
                let stats = combat_stats.get_mut(*target);
                let thing_damages = does_damage.get(want_target.thing);
                if let (Some(thing_damages), Some(_stats)) = (thing_damages, stats) {
                    WantsToTakeDamage::new_damage(&mut apply_damages, *target, thing_damages.damage);
                    let thing_name = names.get(want_target.thing);
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        log.entries.push(format!(
                            "You target the {}, dealing {} {} damage.",
                            thing_name.name,
                            target_name.name,
                            thing_damages.damage
                        ))
                    }
                }
                // Component: InflictsFreezingWhentargetn
                let stats = combat_stats.get_mut(*target);
                let thing_freezes = does_freeze.get(want_target.thing);
                if let (Some(thing_freezes), Some(_stats)) = (thing_freezes, stats) {
                    StatusIsFrozen::new_status(&mut is_frozen, *target, thing_freezes.turns);
                    let thing_name = names.get(want_target.thing);
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        log.entries.push(format!(
                            "You target the {}, freezing {} in place.",
                            thing_name.name,
                            target_name.name,
                        ))
                    }
                }
                // Component: InflictsBurningWhentargetn
                let stats = combat_stats.get_mut(*target);
                let thing_burns = does_burn.get(want_target.thing);
                if let (Some(thing_burns), Some(_stats)) = (thing_burns, stats) {
                    StatusIsBurning::new_status(&mut is_burning, *target, thing_burns.turns, thing_burns.tick_damage);
                    let thing_name = names.get(want_target.thing);
                    let target_name = names.get(*target);
                    if let (Some(thing_name), Some(target_name)) = (thing_name, target_name) {
                        log.entries.push(format!(
                            "You target the {}, stting {} ablaze.",
                            thing_name.name,
                            target_name.name,
                        ))
                    }
                }
            }
            // Component: AreaOfEffectAnimationWhentargetn
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

            // If the thing was single use, clean it up.
            let consumable = consumables.get(want_target.thing);
            if let Some(_) = consumable {
                entities.delete(want_target.thing).expect("Potion delete failed.");
            }

        }
        wants_target.clear();
    }
}

// Helper function to find all targets of a given targetn thing.
//   - Base Case: Find all entites at the given position.
//   - AOE Case: Find all entities within a given viewshed (defined by a radius)
//     of a given position.
fn find_targets<'a>(map: &'a Map, pt: Point, aoe: Option<&AreaOfEffectWhenTargeted>) -> Vec<&'a Entity> {
    let mut targets: Vec<&Entity> = Vec::new();
    let idx = map.xy_idx(pt.x, pt.y);
    match aoe {
        None => {
            for target in map.tile_content[idx].iter() {
                targets.push(target);
            }
        }
        Some(aoe) => {
            let mut blast_tiles = rltk::field_of_view(pt, aoe.radius, &*map);
            blast_tiles.retain(
                |p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
            );
            for tile in blast_tiles.iter() {
                let idx = map.xy_idx(tile.x, tile.y);
                for target in map.tile_content[idx].iter() {
                    targets.push(target);
                }
            }
        }
    }
    targets
}