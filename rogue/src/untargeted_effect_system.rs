
use super::{
    Map, Point, CombatStats, HungerClock, GameLog, AnimationBuilder,
    AnimationRequest, Name, Renderable, Position, Viewshed,
    WantsToUseUntargeted, Consumable, Untargeted, ProvidesFullHealing,
    ProvidesFullFood, MovesToRandomPosition, IncreasesMaxHpWhenUsed,
    ProvidesFireImmunityWhenUsed, StatusIsImmuneToFire
};
use specs::prelude::*;
use rltk::RandomNumberGenerator;


pub struct UntargetedSystem {}

// Searches for WantsToUseUntargeted compoents and then processes the results by
// looking for vatious effect encoding components on the thing:
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
    untargeteds: ReadStorage<'a, Untargeted>,
    wants_use: WriteStorage<'a, WantsToUseUntargeted>,
    increases_hp: ReadStorage<'a, IncreasesMaxHpWhenUsed>,
    provides_fire_immunity: ReadStorage<'a, ProvidesFireImmunityWhenUsed>,
    healing: ReadStorage<'a, ProvidesFullHealing>,
    foods: ReadStorage<'a, ProvidesFullFood>,
    teleports: ReadStorage<'a, MovesToRandomPosition>,
    combat_stats: WriteStorage<'a, CombatStats>,
    hunger_clocks: WriteStorage<'a, HungerClock>,
    status_fire_immunity: WriteStorage<'a, StatusIsImmuneToFire>,
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
            untargeteds,
            mut wants_use,
            increases_hp,
            provides_fire_immunity,
            healing,
            foods,
            teleports,
            mut combat_stats,
            mut hunger_clocks,
            mut status_fire_immunity
        } = data;

        // TODO: Joining on combat stats here is probably incorrect.
        for (entity, want_use, stats) in (&entities, &wants_use, &mut combat_stats).join() {

            // Stuff needed for constructing gamelog messages.
            let thing_name = names.get(want_use.thing);
            let default_verb = "target".to_string();
            let verb = untargeteds
                .get(want_use.thing)
                .map(|t| t.verb.clone())
                .unwrap_or(default_verb);

            // Component: IncreasesMaxHpWhenUsed
            //  NOTE: This needs to come BEFORE any healing, so the healing knows
            //  about the new maximum hp.
            let thing_increases_hp = increases_hp.get(want_use.thing);
            if let Some(thing_increases_hp) = thing_increases_hp {
                stats.increase_max_hp(thing_increases_hp.amount);
            }

            // Component: ProvidesFullHealing.
            let thing_heals = healing.get(want_use.thing);
            if let Some(_) = thing_heals {
                stats.full_heal();
                let name = names.get(entity);
                if let (Some(name), Some(thing_name)) = (name, thing_name) {
                    log.entries.push(format!(
                        "{} {} the {}, and feels great!",
                        name.name,
                        verb,
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

            // Component: ProvidesFullFood
            let thing_foods = foods.get(want_use.thing);
            let clock = hunger_clocks.get_mut(entity);
            if let (Some(_), Some(clock)) = (thing_foods, clock) {
                clock.satiate();
                let name = names.get(entity);
                if let (Some(name), Some(thing_name)) = (name, thing_name) {
                    log.entries.push(format!(
                        "{} {} the {}, and feels full.",
                        name.name,
                        verb,
                        thing_name.name
                    ));
                }
            }

            // Component: ProvidesFireImmunityWhenUsed
            let thing_provides_fire_immunity = provides_fire_immunity.get(want_use.thing);
            if let Some(provides_immunity) = thing_provides_fire_immunity {
                StatusIsImmuneToFire::new_status(
                    &mut status_fire_immunity,
                    entity,
                    provides_immunity.turns
                );
                let name = names.get(entity);
                if let (Some(name), Some(thing_name)) = (name, thing_name) {
                    log.entries.push(format!(
                        "{} {} the {}, and no longer fears fire.",
                        name.name,
                        verb,
                        thing_name.name
                    ));
                }
            }

            // Compontnet: MovesToRandomPosition
            let thing_teleports = teleports.get(want_use.thing);
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
                    if let (Some(name), Some(thing_name)) = (name, thing_name) {
                        log.entries.push(format!(
                            "{} {} the {}, and vanishes!",
                            name.name,
                            verb,
                            thing_name.name
                        ));
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
            let consumable = consumables.get(want_use.thing);
            if let Some(_) = consumable {
                entities.delete(want_use.thing).expect("Potion delete failed.");
            }

        }
        wants_use.clear();
    }
}