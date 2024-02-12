use super::{
    Position, Renderable, Name, BlessingSelectionTile, SimpleMarker,
    SerializeMe, MarkedBuilder, StatusIsImmuneToChill, StatusIsImmuneToFire
};
use rltk::RGB;
use specs::prelude::*;

pub fn blessing_tile(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('☼'),
            fg: RGB::named(rltk::BLACK),
            bg: RGB::named(rltk::GOLD),
            order: 1,
            visible_out_of_fov: true
        })
        .with(BlessingSelectionTile {})
        .with(Name {name: "Tile of Blessing".to_string()})

        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}