
use super::{
    CombatStats, HungerClock, GameLog, AnimationRequestBuffer, AnimationRequest,
    Name, Position, Renderable, WantsToUseUntargeted, Consumable, Untargeted,
    ProvidesFullHealing, ProvidesFullFood, MovesToRandomPosition,
    WantsToMoveToRandomPosition, IncreasesMaxHpWhenUsed,
    ProvidesFireImmunityWhenUsed, ProvidesChillImmunityWhenUsed,
    StatusIsImmuneToFire, StatusIsImmuneToChill, new_status
};
use specs::prelude::*;


pub struct UntargetedSystem {}

// Searches for WantsToUseUntargeted compoents and then processes the results by
// looking for vatious effect encoding components on the thing:
#[derive(SystemData)]
pub struct UntargetedSystemData<'a> {
    entities: Entities<'a>,
    log: WriteExpect<'a, GameLog>,
    animation_builder: WriteExpect<'a, AnimationRequestBuffer>,
    names: ReadStorage<'a, Name>,
    positions: ReadStorage<'a, Position>,
    renderables: ReadStorage<'a, Renderable>,
    consumables: ReadStorage<'a, Consumable>,
    untargeteds: ReadStorage<'a, Untargeted>,
    wants_use: WriteStorage<'a, WantsToUseUntargeted>,
    increases_hp: ReadStorage<'a, IncreasesMaxHpWhenUsed>,
    provides_fire_immunity: ReadStorage<'a, ProvidesFireImmunityWhenUsed>,
    provides_chill_immunity: ReadStorage<'a, ProvidesChillImmunityWhenUsed>,
    healing: ReadStorage<'a, ProvidesFullHealing>,
    foods: ReadStorage<'a, ProvidesFullFood>,
    teleports: ReadStorage<'a, MovesToRandomPosition>,
    wants_to_teleport: WriteStorage<'a, WantsToMoveToRandomPosition>,
    combat_stats: WriteStorage<'a, CombatStats>,
    hunger_clocks: WriteStorage<'a, HungerClock>,
    status_fire_immunity: WriteStorage<'a, StatusIsImmuneToFire>,
    status_chill_immunity: WriteStorage<'a, StatusIsImmuneToChill>,
}

impl<'a> System<'a> for UntargetedSystem {

    type SystemData = UntargetedSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let UntargetedSystemData {
            entities,
            mut log,
            mut animation_builder,
            names,
            positions,
            renderables,
            consumables,
            untargeteds,
            mut wants_use,
            increases_hp,
            provides_fire_immunity,
            provides_chill_immunity,
            healing,
            foods,
            teleports,
            mut wants_to_teleport,
            mut combat_stats,
            mut hunger_clocks,
            mut status_fire_immunity,
            mut status_chill_immunity
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
                new_status::<StatusIsImmuneToFire>(
                    &mut status_fire_immunity,
                    entity,
                    provides_immunity.turns
                );
                // let name = names.get(entity);
                // if let (Some(name), Some(thing_name)) = (name, thing_name) {
                //     log.entries.push(format!(
                //         "{} {} the {}, and no longer fears fire.",
                //         name.name,
                //         verb,
                //         thing_name.name
                //     ));
                // }
            }

            // Component: ProvidesFireImmunityWhenUsed
            let thing_provides_chill_immunity = provides_chill_immunity.get(want_use.thing);
            if let Some(provides_immunity) = thing_provides_chill_immunity {
                new_status::<StatusIsImmuneToChill>(
                    &mut status_chill_immunity,
                    entity,
                    provides_immunity.turns
                );
                // let name = names.get(entity);
                // if let (Some(name), Some(thing_name)) = (name, thing_name) {
                //     log.entries.push(format!(
                //         "{} {} the {}, and no longer fears cold.",
                //         name.name,
                //         verb,
                //         thing_name.name
                //     ));
                // }
            }

            // Compontnet: MovesToRandomPosition
            let thing_teleports = teleports.get(want_use.thing);
            if let Some(_) = thing_teleports {
                wants_to_teleport.insert(entity, WantsToMoveToRandomPosition {})
                    .expect("Failed to insert WantsToMoveToRandomPosition");
                let user_name = names.get(entity);
                if let (Some(thing_name), Some(user_name)) = (thing_name, user_name) {
                    log.entries.push(format!(
                        "You {} the {}, and {} disappears.",
                        verb,
                        thing_name.name,
                        user_name.name
                    ));
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