
use super::{
    Name, Position, Renderable, PickUpable, Useable, Untargeted, Consumable,
    ProvidesFullHealing, ProvidesFullFood, IncreasesMaxHpWhenUsed,
    SimpleMarker, SerializeMe, MarkedBuilder, DissipateWhenBurning, StatusIsImmuneToChill
};
use crate::components::*;
use rltk::{RGB};
use specs::prelude::*;

// The basic food, restores the hunger meter to full.
pub fn turnip(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(';'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Turnip".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Backpack
        })
        .with(Useable {})
        .with(Consumable {})
        .with(Untargeted {verb: "eats".to_string()})
        // TODO: We probably want a component for triggering the healing animation.
        .with(ProvidesFullFood {})
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

// Nom! Resotres both the hunger meter and HP to full.
pub fn pomegranate(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(';'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            order: 2,
            visible_out_of_fov: false
        })
        .with(Name {name: "Pomegranate".to_string()})
        .with(PickUpable {
            destination: PickupDestination::Backpack
        })
        .with(Useable {})
        .with(Consumable {})
        .with(Untargeted {verb: "eats".to_string()})
        // TODO: We probably want a component for triggering the healing animation.
        .with(ProvidesFullFood {})
        .with(ProvidesFullHealing {})
        .with(IncreasesMaxHpWhenUsed {amount: 5})
        .with(DissipateWhenBurning {})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}

