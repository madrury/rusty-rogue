use crate::components::*;
use crate::components::animation::*;
use crate::components::magic::*;
use crate::components::game_effects::*;
use crate::components::melee::*;
use crate::components::equipment::*;
use crate::components::signaling::*;
use crate::components::status_effects::*;
use crate::components::spawn_despawn::*;
use crate::components::targeting::*;
use crate::entity_spawners::spells::{
    FIREBALL_RANGE, FIREBALL_DAMAGE, FIREBALL_BURNING_TURNS,
    FIREBALL_BURNING_TICK_DAMAGE, FIREBLAST_FIRE_DISSIPATE_CHANCE,
    FIREBLAST_FIRE_SPREAD_CHANCE, ICESPIKE_DAMAGE, ICESPIKE_RANGE,
    ICESPIKE_FREEZING_TURNS, ICEBLAST_CHILL_SPREAD_CHANCE,
    ICEBLAST_CHILL_DISSIPATE_CHANCE
};

use rltk::RGB;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};


const SWORD_GLYPH: u16 = 160;
const ROD_GLYPH: u16 = 161;
const DAGGER_GLYPH: u16 = 162;
const SPEAR_GLYPH: u16 = 163;
const RAPIER_GLYPH: u16 = 164;

const DAGGER_MEELE_ATTACK_POWER: i32 = 2;
const DAGGER_THROW_RANGE: f32 = 10.0;
const DAGGER_THROW_DAMAGE: i32 = 15;
const DAGGER_SPECIAL_REGEN_TURNS: i32 = 100;

pub fn dagger(ecs: &mut World, x: i32, y: i32)  -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: DAGGER_GLYPH,
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
            bonus: DAGGER_MEELE_ATTACK_POWER,
            formation: MeeleAttackFormation::Basic,
            element: ElementalDamageKind::Physical
        })
        .with(TargetedWhenThrown {
            range: DAGGER_THROW_RANGE,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenThrown(
            InflictsDamageData {
                damage: DAGGER_THROW_DAMAGE,
                element: ElementalDamageKind::Physical
            }
        ))
        .with(AnimationWhenThrown {
            sequence: vec![
                AnimationComponentData::AlongRay {
                    fg: RGB::named(rltk::SILVER),
                    bg: None,
                    glyph: DAGGER_GLYPH,
                    until_blocked: true
                }
            ]
        })
        .with(WeaponSpecial {
            regen_time: DAGGER_SPECIAL_REGEN_TURNS,
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


const SWORD_MEELE_ATTACK_POWER: i32 = 5;
const SWORD_THROW_RANGE: f32 = 6.5;
const SWORD_THROW_DAMAGE: i32 = 30;
const SWORD_SPECIAL_REGEN_TURNS: i32 = 200;

pub fn sword(ecs: &mut World, x: i32, y: i32)  -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: SWORD_GLYPH,
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
            bonus: SWORD_MEELE_ATTACK_POWER,
            formation: MeeleAttackFormation::Basic,
            element: ElementalDamageKind::Physical
        })
        .with(TargetedWhenThrown {
            range: SWORD_THROW_RANGE,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenThrown(
            InflictsDamageData {
                damage: SWORD_THROW_DAMAGE,
                element: ElementalDamageKind::Physical
            }
        ))
        .with(AnimationWhenThrown {
            sequence: vec![
                AnimationComponentData::AlongRay {
                    fg: RGB::named(rltk::SILVER),
                    bg: None,
                    glyph: SWORD_GLYPH,
                    until_blocked: true
                }
            ]
        })
        .with(WeaponSpecial {
            regen_time: SWORD_SPECIAL_REGEN_TURNS,
            time: 0,
            kind: WeaponSpecialKind::SpinAttack
        })
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}


const RAPIER_MEELE_ATTACK_POWER: i32 = 4;
const RAPIER_THROW_RANGE: f32 = 6.5;
const RAPIER_THROW_DAMAGE: i32 = 25;
const RAPIER_SPECIAL_REGEN_TURNS: i32 = 150;

pub fn rapier(ecs: &mut World, x: i32, y: i32)  -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: RAPIER_GLYPH,
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
            bonus: RAPIER_MEELE_ATTACK_POWER,
            formation: MeeleAttackFormation::Dash,
            element: ElementalDamageKind::Physical
        })
        .with(TargetedWhenThrown {
            range: RAPIER_THROW_RANGE,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenThrown (
            InflictsDamageData {
                damage: RAPIER_THROW_DAMAGE,
                element: ElementalDamageKind::Physical
            }
        ))
        .with(AnimationWhenThrown {
            sequence: vec![
                AnimationComponentData::AlongRay {
                    fg: RGB::named(rltk::SILVER),
                    bg: None,
                    glyph: RAPIER_GLYPH,
                    until_blocked: true
                }
            ]
        })
        .with(WeaponSpecial {
            regen_time: RAPIER_SPECIAL_REGEN_TURNS,
            time: 0,
            kind: WeaponSpecialKind::Dash
        })
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

const ROD_MEELE_ATTACK_POWER: i32 = 0;
const ROD_THROW_RANGE: f32 = 6.5;
const ROD_THROW_AOE_RANGE: f32 = 1.0;
const ROD_THROW_DAMAGE: i32 = 10;
const ROD_SPECIAL_REGEN_TURNS: i32 = 150;

pub fn rod(ecs: &mut World, x: i32, y: i32, element: ElementalDamageKind)  -> Option<Entity> {
    let fg = match element {
        ElementalDamageKind::Fire => RGB::named(rltk::ORANGE),
        ElementalDamageKind::Freezing => RGB::named(rltk::LIGHTBLUE),
        _ => RGB::named(rltk::SILVER)
    };
    let name = match element {
        ElementalDamageKind::Fire => "Fire Rod".to_string(),
        ElementalDamageKind::Freezing => "Freezing Rod".to_string(),
        _ => "Rod".to_string()
    };
    let (cast_damage, cast_range) = match element {
        ElementalDamageKind::Fire => (FIREBALL_DAMAGE, FIREBALL_RANGE),
        ElementalDamageKind::Freezing => (ICESPIKE_DAMAGE, ICESPIKE_RANGE),
        _ => (10, 10.0)
    };
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: ROD_GLYPH,
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
        .with(Name {name})
        .with(IncresesSpellRechargeRateWhenEquipped {element})
        .with(MeeleAttackWepon {
            bonus: ROD_MEELE_ATTACK_POWER,
            formation: MeeleAttackFormation::Basic,
            element: element
        })
        .with(TargetedWhenThrown {
            range: ROD_THROW_RANGE,
            kind: TargetingKind::AreaOfEffect {
                radius: ROD_THROW_AOE_RANGE
            }
        })
        .with(InflictsDamageWhenThrown(
            InflictsDamageData {
                damage: ROD_THROW_DAMAGE,
                element
            }
        ))
        .with(WeaponSpecial {
            regen_time: ROD_SPECIAL_REGEN_TURNS,
            time: 0,
            kind: WeaponSpecialKind::AddToSpellBook
        })
        .with(ExpendWeaponSpecialWhenCast {})
        .with(TargetedWhenCast {
            range: cast_range,
            kind: TargetingKind::AlongRay {until_blocked: true}
        })
        .with(InflictsDamageWhenCast (
            InflictsDamageData {
                damage: cast_damage,
                element
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
                    turns: FIREBALL_BURNING_TURNS,
                    tick_damage: FIREBALL_BURNING_TICK_DAMAGE
                }
            )).expect("Failed to insert burning component when creating rod.");

            let mut castanimation = ecs.write_storage::<AnimationWhenCast>();
            castanimation.insert(entity, AnimationWhenCast {
                sequence: vec![
                    AnimationComponentData::AlongRay {
                        fg: RGB::named(rltk::ORANGE),
                        bg: Some(RGB::named(rltk::RED)),
                        glyph: rltk::to_cp437('^'),
                        until_blocked: true
                    },
                ]
            }).expect("Falied to insert cast animation when creating rod.");
            let mut throwanimation = ecs.write_storage::<AnimationWhenThrown>();
            throwanimation.insert(entity, AnimationWhenThrown {
                sequence: vec![
                    AnimationComponentData::AreaOfEffect {
                        radius: ROD_THROW_AOE_RANGE,
                        fg: RGB::named(rltk::ORANGE),
                        bg: RGB::named(rltk::RED),
                        glyph: rltk::to_cp437('^')
                    },
                    AnimationComponentData::AlongRay {
                        fg: fg,
                        bg: None,
                        glyph: ROD_GLYPH,
                        until_blocked: true
                    },
                ]
            }).expect("Falied to insert throw animation when creating rod.");
            let mut burning = ecs.write_storage::<InflictsBurningWhenThrown>();
            burning.insert(entity, InflictsBurningWhenThrown (
                InflictsBurningData {
                    turns: FIREBALL_BURNING_TURNS,
                    tick_damage: FIREBALL_BURNING_TICK_DAMAGE
                }
            )).expect("Failed to insert burning status when creating rod.");
            let mut spawner = ecs.write_storage::<SpawnsEntityInAreaWhenThrown>();
            spawner.insert(entity, SpawnsEntityInAreaWhenThrown (
                SpawnsEntityInAreaData {
                    radius: ROD_THROW_AOE_RANGE,
                    kind: EntitySpawnKind::Fire {
                        spread_chance: FIREBLAST_FIRE_SPREAD_CHANCE,
                        dissipate_chance: FIREBLAST_FIRE_DISSIPATE_CHANCE,
                    }
                }
            )).expect("Failed to insert fire spawner when creating rod.");
        },
        ElementalDamageKind::Freezing => {
            let mut freezing = ecs.write_storage::<InflictsFreezingWhenCast>();
            freezing.insert(entity, InflictsFreezingWhenCast (
                InflictsFreezingData {
                    turns: ICESPIKE_FREEZING_TURNS,
                }
            )).expect("Failed to insert freezing component when creating rod.");
            let mut castanimation = ecs.write_storage::<AnimationWhenCast>();
            castanimation.insert(entity, AnimationWhenCast {
                sequence: vec![
                    AnimationComponentData::AlongRay {
                        fg: RGB::named(rltk::WHITE),
                        bg: Some(RGB::named(rltk::LIGHT_BLUE)),
                        glyph: rltk::to_cp437('*'),
                        until_blocked: true
                    },
                ]
            }).expect("Falied to insert animation when creating rod.");
            let mut throwanimation = ecs.write_storage::<AnimationWhenThrown>();
            throwanimation.insert(entity, AnimationWhenThrown {
                sequence: vec![
                    AnimationComponentData::AreaOfEffect {
                        radius: ROD_THROW_AOE_RANGE,
                        fg: RGB::named(rltk::WHITE),
                        bg: RGB::named(rltk::LIGHT_BLUE),
                        glyph: rltk::to_cp437('*')
                    },
                    AnimationComponentData::AlongRay {
                        fg: fg,
                        bg: None,
                        glyph: ROD_GLYPH,
                        until_blocked: true
                    },
                ]
            }).expect("Falied to insert animation when creating rod.");
            let mut freezing = ecs.write_storage::<InflictsFreezingWhenThrown>();
            freezing.insert(entity, InflictsFreezingWhenThrown (
                InflictsFreezingData {
                    turns: ICESPIKE_FREEZING_TURNS,
                }
            )).expect("Failed to insert freezing status when creating rod.");
            let mut spawner = ecs.write_storage::<SpawnsEntityInAreaWhenThrown>();
            spawner.insert(entity, SpawnsEntityInAreaWhenThrown (
                SpawnsEntityInAreaData {
                    radius: ROD_THROW_AOE_RANGE,
                    kind: EntitySpawnKind::Chill {
                        spread_chance: ICEBLAST_CHILL_SPREAD_CHANCE,
                        dissipate_chance: ICEBLAST_CHILL_DISSIPATE_CHANCE,
                    }
                }
            )).expect("Failed to insert clill spawner when creating rod.");
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