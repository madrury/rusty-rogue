use crate::components::*;
use crate::components::status_effects::*;

use super::{
    Name, Position, Renderable, PickUpable, SimpleMarker, SerializeMe,
    MarkedBuilder, BlessingOrb
};

use rltk::{RGB};
use specs::prelude::*;


const ORB_RENDER_ORDER: i32 = 2;

pub fn blessing_orb(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('•'),
            fg: RGB::named(rltk::AQUAMARINE),
            bg: RGB::named(rltk::BLACK),
            order: ORB_RENDER_ORDER,
            visible_out_of_fov: false
        })
        .with(Name {name: "Orb of Blessing".to_string()})
        .with(PickUpable {
            destination: PickupDestination::BlessingOrbBag
        })
        .with(BlessingOrb {})
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
        Some(entity)
}