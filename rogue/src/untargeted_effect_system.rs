
use super::{
    Map, Point, CombatStats, GameLog, AnimationBuilder, AnimationRequest,
    Name, Renderable, Position, Viewshed, WantsToUseUntargeted, Consumable,
    ProvidesFullHealing, MovesToRandomPosition, IncreasesMaxHpWhenUsed
};
use specs::prelude::*;
use rltk::RandomNumberGenerator;


pub struct UntargetedSystem {}

// Searches for WantsToUsething compoents and then processes the results by
// looking for vatious effect encoding components on the thing:
//    ProvidesHealing: Restores all of the using entities hp.
//
#[derive(SystemData)]
pub struct UntargetedSystemData<'a> {
    entities: Entities<'a>,
    player: ReadExpect<'a, Entity>,
    player_position: WriteExpect<'a, Point>,
    map: ReadExpect<'a, Map>,
    log: WriteExpect<'a, GameLog>,
    animation_builder: WriteExpect<'a, AnimationBuilder>,
    rng: WriteExpect<'a, RandomNumberGenerator>,
    names: ReadStorage<'a, Name>,
    renderables: ReadStorage<'a, Renderable>,
    positions: WriteStorage<'a, Position>,
    viewsheds: WriteStorage<'a, Viewshed>,
    consumables: ReadStorage<'a, Consumable>,
    wants_use: WriteStorage<'a, WantsToUseUntargeted>,
    increases_hp: ReadStorage<'a, IncreasesMaxHpWhenUsed>,
    healing: ReadStorage<'a, ProvidesFullHealing>,
    teleports: ReadStorage<'a, MovesToRandomPosition>,
    combat_stats: WriteStorage<'a, CombatStats>,
}

impl<'a> System<'a> for UntargetedSystem {

    type SystemData = UntargetedSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let UntargetedSystemData {
            entities,
            player,
            mut player_position,
            map,
            mut log,
            mut animation_builder,
            mut rng,
            names,
            renderables,
            mut positions,
            mut viewsheds,
            consumables,
            mut wants_use,
            increases_hp,
            healing,
            teleports,
            mut combat_stats,
        } = data;

        for (entity, do_use, stats) in (&entities, &wants_use, &mut combat_stats).join() {

            // Component: IncreasesMaxHpWhenUsed
            //  NOTE: This needs to come BEFORE any healing, so the healing knows
            //  about the new maximum hp.
            let thing_increases_hp = increases_hp.get(do_use.thing);
            if let Some(thing_increases_hp) = thing_increases_hp {
                stats.increase_max_hp(thing_increases_hp.amount);
            }

            // Component: ProvidesFullHealing.
            let thing_heals = healing.get(do_use.thing);
            if let Some(_) = thing_heals {
                stats.full_heal();
                let name = names.get(entity);
                let thing_name = names.get(do_use.thing);
                if let (Some(name), Some(thing_name)) = (name, thing_name) {
                    log.entries.push(format!(
                        "{} drink's the {}.",
                        name.name,
                        thing_name.name
                    ));
                }
                let pos = positions.get(entity);
                let render = renderables.get(entity);
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
            let thing_teleports = teleports.get(do_use.thing);
            if let Some(_) = thing_teleports {
                let new_pos = map.random_unblocked_point(10, &mut *rng);
                let pos = positions.get_mut(entity);
                if let (Some(pos), Some(new_pos)) = (pos, new_pos) {
                    pos.x = new_pos.0;
                    pos.y = new_pos.1;
                    let viewshed = viewsheds.get_mut(entity);
                    if let Some(viewshed) = viewshed {
                        viewshed.dirty = true;
                    }
                    let name = names.get(entity);
                    if let Some(name) = name {
                        log.entries.push(format!("{} vanishes!", name.name));
                    }
                    let render = renderables.get(entity);
                    if let Some(render) = render {
                        animation_builder.request(AnimationRequest::Teleportation {
                            x: pos.x,
                            y: pos.y,
                            bg: render.bg,
                        })
                    }
                    // If the using entity is the player, we have to keep the
                    // player's position synchronized in both their Position
                    // component AND their position as a resource in the ECS.
                    if entity == *player {
                        player_position.x = new_pos.0;
                        player_position.y = new_pos.1;
                    }
                }
            }

            // If the thing was single use, clean it up.
            let consumable = consumables.get(do_use.thing);
            if let Some(_) = consumable {
                entities.delete(do_use.thing).expect("Potion delete failed.");
            }

        }
        wants_use.clear();
    }
}