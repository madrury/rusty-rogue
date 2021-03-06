use super::{
    Map, Point, CombatStats, GameLog, AnimationBuilder, AnimationRequest,
    InBackpack, Name, Renderable, Position, Viewshed, WantsToUseItem,
    WantsToPickupItem, WantsToEquipItem, WantsToRemoveItem,
    Equipped, Consumable, ProvidesFullHealing, MovesToRandomPosition,
    IncreasesMaxHpWhenUsed
};
use specs::prelude::*;
use rltk::RandomNumberGenerator;


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
                let name = &names.get(pickup.item);
                if let Some(name) = name {
                    log.entries.push(format!("You pickup the {}", name.name))
                }
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
#[derive(SystemData)]
pub struct UseItemSystemData<'a> {
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
    wants_use: WriteStorage<'a, WantsToUseItem>,
    increases_hp: ReadStorage<'a, IncreasesMaxHpWhenUsed>,
    healing: ReadStorage<'a, ProvidesFullHealing>,
    teleports: ReadStorage<'a, MovesToRandomPosition>,
    combat_stats: WriteStorage<'a, CombatStats>,
}

impl<'a> System<'a> for ItemUseSystem {

    type SystemData = UseItemSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let UseItemSystemData {
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
            let item_increases_hp = increases_hp.get(do_use.item);
            if let Some(item_increases_hp) = item_increases_hp {
                stats.increase_max_hp(item_increases_hp.amount);
            }

            // Component: ProvidesFullHealing.
            let item_heals = healing.get(do_use.item);
            if let Some(_) = item_heals {
                stats.full_heal();
                let name = names.get(entity);
                let item_name = names.get(do_use.item);
                if let (Some(name), Some(item_name)) = (name, item_name) {
                    log.entries.push(format!(
                        "{} drink's the {}.",
                        name.name,
                        item_name.name
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
            let item_teleports = teleports.get(do_use.item);
            if let Some(_) = item_teleports {
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

            // If the item was single use, clean it up.
            let consumable = consumables.get(do_use.item);
            if let Some(_) = consumable {
                entities.delete(do_use.item).expect("Potion delete failed.");
            }

        }
        wants_use.clear();
    }
}


pub struct ItemEquipSystem {}

// Searches for WantsToEquipItem compoents and then processes the results
// by attaching an Equipped component to the item. This component contains a
// reference to the equipper entity.
impl<'a> System<'a> for ItemEquipSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, WantsToEquipItem>,
        WriteStorage<'a, Equipped>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            names,
            mut wants_equip,
            mut equipped,
        ) = data;
        // Remove any already equipped items.
        let mut already_equipped: Vec<Entity> = Vec::new();
        for (equipper, do_equip) in (&entities, &wants_equip).join() {
            already_equipped.extend(
                (&entities, &equipped)
                    .join()
                    .filter(|(_item, eqp)| eqp.owner == equipper && eqp.slot == do_equip.slot)
                    .map(|(item, _eqp)| item)
            )
        }
        for item in already_equipped {
            equipped.remove(item);
        }
        // Weild the equipment.
        for (equipper, do_equip, name) in (&entities, &wants_equip, &names).join() {
            equipped.
                insert(do_equip.item, Equipped {owner: equipper, slot: do_equip.slot})
                .expect("Failed to equip item.");
            let item_name = names.get(do_equip.item);
            if let Some(item_name) = item_name {
                log.entries.push(format!("{} equipped the {}.", name.name, item_name.name));
            }
        }
        wants_equip.clear();
    }
}


pub struct ItemRemoveSystem {}

// Searches for WantsToEquipItem compoents and then processes the results
// by attaching an Equipped component to the item. This component contains a
// reference to the equipper entity.
impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            names,
            mut wants_remove,
            mut equipped,
        ) = data;

        for (remover, do_remove) in (&entities, &wants_remove).join() {
            equipped.remove(do_remove.item);
            let item_name = names.get(do_remove.item);
            let remover_name = names.get(remover);
            if let (Some(item_name), Some(remover_name)) = (item_name, remover_name) {
                log.entries.push(format!(
                    "{} reomves {}.", remover_name.name, item_name.name
                ))
            }
        }
        wants_remove.clear();
    }
}