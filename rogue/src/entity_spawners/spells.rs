
use crate::DissipateWhenBurning;

use super::{
    EntitySpawnKind, Name, Position, Renderable, PickUpable, Castable,
    SpellCharges, TargetedWhenCast, TargetingKind, InflictsDamageWhenTargeted,
    InflictsFreezingWhenTargeted, InflictsBurningWhenTargeted,
    BuffsMeleeAttackWhenTargeted, BuffsPhysicalDefenseWhenTargeted,
    AreaOfEffectAnimationWhenTargeted, AlongRayAnimationWhenTargeted,
    ProvidesFullHealing, SpawnsEntityInAreaWhenTargeted,
    MoveToPositionWhenTargeted, SimpleMarker, SerializeMe, MarkedBuilder,
    ElementalDamageKind, BlessingSlot, StatusIsImmuneToChill
};
use rltk::{RGB};
use specs::prelude::*;

//----------------------------------------------------------------------------
// Fire elemental attack spells.
//----------------------------------------------------------------------------
pub fn fireball(ecs: &mut World, x: i32, y: i32, max_charges: i32, charges: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Fireball".to_string()})
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::FireAttackLevel1
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_time: 100,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(InflictsDamageWhenTargeted {
            damage: 10,
            kind: ElementalDamageKind::Fire
        })
        .with(InflictsBurningWhenTargeted {
            turns: 4,
            tick_damage: 4
        })
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::RED),
            glyph: rltk::to_cp437('^'),
            until_blocked: true
        })
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

pub fn fireblast(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Fireblast".to_string()})
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::FireAttackLevel2
        })
        .with(SpellCharges {
            max_charges: 3,
            charges: 1,
            regen_time: 200,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AreaOfEffect {radius: 2.5}
        })
        .with(InflictsDamageWhenTargeted {
            damage: 10,
            kind: ElementalDamageKind::Fire
        })
        .with(InflictsBurningWhenTargeted {
            turns: 4,
            tick_damage: 4
        })
        .with(SpawnsEntityInAreaWhenTargeted {
            radius: 1,
            kind: EntitySpawnKind::Fire {
                spread_chance: 50,
                dissipate_chance: 50,
            }
        })
        .with(AreaOfEffectAnimationWhenTargeted {
            radius: 2,
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::RED),
            glyph: rltk::to_cp437('^')
        })
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

//----------------------------------------------------------------------------
// Chill elemental attack spells.
//----------------------------------------------------------------------------
pub fn icespike(ecs: &mut World, x: i32, y: i32, max_charges: i32, charges: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Icespike".to_string()})
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::ChillAttackLevel1
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_time: 100,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(InflictsDamageWhenTargeted {
            damage: 10,
            kind: ElementalDamageKind::Chill
        })
        .with(InflictsFreezingWhenTargeted {
            turns: 6,
        })
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::LIGHT_BLUE),
            glyph: rltk::to_cp437('*'),
            until_blocked: true
        })
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

pub fn iceblast(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Scroll of Iceblast".to_string()})
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::ChillAttackLevel2
        })
        .with(SpellCharges {
            max_charges: 3,
            charges: 1,
            regen_time: 200,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AreaOfEffect {radius: 2.5}
        })
        .with(InflictsDamageWhenTargeted {
            damage: 10,
            kind: ElementalDamageKind::Chill
        })
        .with(InflictsFreezingWhenTargeted {turns: 8})
        .with(SpawnsEntityInAreaWhenTargeted {
            radius: 1,
            kind: EntitySpawnKind::Chill {
                spread_chance: 20,
                dissipate_chance: 60,
            }
        })
        .with(AreaOfEffectAnimationWhenTargeted {
            radius: 2,
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::LIGHT_BLUE),
            glyph: rltk::to_cp437('*')
        })
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
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::NonElementalAttack
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_time: 50,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(InflictsDamageWhenTargeted {
            damage: 10,
            kind: ElementalDamageKind::Physical
        })
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
            glyph: rltk::to_cp437('.'),
            until_blocked: true
        })
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
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::Movement
        })
        .with(SpellCharges {
            max_charges: 3,
            charges: 1,
            regen_time: 100,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 8.5,
            kind: TargetingKind::AlongRay {until_blocked: false}
        })
        .with(MoveToPositionWhenTargeted {})
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::PURPLE),
            bg: RGB::named(rltk::BLACK),
            glyph: rltk::to_cp437('@'),
            until_blocked: false
        })
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
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::Assist
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_time: 100,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(ProvidesFullHealing {})
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            glyph: rltk::to_cp437('♥'),
            until_blocked: true
        })
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
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::Assist
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_time: 100,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(BuffsMeleeAttackWhenTargeted {turns: 10})
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            glyph: rltk::to_cp437('▲'),
            until_blocked: true
        })
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
        .with(PickUpable {})
        .with(Castable {
            slot: BlessingSlot::Assist
        })
        .with(SpellCharges {
            max_charges: max_charges,
            charges: charges,
            regen_time: 100,
            time: 0
        })
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(BuffsPhysicalDefenseWhenTargeted {turns: 10})
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            glyph: rltk::to_cp437('▲'),
            until_blocked: true
        })
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}