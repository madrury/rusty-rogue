use crate::components::*;
use crate::components::animation::*;
use crate::components::game_effects::*;
use crate::components::magic::*;
use crate::components::signaling::*;
use crate::components::spawn_despawn::*;
use crate::components::status_effects::*;
use crate::components::targeting::*;

use rltk::{RGB, FontCharType};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};

// ♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪♪
const SPELL_SCROLL_GLYPH: FontCharType = 13;

//----------------------------------------------------------------------------
// Fire elemental attack spells.
//----------------------------------------------------------------------------
const FIREBALL_REGEN_TICKS: i32 = 100;
pub const FIREBALL_RANGE: f32 = 10.0;
pub const FIREBALL_DAMAGE: i32 = 10;
pub const FIREBALL_BURNING_TURNS: i32 = 5;
pub const FIREBALL_BURNING_TICK_DAMAGE: i32 = 2;

pub fn fireball(ecs: &mut World, x: i32, y: i32, max_charges: i32, charges: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: SPELL_SCROLL_GLYPH,
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Fireball".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::FireAttackLevel1
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_ticks: FIREBALL_REGEN_TICKS,
            ticks: 0,
            element: ElementalDamageKind::Fire
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: FIREBALL_RANGE,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(InflictsDamageWhenCast (
            InflictsDamageData {
                damage: FIREBALL_DAMAGE,
                kind: ElementalDamageKind::Fire
            }
        ))
        .with(InflictsBurningWhenCast (
            InflictsBurningData {
                turns: FIREBALL_BURNING_TURNS,
                tick_damage: FIREBALL_BURNING_TICK_DAMAGE
            }
        ))
        .with(AlongRayAnimationWhenCast (
            AlongRayAnimationData {
                fg: RGB::named(rltk::ORANGE),
                bg: RGB::named(rltk::RED),
                glyph: rltk::to_cp437('^'),
                until_blocked: true
            }
        ))
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

const FIREBLAST_MAX_CHARGES: i32 = 2;
const FIREBLAST_REGEN_TICKS: i32 = 200;
const FIREBLAST_RANGE: f32 = 8.0;
const FIREBLAST_AOE_RADIUS: f32 = 2.0;
const FIREBLAST_DAMAGE: i32 = 15;
const FIREBLAST_BURNING_TURNS: i32 = 5;
const FIREBLAST_BURNING_TICK_DAMAGE: i32 = 2;
pub const FIREBLAST_FIRE_SPREAD_CHANCE: i32 = 50;
pub const FIREBLAST_FIRE_DISSIPATE_CHANCE: i32 = 50;

pub fn fireblast(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: SPELL_SCROLL_GLYPH,
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Fireblast".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::FireAttackLevel2
        })
        .with(SpellCharges {
            max_charges: FIREBLAST_MAX_CHARGES,
            charges: 1,
            regen_ticks: FIREBLAST_REGEN_TICKS,
            ticks: 0,
            element: ElementalDamageKind::Fire
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: FIREBLAST_RANGE,
            kind: TargetingKind::AreaOfEffect {
                radius: FIREBLAST_AOE_RADIUS
            }
        })
        .with(InflictsDamageWhenCast(
            InflictsDamageData {
                damage: FIREBLAST_DAMAGE,
                kind: ElementalDamageKind::Fire
            }
        ))
        .with(InflictsBurningWhenCast (
            InflictsBurningData {
                turns: FIREBLAST_BURNING_TURNS,
                tick_damage: FIREBLAST_BURNING_TICK_DAMAGE
            }
        ))
        .with(SpawnsEntityInAreaWhenCast (
            SpawnsEntityInAreaData {
                radius: FIREBLAST_AOE_RADIUS,
                kind: EntitySpawnKind::Fire {
                    spread_chance: FIREBLAST_FIRE_SPREAD_CHANCE,
                    dissipate_chance: FIREBLAST_FIRE_DISSIPATE_CHANCE,
                }
            }
        ))
        .with(AreaOfEffectAnimationWhenCast (
            AreaOfEffectAnimationData {
                radius: FIREBLAST_AOE_RADIUS,
                fg: RGB::named(rltk::ORANGE),
                bg: RGB::named(rltk::RED),
                glyph: rltk::to_cp437('^')
            }
        ))
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

//----------------------------------------------------------------------------
// Freezing elemental attack spells.
//----------------------------------------------------------------------------
const ICESPIKE_REGEN_TICKS: i32 = 100;
pub const ICESPIKE_RANGE: f32 = 10.0;
pub const ICESPIKE_DAMAGE: i32 = 10;
pub const ICESPIKE_FREEZING_TURNS: i32 = 10;

pub fn icespike(ecs: &mut World, x: i32, y: i32, max_charges: i32, charges: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: SPELL_SCROLL_GLYPH,
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Icespike".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::ChillAttackLevel1
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_ticks: ICESPIKE_REGEN_TICKS,
            ticks: 0,
            element: ElementalDamageKind::Freezing
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: ICESPIKE_RANGE,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(InflictsDamageWhenCast (
            InflictsDamageData {
                damage: ICESPIKE_DAMAGE,
                kind: ElementalDamageKind::Freezing
            }
        ))
        .with(InflictsFreezingWhenCast (
            InflictsFreezingData { turns: ICESPIKE_FREEZING_TURNS }
        ))
        .with(AlongRayAnimationWhenCast (
            AlongRayAnimationData {
                fg: RGB::named(rltk::WHITE),
                bg: RGB::named(rltk::LIGHT_BLUE),
                glyph: rltk::to_cp437('*'),
                until_blocked: true
            }
        ))
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}


const ICEBLAST_MAX_CHARGES: i32 = 2;
const ICEBLAST_REGEN_TICKS: i32 = 200;
const ICEBLAST_RANGE: f32 = 8.0;
const ICEBLAST_AOE_RADIUS: f32 = 2.0;
const ICEBLAST_DAMAGE: i32 = 15;
const ICEBLAST_FROZEN_TURNS: i32 = 10;
pub const ICEBLAST_CHILL_SPREAD_CHANCE: i32 = 70;
pub const ICEBLAST_CHILL_DISSIPATE_CHANCE: i32 = 30;

pub fn iceblast(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: SPELL_SCROLL_GLYPH,
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Iceblast".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::ChillAttackLevel2
        })
        .with(SpellCharges {
            max_charges: ICEBLAST_MAX_CHARGES,
            charges: 1,
            regen_ticks: ICEBLAST_REGEN_TICKS,
            ticks: 0,
            element: ElementalDamageKind::Freezing
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: ICEBLAST_RANGE,
            kind: TargetingKind::AreaOfEffect {
                radius: ICEBLAST_AOE_RADIUS
            }
        })
        .with(InflictsDamageWhenCast (
            InflictsDamageData {
                damage: ICEBLAST_DAMAGE,
                kind: ElementalDamageKind::Freezing
            }
        ))
        .with(InflictsFreezingWhenCast(
            InflictsFreezingData {
                turns: ICEBLAST_FROZEN_TURNS
            }
        ))
        .with(SpawnsEntityInAreaWhenCast (
            SpawnsEntityInAreaData {
                radius: ICEBLAST_AOE_RADIUS,
                kind: EntitySpawnKind::Chill {
                    spread_chance: ICEBLAST_CHILL_SPREAD_CHANCE,
                    dissipate_chance: ICEBLAST_CHILL_DISSIPATE_CHANCE,
                }
            }
        ))
        .with(AreaOfEffectAnimationWhenCast (
            AreaOfEffectAnimationData {
                radius: ICEBLAST_AOE_RADIUS,
                fg: RGB::named(rltk::WHITE),
                bg: RGB::named(rltk::LIGHT_BLUE),
                glyph: rltk::to_cp437('*')
            }
        ))
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

//----------------------------------------------------------------------------
// Non-elemental attack spells.
//----------------------------------------------------------------------------
pub fn magic_missile(ecs: &mut World, x: i32, y: i32, max_charges: i32, charges: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Magic Missile".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::NonElementalAttack
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_ticks: 50,
            ticks: 0,
            element: ElementalDamageKind::Physical
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(InflictsDamageWhenCast (
            InflictsDamageData {
                damage: 10,
                kind: ElementalDamageKind::Physical
            }
        ))
        .with(AlongRayAnimationWhenCast (
            AlongRayAnimationData {
                fg: RGB::named(rltk::SILVER),
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('.'),
                until_blocked: true
            }
        ))
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

//----------------------------------------------------------------------------
// Non-attack spells.
//----------------------------------------------------------------------------
pub fn blink(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Blinking".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::Movement
        })
        .with(SpellCharges {
            max_charges: 3,
            charges: 1,
            regen_ticks: 100,
            ticks: 0,
            element: ElementalDamageKind::None
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 8.5,
            kind: TargetingKind::AlongRay {until_blocked: false}
        })
        .with(MoveToPositionWhenCast {})
        .with(AlongRayAnimationWhenCast (
            AlongRayAnimationData {
                fg: RGB::named(rltk::PURPLE),
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('@'),
                until_blocked: false
            }
        ))
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

pub fn health(ecs: &mut World, x: i32, y: i32, max_charges: i32, charges: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Healing".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::Assist
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_ticks: 100,
            ticks: 0,
            element: ElementalDamageKind::Healing
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(ProvidesFullHealing {})
        .with(AlongRayAnimationWhenCast (
            AlongRayAnimationData {
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('♥'),
                until_blocked: true
            }
        ))
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

pub fn invigorate(ecs: &mut World, x: i32, y: i32, max_charges: i32, charges: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Invigorate".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::Assist
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_ticks: 100,
            ticks: 0,
            element: ElementalDamageKind::None
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(BuffsMeleeAttackWhenCast {turns: 10})
        .with(AlongRayAnimationWhenCast (
            AlongRayAnimationData {
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('▲'),
                until_blocked: true
            }
        ))
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

pub fn protect(ecs: &mut World, x: i32, y: i32, max_charges: i32, charges: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Protection".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Spellbook
        })
        .with(Castable {
            slot: BlessingSlot::Assist
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_ticks: 100,
            ticks: 0,
            element: ElementalDamageKind::None
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(BuffsPhysicalDefenseWhenCast {turns: 10})
        .with(AlongRayAnimationWhenCast (
            AlongRayAnimationData {
                fg: RGB::named(rltk::BLUE),
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('▲'),
                until_blocked: true
            }
        ))
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}