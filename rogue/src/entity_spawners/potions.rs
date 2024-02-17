use super::{
    EntitySpawnKind, Name, Position, Renderable, PickUpable, Useable,
    Throwable, Targeted, TargetingKind, Untargeted, Consumable,
    ProvidesFullHealing, IncreasesMaxHpWhenUsed, InflictsDamageWhenTargeted,
    InflictsFreezingWhenTargeted, InflictsBurningWhenTargeted,
    AreaOfEffectAnimationWhenTargeted, MovesToRandomPosition,
    SpawnsEntityInAreaWhenTargeted, ProvidesFireImmunityWhenUsed,
    ProvidesChillImmunityWhenUsed, ProvidesFullSpellRecharge,
    DecreasesSpellRechargeWhenUsed, SimpleMarker, SerializeMe, MarkedBuilder,
    ElementalDamageKind, StatusIsImmuneToFire, StatusIsImmuneToChill
};
use rltk::{RGB};
use specs::prelude::*;

const POTION_THROW_RANGE: f32 = 8.5;
const POTION_RENDER_ORDER: i32 = 2;

//----------------------------------------------------------------------------
// Healing Potion
//
// Fully restores the user's HP (or a target if thrown) and increases their max
// HP a bit.
//----------------------------------------------------------------------------
const HEALTH_POTION_MAX_HP_INCREASE: i32 = 10;

pub fn health(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            order: POTION_RENDER_ORDER,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Healing".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted {verb: "drinks".to_string()})
        // TODO: Restore this after we've added allies.
        // .with(Throwable {})
        // .with(Targeted {
        //     verb: "throws".to_string(),
        //     range: POTION_THROW_RANGE,
        //     kind: TargetingKind::Simple
        // })
        .with(Consumable {})
        .with(ProvidesFullHealing {})
        .with(IncreasesMaxHpWhenUsed {amount: HEALTH_POTION_MAX_HP_INCREASE})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

//----------------------------------------------------------------------------
// Recharging Potion
//
// Fully restores all spell charges to all the user's spells and increases
// their spell's recharge rate a bit.
//----------------------------------------------------------------------------
const RECHARGING_POTION_SPELL_CHARGE_DECREASE_PERCENTAGE: i32 = 25;

pub fn recharging(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::MEDIUM_PURPLE),
            bg: RGB::named(rltk::BLACK),
            order: POTION_RENDER_ORDER,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Recharging".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted {verb: "drinks".to_string()})
        .with(Consumable {})
        // TODO: Make this throwable once we've added allies.
        .with(ProvidesFullSpellRecharge {})
        .with(DecreasesSpellRechargeWhenUsed {
            percentage: RECHARGING_POTION_SPELL_CHARGE_DECREASE_PERCENTAGE,
        })
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

//----------------------------------------------------------------------------
// Fire Potion
//
// Has differing effects if used vs. if thrown.
//   - If Used: Grants immunity from fire damage for a number of game turns.
//   - If Thrown: Deals AOE fire damage, and spawns fire entities within the AOE.
//----------------------------------------------------------------------------
const FIRE_POTION_AOE_RADIUS: f32 = 2.5;
const FIRE_POTION_SPAWN_RADIUS: i32 = 1;
const FIRE_POTION_DAMAGE: i32 = 10;
const FIRE_POTION_BURNING_TURNS: i32 = 5;
// TODO: This should be global throughout the game.
const FIRE_POTION_BURNING_TICK_DAMAGE: i32 = 5;
const FIRE_POTION_SPAWN_SPREAD_CHANCE: i32 = 50;
const FIRE_POTION_SPAWN_DISSIPATE_CHANCE: i32 = 50;
const FIRE_POTION_IMMUNITY_TURNS: i32 = 50;

pub fn fire(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            order: POTION_RENDER_ORDER,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Fire".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted {verb: "drinks".to_string()})
        .with(Throwable {})
        .with(Targeted {
            verb: "throws".to_string(),
            range: POTION_THROW_RANGE,
            kind: TargetingKind::AreaOfEffect {
                radius: FIRE_POTION_AOE_RADIUS
            }
        })
        .with(Consumable {})
        .with(ProvidesFireImmunityWhenUsed {
            turns: FIRE_POTION_IMMUNITY_TURNS
        })
        .with(InflictsDamageWhenTargeted {
            damage: FIRE_POTION_DAMAGE,
            kind: ElementalDamageKind::Fire
        })
        .with(InflictsBurningWhenTargeted {
            turns: FIRE_POTION_BURNING_TURNS,
            tick_damage: FIRE_POTION_BURNING_TICK_DAMAGE
        })
        .with(SpawnsEntityInAreaWhenTargeted {
            radius: FIRE_POTION_SPAWN_RADIUS,
            kind: EntitySpawnKind::Fire {
                spread_chance: FIRE_POTION_SPAWN_SPREAD_CHANCE,
                dissipate_chance: FIRE_POTION_SPAWN_DISSIPATE_CHANCE,
            }
        })
        .with(AreaOfEffectAnimationWhenTargeted {
            // TODO: This field should probably just be f32.
            radius: FIRE_POTION_AOE_RADIUS as i32,
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::RED),
            glyph: rltk::to_cp437('^')
        })
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

//----------------------------------------------------------------------------
// Chill Potion
//
// Has differing effects if used vs. if thrown.
//   - If Used: Grants immunity from chill damage for a number of game turns.
//   - If Thrown: Deals AOE chill damage, and spawns chill entities within the AOE.
//----------------------------------------------------------------------------
const CHILL_POTION_AOE_RADIUS: f32 = 2.5;
const CHILL_POTION_SPAWN_RADIUS: i32 = 2;
const CHILL_POTION_DAMAGE: i32 = 10;
const CHILL_POTION_FREEZING_TURNS: i32 = 5;
const CHILL_POTION_SPAWN_SPREAD_CHANCE: i32 = 60;
const CHILL_POTION_SPAWN_DISSIPATE_CHANCE: i32 = 30;
const CHILL_POTION_IMMUNITY_TURNS: i32 = 50;

pub fn freezing(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
            order: POTION_RENDER_ORDER,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Freezing".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted {verb: "drinks".to_string()})
        .with(Throwable {})
        .with(Targeted {
            verb: "throws".to_string(),
            range: POTION_THROW_RANGE,
            kind: TargetingKind::AreaOfEffect {
                radius: CHILL_POTION_AOE_RADIUS
            }
        })
        .with(Consumable {})
        .with(ProvidesChillImmunityWhenUsed {
            turns: CHILL_POTION_IMMUNITY_TURNS
        })
        .with(InflictsDamageWhenTargeted {
            damage: CHILL_POTION_DAMAGE,
            kind: ElementalDamageKind::Chill
        })
        .with(InflictsFreezingWhenTargeted {
            turns: CHILL_POTION_FREEZING_TURNS
        })
        .with(SpawnsEntityInAreaWhenTargeted {
            radius: CHILL_POTION_SPAWN_RADIUS,
            kind: EntitySpawnKind::Chill {
                spread_chance: CHILL_POTION_SPAWN_SPREAD_CHANCE,
                dissipate_chance: CHILL_POTION_SPAWN_DISSIPATE_CHANCE,
            }
        })
        .with(AreaOfEffectAnimationWhenTargeted {
            radius: CHILL_POTION_AOE_RADIUS as i32,
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::LIGHT_BLUE),
            glyph: rltk::to_cp437('*')
        })
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

//----------------------------------------------------------------------------
// Teleportation Potion
//
// Teleports to user or target to a random positon on the curernt dungeon floor.
//----------------------------------------------------------------------------
pub fn teleportation(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
            order: POTION_RENDER_ORDER,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Teleportation".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted{ verb: "drinks".to_string()})
        .with(Throwable {})
        .with(Targeted {
            verb: "throws".to_string(),
            range: POTION_THROW_RANGE,
            kind: TargetingKind::Simple
        })
        .with(Consumable {})
        .with(MovesToRandomPosition {})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}