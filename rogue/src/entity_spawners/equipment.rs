use super::{
    Name, Position, Renderable, PickUpable, Equippable, EquipmentSlot,
    Throwable, Targeted, TargetingKind, Consumable,
    InflictsDamageWhenTargeted, GrantsMeleeAttackBonus,
    GrantsMeleeDefenseBonus, SimpleMarker, SerializeMe, MarkedBuilder,
    ElementalDamageKind, AlongRayAnimationWhenTargeted
};
use rltk::{RGB};
use specs::prelude::*;

pub fn dagger(ecs: &mut World, x: i32, y: i32)  -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('↑'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            order: 2
        })
        .with(Name {name : "Dagger".to_string()})
        .with(PickUpable {})
        .with(Equippable {slot: EquipmentSlot::Melee})
        .with(GrantsMeleeAttackBonus {bonus: 2})
        .with(Throwable {})
        .with(Targeted {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(Consumable {})
        .with(InflictsDamageWhenTargeted {
            damage: 15,
            kind: ElementalDamageKind::Physical
        })
        .with(AlongRayAnimationWhenTargeted {
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            glyph: rltk::to_cp437('•'),
            until_blocked: true
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

pub fn leather_armor(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(']'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            order: 2
        })
        .with(Name {name : "Leather Armor".to_string()})
        .with(PickUpable {})
        .with(Equippable {slot: EquipmentSlot::Armor})
        .with(GrantsMeleeDefenseBonus {bonus: 2})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}