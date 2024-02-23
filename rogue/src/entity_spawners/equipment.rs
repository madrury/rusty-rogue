use crate::components::*;
use crate::components::animation::*;
use crate::components::magic::*;
use crate::components::game_effects::*;
use crate::components::melee::*;
use crate::components::equipment::*;
use crate::components::signaling::*;
use crate::components::status_effects::*;
use crate::components::targeting::*;

use rltk::RGB;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};

pub fn dagger(ecs: &mut World, x: i32, y: i32)  -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('↑'),
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(PickUpable {
            destination: PickupDestination::Backpack
        })
        .with(Throwable {})
        .with(Consumable {})
        .with(Name {name : "Dagger".to_string()})
        .with(Equippable {slot: EquipmentSlot::Melee})
        .with(MeeleAttackWepon {
            bonus: 2,
            formation: MeeleAttackFormation::Basic,
            element: ElementalDamageKind::Physical
        })
        .with(TargetedWhenThrown {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenThrown(
            InflictsDamageData {
                damage: 15,
                kind: ElementalDamageKind::Physical
            }
        ))
        .with(AlongRayAnimationWhenThrown (
            AlongRayAnimationData {
                fg: RGB::named(rltk::SILVER),
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('↑'),
                until_blocked: true
            }
        ))
        .with(WeaponSpecial {
            regen_time: 100,
            time: 0,
            kind: WeaponSpecialKind::ThrowWithoutExpending
        })
        .with(ExpendWeaponSpecialWhenThrown {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

pub fn sword(ecs: &mut World, x: i32, y: i32)  -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('↑'),
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(PickUpable {
            destination: PickupDestination::Backpack
        })
        .with(Throwable {})
        .with(Consumable {})
        .with(Name {name : "Sword".to_string()})
        .with(Equippable {slot: EquipmentSlot::Melee})
        .with(MeeleAttackWepon {
            bonus: 5,
            formation: MeeleAttackFormation::Basic,
            element: ElementalDamageKind::Physical
        })
        .with(TargetedWhenThrown {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenThrown(
            InflictsDamageData {
                damage: 30,
                kind: ElementalDamageKind::Physical
            }
        ))
        .with(AlongRayAnimationWhenThrown (
            AlongRayAnimationData {
                fg: RGB::named(rltk::SILVER),
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('↑'),
                until_blocked: true
            }
        ))
        .with(WeaponSpecial {
            regen_time: 100,
            time: 0,
            kind: WeaponSpecialKind::SpinAttack
        })
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

pub fn rapier(ecs: &mut World, x: i32, y: i32)  -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('↑'),
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(PickUpable {
            destination: PickupDestination::Backpack
        })
        .with(Throwable {})
        .with(Consumable {})
        .with(Name {name : "Rapier".to_string()})
        .with(Equippable {slot: EquipmentSlot::Melee})
        .with(MeeleAttackWepon {
            bonus: 4,
            formation: MeeleAttackFormation::Dash,
            element: ElementalDamageKind::Physical
        })
        .with(TargetedWhenThrown {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenThrown (
            InflictsDamageData {
                damage: 25,
                kind: ElementalDamageKind::Physical
            }
        ))
        .with(AlongRayAnimationWhenThrown (
            AlongRayAnimationData {
                fg: RGB::named(rltk::SILVER),
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('↑'),
                until_blocked: true
            }
        ))
        .with(WeaponSpecial {
            regen_time: 10,
            time: 0,
            kind: WeaponSpecialKind::Dash
        })
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

pub fn rod(ecs: &mut World, x: i32, y: i32, element: ElementalDamageKind)  -> Option<Entity> {
    let fg = match element {
        ElementalDamageKind::Fire => RGB::named(rltk::ORANGE),
        ElementalDamageKind::Freezing => RGB::named(rltk::LIGHTBLUE),
        _ => RGB::named(rltk::SILVER)
    };
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: fg,
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(PickUpable {
            destination: PickupDestination::Backpack
        })
        .with(Throwable {})
        .with(Consumable {})
        .with(Equippable {slot: EquipmentSlot::Melee})
        .with(Castable { slot: BlessingSlot::None })
        .with(Name {name : "Rod".to_string()})
        .with(MeeleAttackWepon {
            bonus: 0,
            formation: MeeleAttackFormation::Basic,
            element: element
        })
        .with(TargetedWhenThrown {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenThrown(
            InflictsDamageData {
                damage: 15,
                kind: element
            }
        ))
        .with(AlongRayAnimationWhenThrown (
            AlongRayAnimationData {
                fg: fg,
                bg: RGB::named(rltk::BLACK),
                glyph: rltk::to_cp437('/'),
                until_blocked: true
            }
        ))
        .with(WeaponSpecial {
            regen_time: 5,
            time: 0,
            kind: WeaponSpecialKind::AddToSpellBook
        })
        .with(ExpendWeaponSpecialWhenCast {})
        .with(TargetedWhenCast {
            verb: "casts".to_string(),
            range: 6.5,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(InflictsDamageWhenCast (
            InflictsDamageData {
                damage: 10,
                kind: element
            }
        ))
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    match element {
        ElementalDamageKind::Fire => {
            let mut burning = ecs.write_storage::<InflictsBurningWhenCast>();
            burning.insert(entity, InflictsBurningWhenCast (
                InflictsBurningData {
                    turns: 4,
                    tick_damage: 4
                }
            )).expect("Failed to insert burning component when creating rod.");
            let mut animation = ecs.write_storage::<AlongRayAnimationWhenCast>();
            animation.insert(entity, AlongRayAnimationWhenCast (
                AlongRayAnimationData {
                    fg: RGB::named(rltk::ORANGE),
                    bg: RGB::named(rltk::RED),
                    glyph: rltk::to_cp437('^'),
                    until_blocked: true
                }
            )).expect("Falied to insert animation when creating rod.");
        },
        ElementalDamageKind::Freezing => {
            let mut freezing = ecs.write_storage::<InflictsFreezingWhenCast>();
            freezing.insert(entity, InflictsFreezingWhenCast (
                InflictsFreezingData {
                    turns: 4,
                }
            )).expect("Failed to insert freezing component when creating rod.");
            let mut animation = ecs.write_storage::<AlongRayAnimationWhenCast>();
            animation.insert(entity, AlongRayAnimationWhenCast (
                AlongRayAnimationData {
                    fg: RGB::named(rltk::WHITE),
                    bg: RGB::named(rltk::LIGHT_BLUE),
                    glyph: rltk::to_cp437('*'),
                    until_blocked: true
                }
            )).expect("Falied to insert animation when creating rod.");
        },
        _ => panic!("Cannot create rod of element {:?}", element)
    }
    Some(entity)
}


pub fn leather_armor(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(']'),
            fg: RGB::named(rltk::SADDLEBROWN),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name : "Leather Armor".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Backpack
        })
        .with(Equippable {slot: EquipmentSlot::Armor})
        .with(GrantsMeleeDefenseBonus {bonus: 2})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}