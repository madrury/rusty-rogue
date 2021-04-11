use super::{
    EntitySpawnKind, Name, Position, Renderable, PickUpable, Useable,
    Throwable, Targeted, TargetingKind, Untargeted, Consumable,
    ProvidesFullHealing, IncreasesMaxHpWhenUsed, InflictsDamageWhenTargeted,
    InflictsFreezingWhenTargeted, InflictsBurningWhenTargeted,
    AreaOfEffectAnimationWhenTargeted, MovesToRandomPosition,
    SpawnsEntityInAreaWhenTargeted, ProvidesFireImmunityWhenUsed,
    ProvidesChillImmunityWhenUsed, ProvidesFullSpellRecharge,
    DecreasesSpellRechargeWhenUsed, SimpleMarker, SerializeMe, MarkedBuilder,
    ElementalDamageKind
};
use rltk::{RGB};
use specs::prelude::*;

pub fn health(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Healing".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted {verb: "drinks".to_string()})
        .with(Throwable {})
        .with(Targeted {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(Consumable {})
        // TODO: We probably want a component for triggering the healing
        // animation.
        .with(ProvidesFullHealing {})
        .with(IncreasesMaxHpWhenUsed {amount: 5})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

pub fn recharging(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Recharging".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted {verb: "drinks".to_string()})
        .with(Consumable {})
        .with(ProvidesFullSpellRecharge {})
        .with(DecreasesSpellRechargeWhenUsed {percentage: 25})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

pub fn fire(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Fire".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted {verb: "drinks".to_string()})
        .with(Throwable {})
        .with(Targeted {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::AreaOfEffect {radius: 2.5}
        })
        .with(Consumable {})
        .with(ProvidesFireImmunityWhenUsed {turns: 50})
        .with(InflictsDamageWhenTargeted {
            damage: 10,
            kind: ElementalDamageKind::Fire
        })
        .with(InflictsBurningWhenTargeted {
            turns: 5,
            tick_damage: 2
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

pub fn freezing(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Freezing".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted {verb: "drinks".to_string()})
        .with(Throwable {})
        .with(Targeted {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::AreaOfEffect {radius: 2.5}
        })
        .with(Consumable {})
        .with(ProvidesChillImmunityWhenUsed {turns: 50})
        .with(InflictsDamageWhenTargeted {
            damage: 10,
            kind: ElementalDamageKind::Chill
        })
        .with(InflictsFreezingWhenTargeted {turns: 6})
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

pub fn teleportation(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::MEDIUM_PURPLE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Potion of Teleportation".to_string()})
        .with(PickUpable {})
        .with(Useable {})
        .with(Untargeted{ verb: "drinks".to_string()})
        .with(Throwable {})
        .with(Targeted {
            verb: "throws".to_string(),
            range: 6.5,
            kind: TargetingKind::Simple
        })
        .with(Consumable {})
        .with(MovesToRandomPosition {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}