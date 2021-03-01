use super::{
    Map, Point, CombatStats, GameLog, ProvidesHealing, InBackpack, Name,
    Position, WantsToUseItem, WantsToPickupItem, WantsToThrowItem,
    Consumable, InflictsDamageWhenThrown, InflictsFreezingWhenThrown,
    InflictsBurningWhenThrown, AreaOfEffectWhenThrown, ApplyDamage,
    StatusIsFrozen, StatusIsBurning
};
use specs::prelude::*;


pub struct ItemCollectionSystem {}

// Looks for WantsToPickUp components, then tries to place the requested item it
// the owner's backpack by attaching the InBackpack component to the item.
impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, mut log, mut pickups, mut positions, names, mut backpacks) = data;
        for pickup in pickups.join() {
            positions.remove(pickup.item);
            backpacks
                .insert(pickup.item, InBackpack { owner: pickup.by })
                .expect("Unable to insert item in backpack.");
            if pickup.by == *player {
                let name = &names.get(pickup.item).unwrap().name;
                log.entries.push(format!("You pickup the {}", *name))
            }
        }
        pickups.clear();
    }
}


pub struct ItemUseSystem {}

// Searches for WantsToUseItem compoents and then processes the results by
// looking for vatious effect encoding components on the item:
//    ProvidesHealing: Restores all of the using entities hp.
//
impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player,
            mut log,
            names,
            consumables,
            mut wants_use,
            healing,
            mut combat_stats,
        ) = data;

        for (entity, do_use, stats) in (&entities, &wants_use, &mut combat_stats).join() {

            // Component: ProvidesHealing.
            let item_heals = healing.get(do_use.item);
            if let Some(_) = item_heals {
                stats.hp = stats.max_hp; // TODO: This probably should be a system call.
                if entity == *player {
                    log.entries.push(format!(
                        "You drink the {}.",
                        names.get(do_use.item).unwrap().name,
                    ));
                }
            }

            // If the item was single use, clean it up.
            let consumable = consumables.get(do_use.item);
            if let Some(_) = consumable {
                entities.delete(do_use.item).expect("Potion delete failed.");
            }

        }
        wants_use.clear();
    }
}


pub struct ItemThrowSystem {}

// Searches for WantsToThrowItem compoents and then processes the results by
// finding targets in the selected position, and then looking for vatious effect
// encoding components on the item:
//    ProvidesHealing: Restores all of the using entities hp.
impl<'a> System<'a> for ItemThrowSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, WantsToThrowItem>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamageWhenThrown>,
        WriteStorage<'a, InflictsFreezingWhenThrown>,
        WriteStorage<'a, InflictsBurningWhenThrown>,
        ReadStorage<'a, AreaOfEffectWhenThrown>,
        WriteStorage<'a, ApplyDamage>,
        WriteStorage<'a, StatusIsFrozen>,
        WriteStorage<'a, StatusIsBurning>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            map,
            mut log,
            names,
            consumables,
            mut wants_throw,
            mut combat_stats,
            healing,
            does_damage,
            does_freeze,
            does_burn,
            aoes,
            mut apply_damages,
            mut is_frozen,
            mut is_burning,
        ) = data;

        for (thrower, do_throw) in (&entities, &wants_throw).join() {
            let target_point = do_throw.target;
            let aoe = aoes.get(do_throw.item);

            let targets: Vec<&Entity> = find_targets(&*map, target_point, aoe)
                .into_iter()
                .filter(|&e| *e != thrower)
                .collect();

            for target in targets {

                // Component: ProvidesHealing.
                let stats = combat_stats.get_mut(*target);
                let item_heals = healing.get(do_throw.item);
                if let (Some(_), Some(stats)) = (item_heals, stats) {
                    stats.hp = stats.max_hp; // TODO: This probably should be a system call.
                    log.entries.push(format!(
                        "You throw the {}, healing {}.",
                        names.get(do_throw.item).unwrap().name,
                        names.get(*target).unwrap().name,
                    ))
                }

                // Component: InflictsDamageWhenThrown
                let stats = combat_stats.get_mut(*target);
                let item_damages = does_damage.get(do_throw.item);
                if let (Some(item_damages), Some(_stats)) = (item_damages, stats) {
                    ApplyDamage::new_damage(&mut apply_damages, *target, item_damages.damage);
                    log.entries.push(format!(
                        "You throw the {}, dealing {} {} damage.",
                        names.get(do_throw.item).unwrap().name,
                        names.get(*target).unwrap().name,
                        item_damages.damage
                    ))
                }

                // Component: InflictsFreezingWhenThrown
                let stats = combat_stats.get_mut(*target);
                let item_freezes = does_freeze.get(do_throw.item);
                if let (Some(item_freezes), Some(_stats)) = (item_freezes, stats) {
                    StatusIsFrozen::new_status(&mut is_frozen, *target, item_freezes.turns);
                    log.entries.push(format!(
                        "You throw the {}, freezing {} in place.",
                        names.get(do_throw.item).unwrap().name,
                        names.get(*target).unwrap().name,
                    ))
                }

                // Component: InflictsBurningWhenThrown
                let stats = combat_stats.get_mut(*target);
                let item_burns = does_burn.get(do_throw.item);
                if let (Some(item_burns), Some(_stats)) = (item_burns, stats) {
                    StatusIsBurning::new_status(&mut is_burning, *target, item_burns.turns, item_burns.tick_damage);
                    log.entries.push(format!(
                        "You throw the {}, stting {} ablaze.",
                        names.get(do_throw.item).unwrap().name,
                        names.get(*target).unwrap().name,
                    ))
                }

            }
            // If the item was single use, clean it up.
            let consumable = consumables.get(do_throw.item);
            if let Some(_) = consumable {
                entities.delete(do_throw.item).expect("Potion delete failed.");
            }

        }
        wants_throw.clear();
    }
}

// Helper function to find all targets of a given thrown item.
//   - Base Case: Find all entites at the given position.
//   - AOE Case: Find all entities within a given viewshed (defined by a radius)
//     of a given position.
fn find_targets<'a>(map: &'a Map, pt: Point, aoe: Option<&AreaOfEffectWhenThrown>) -> Vec<&'a Entity> {
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
            for tile_idx in blast_tiles.iter() {
                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                for target in map.tile_content[idx].iter() {
                    targets.push(target);
                }
            }
        }
    }
    targets
}