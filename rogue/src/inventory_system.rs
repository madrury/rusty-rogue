use rltk::RandomNumberGenerator;
use super::{
    Map, Point, CombatStats, GameLog, AnimationBuilder, AnimationRequest,
    InBackpack, Name, Renderable, Position, Viewshed, WantsToUseItem,
    WantsToPickupItem, WantsToThrowItem, WantsToEquipItem, WantsToRemoveItem,
    Equipped, Consumable, ProvidesFullHealing, MovesToRandomPosition,
    IncreasesMaxHpWhenUsed, InflictsDamageWhenThrown,
    InflictsFreezingWhenThrown, InflictsBurningWhenThrown,
    AreaOfEffectWhenThrown, AreaOfEffectAnimationWhenThrown, ApplyDamage,
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


pub struct ItemThrowSystem {}

// Searches for WantsToThrowItem compoents and then processes the results by
// finding targets in the selected position, and then looking for vatious effect
// encoding components on the item:
//    ProvidesHealing: Restores all of the using entities hp.
#[derive(SystemData)]
pub struct ThrowItemSystemData<'a> {
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
        wants_throw: WriteStorage<'a, WantsToThrowItem>,
        combat_stats: WriteStorage<'a, CombatStats>,
        healing: ReadStorage<'a, ProvidesFullHealing>,
        does_damage: ReadStorage<'a, InflictsDamageWhenThrown>,
        does_freeze: WriteStorage<'a, InflictsFreezingWhenThrown>,
        does_burn: WriteStorage<'a, InflictsBurningWhenThrown>,
        aoes: ReadStorage<'a, AreaOfEffectWhenThrown>,
        aoe_animations: ReadStorage<'a, AreaOfEffectAnimationWhenThrown>,
        apply_damages: WriteStorage<'a, ApplyDamage>,
        teleports: ReadStorage<'a, MovesToRandomPosition>,
        is_frozen: WriteStorage<'a, StatusIsFrozen>,
        is_burning: WriteStorage<'a, StatusIsBurning>,
}

impl<'a> System<'a> for ItemThrowSystem {

    type SystemData = ThrowItemSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let ThrowItemSystemData {
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
            mut wants_throw,
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

        // The WantsToThrowItem object (do_throw below), has references to the
        // targeted position and the thrown item.
        for (thrower, do_throw) in (&entities, &wants_throw).join() {
            let target_point = do_throw.target;
            let aoe = aoes.get(do_throw.item);

            let targets: Vec<&Entity> = find_targets(&*map, target_point, aoe)
                .into_iter()
                .filter(|&e| *e != thrower)
                .collect();

            for target in targets {
                // Component: ProvidesHealing.
                let item_heals = healing.get(do_throw.item);
                let stats = combat_stats.get_mut(*target);
                if let (Some(_), Some(stats)) = (item_heals, stats) {
                    stats.full_heal();
                    let item_name = names.get(do_throw.item);
                    let target_name = names.get(*target);
                    if let (Some(item_name), Some(target_name)) = (item_name, target_name) {
                        log.entries.push(format!(
                            "You throw the {}, healing {}.",
                            item_name.name,
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
                let item_teleports = teleports.get(do_throw.item);
                let target_pos = positions.get_mut(*target);
                if let (Some(_), Some(tpos)) = (item_teleports, target_pos) {
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
                // Component: InflictsDamageWhenThrown
                let stats = combat_stats.get_mut(*target);
                let item_damages = does_damage.get(do_throw.item);
                if let (Some(item_damages), Some(_stats)) = (item_damages, stats) {
                    ApplyDamage::new_damage(&mut apply_damages, *target, item_damages.damage);
                    let item_name = names.get(do_throw.item);
                    let target_name = names.get(*target);
                    if let (Some(item_name), Some(target_name)) = (item_name, target_name) {
                        log.entries.push(format!(
                            "You throw the {}, dealing {} {} damage.",
                            item_name.name,
                            target_name.name,
                            item_damages.damage
                        ))
                    }
                }
                // Component: InflictsFreezingWhenThrown
                let stats = combat_stats.get_mut(*target);
                let item_freezes = does_freeze.get(do_throw.item);
                if let (Some(item_freezes), Some(_stats)) = (item_freezes, stats) {
                    StatusIsFrozen::new_status(&mut is_frozen, *target, item_freezes.turns);
                    let item_name = names.get(do_throw.item);
                    let target_name = names.get(*target);
                    if let (Some(item_name), Some(target_name)) = (item_name, target_name) {
                        log.entries.push(format!(
                            "You throw the {}, freezing {} in place.",
                            item_name.name,
                            target_name.name,
                        ))
                    }
                }
                // Component: InflictsBurningWhenThrown
                let stats = combat_stats.get_mut(*target);
                let item_burns = does_burn.get(do_throw.item);
                if let (Some(item_burns), Some(_stats)) = (item_burns, stats) {
                    StatusIsBurning::new_status(&mut is_burning, *target, item_burns.turns, item_burns.tick_damage);
                    let item_name = names.get(do_throw.item);
                    let target_name = names.get(*target);
                    if let (Some(item_name), Some(target_name)) = (item_name, target_name) {
                        log.entries.push(format!(
                            "You throw the {}, stting {} ablaze.",
                            item_name.name,
                            target_name.name,
                        ))
                    }
                }
            }
            // Component: AreaOfEffectAnimationWhenThrown
            let has_aoe_animation = aoe_animations.get(do_throw.item);
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