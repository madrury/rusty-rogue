
use super::{
    Map, Position, Renderable, SetsBgColor, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, Hazard, ChanceToSpawnEntityWhenBurning,
    RemoveBurningWhenEncroachedUpon, RemoveBurningOnUpkeep,
    DissipateFireWhenEncroachedUpon, EntitySpawnKind, StatusIsImmuneToChill,
    IsEntityKind
};
use crate::map_builders::WaterSpawnTable;

use rltk::RGB;
use specs::prelude::*;


pub fn spawn_water_from_table(
    ecs: &mut World,
    water_spawn_table: &WaterSpawnTable
) {
    for e in &water_spawn_table.shallow {
        {
            let mut map = ecs.write_resource::<Map>();
            let idx = map.xy_idx(e.x, e.y);
            if map.blocked[idx] {
                continue;
            }
            map.ok_to_spawn[idx] = true;
        }
        shallow_water(ecs, e.x, e.y, e.fgcolor, e.bgcolor);
    }
    for e in &water_spawn_table.deep {
        {
            let map = ecs.read_resource::<Map>();
            let idx = map.xy_idx(e.x, e.y);
            if map.blocked[idx] {
                continue;
            }
        }
        deep_water(ecs, e.x, e.y, e.fgcolor, e.bgcolor);
    }
}

pub fn shallow_water(ecs: &mut World, x: i32, y: i32, fgcolor: RGB, bgcolor: RGB) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('~'),
            fg: fgcolor,
            bg: bgcolor,
            order: 4,
            visible_out_of_fov: true
        })
        .with(SetsBgColor {order: 2})
        .with(Name {name: "Shallow Water".to_string()})
        .with(Hazard {})
        .with(ChanceToSpawnEntityWhenBurning {
            kind: EntitySpawnKind::Steam {
                spread_chance: 75,
                dissipate_chance: 20
            },
            chance: 100
        })
        .with(RemoveBurningWhenEncroachedUpon {})
        .with(RemoveBurningOnUpkeep {})
        .with(DissipateFireWhenEncroachedUpon {})
        // TODO: Clearly water should freeze when chilled.
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}

pub fn deep_water(ecs: &mut World, x: i32, y: i32, fgcolor: RGB, bgcolor: RGB) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('~'),
            fg: fgcolor,
            bg: bgcolor,
            order: 4,
            visible_out_of_fov: true
        })
        .with(SetsBgColor {order: 2})
        .with(Name {name: "Deep Water".to_string()})
        .with(Hazard {})
        .with(IsEntityKind {kind: EntitySpawnKind::DeepWater})
        .with(RemoveBurningWhenEncroachedUpon {})
        .with(RemoveBurningOnUpkeep {})
        .with(DissipateFireWhenEncroachedUpon {})
        // TODO: Clearly water should freeze when chilled.
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let mut map = ecs.write_resource::<Map>();
    let idx = map.xy_idx(x, y);
    map.ok_to_spawn[idx] = false;
    map.deep_water[idx] = true;
    Some(entity)
}
