use super::{
    Name, Position, Renderable, PickUpable, Equippable, EquipmentSlot,
    Throwable, Targeted, TargetingKind, Consumable,
    InflictsDamageWhenTargeted, GrantsMeleeAttackBonus,
    GrantsMeleeDefenseBonus, SimpleMarker, SerializeMe, MarkedBuilder,
    ElementalDamageKind, AlongRayAnimationWhenTargeted, StatusIsImmuneToChill, StatusIsImmuneToFire, WeaponSpecial, WeaponSpecialKind
};
use rltk::RGB;
use specs::prelude::*;

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
        .with(PickUpable {})
        .with(Throwable {})
        .with(Consumable {})
        .with(Name {name : "Dagger".to_string()})
        .with(Equippable {slot: EquipmentSlot::Melee})
        .with(GrantsMeleeAttackBonus {bonus: 2})
        .with(Targeted {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenTargeted {
            damage: 15,
            kind: ElementalDamageKind::Physical
        })
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
            glyph: rltk::to_cp437('↑'),
            until_blocked: true
        })
        .with(WeaponSpecial {
            regen_time: 50,
            time: 0,
            kind: WeaponSpecialKind::ThrowWithoutExpending
        })
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
        .with(PickUpable {})
        .with(Throwable {})
        .with(Consumable {})
        .with(Name {name : "Sword".to_string()})
        .with(Equippable {slot: EquipmentSlot::Melee})
        .with(GrantsMeleeAttackBonus {bonus: 4})
        .with(Targeted {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(InflictsDamageWhenTargeted {
            damage: 30,
            kind: ElementalDamageKind::Physical
        })
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
            glyph: rltk::to_cp437('↑'),
            until_blocked: true
        })
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
        .with(PickUpable {})
        .with(Equippable {slot: EquipmentSlot::Armor})
        .with(GrantsMeleeDefenseBonus {bonus: 2})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}