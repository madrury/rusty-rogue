use crate::IsEntityKind;

use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    EntitySpawnKind, StatusIsImmuneToChill, Hazard, Opaque
};
use crate::map_builders::GrassSpawnTable;
use rltk::RGB;
use specs::prelude::*;


pub fn spawn_grass_from_table(
    ecs: &mut World,
    grass_spawn_table: &GrassSpawnTable
) {
    for e in &grass_spawn_table.short {
        {
            let map = ecs.read_resource::<Map>();
            let idx = map.xy_idx(e.x, e.y);
            if map.blocked[idx] || map.water[idx] { continue; }
        }
        grass(ecs, e.x, e.y, e.fgcolor);
    }
    for e in &grass_spawn_table.long {
        {
            let map = ecs.read_resource::<Map>();
            let idx = map.xy_idx(e.x, e.y);
            if map.blocked[idx] || map.water[idx] {
                continue;
            }
        }
        tall_grass(ecs, e.x, e.y, e.fgcolor);
    }
}

// Grass growing serenely in a tile. Does not do much but offer kindling for
// other effects.
pub fn grass(ecs: &mut World, x: i32, y: i32, fgcolor: RGB) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('"'),
            fg: fgcolor,
            bg: RGB::named(rltk::BLACK),
            order: 3,
            visible_out_of_fov: true
        })
        .with(Name {name: "Grass".to_string()})
        .with(IsEntityKind {
            kind: EntitySpawnKind::ShortGrass { fg: fgcolor }
        })
        // Hard to justify as a hazard? Well, it needs to take a turn ok?
        .with(Hazard {})
        .with(DissipateWhenBurning {})
        .with(ChanceToSpawnEntityWhenBurning {
            kind: EntitySpawnKind::Fire {
                spread_chance: 50,
                dissipate_chance: 50
            },
            chance: 100
        })
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let mut map = ecs.write_resource::<Map>();
    let idx = map.xy_idx(x, y);
    map.grass[idx] = true;
    Some(entity)
}

// Tall grass growing serenely in a tile.
// Tall grass blocks entities vision (so you can hide behind it). When
// encroached upon, it is replaced by short grass.
pub fn tall_grass(ecs: &mut World, x: i32, y: i32, fgcolor: RGB) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♣'),
            fg: fgcolor,
            bg: RGB::named(rltk::BLACK),
            order: 3,
            visible_out_of_fov: true
        })
        .with(Name {name: "Tall Grass".to_string()})
        .with(IsEntityKind {
            kind: EntitySpawnKind::TallGrass { fg: fgcolor }
        })
        // Hard to justify? Well, it needs to take a turn ok?
        .with(Hazard {})
        .with(Opaque {})
        .with(DissipateWhenBurning {})
        .with(DissipateWhenEnchroachedUpon {})
        .with(SpawnEntityWhenEncroachedUpon {
            chance: 100,
            kind: EntitySpawnKind::ShortGrass {fg: fgcolor}
        })
        .with(ChanceToSpawnEntityWhenBurning {
            kind: EntitySpawnKind::Fire {
                spread_chance: 50,
                dissipate_chance: 50
            },
            chance: 100
        })
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let mut map = ecs.write_resource::<Map>();
    let idx = map.xy_idx(x, y);
    map.opaque[idx] = true;
    map.grass[idx] = true;
    Some(entity)
}

pub fn destroy_grass(ecs: &mut World, entity: &Entity) {
    let idx;
    { // Contain first borrow of ECS.
        let positions = ecs.read_storage::<Position>();
        let map = ecs.fetch::<Map>();
        let pos = positions.get(*entity);
        match pos {
            Some(pos) => {
                idx = map.xy_idx(pos.x, pos.y);
                if !map.grass[idx] {
                    panic!("Attempted to delete short grass but map arrays are unsynchronized.")
                }
            }
            None => panic!("Attempted to delete short grass, but long grass has no position.")
        }
    }
    { // Contain second borrow of ECS.
        let mut map = ecs.fetch_mut::<Map>();
        map.grass[idx] = false;
    }
    ecs.delete_entity(*entity).expect("Unable to remove short grass entity.");
}

pub fn destroy_tall_grass(ecs: &mut World, entity: &Entity) {
    let idx;
    { // Contain first borrow of ECS.
        let positions = ecs.read_storage::<Position>();
        let map = ecs.fetch::<Map>();
        let pos = positions.get(*entity);
        match pos {
            Some(pos) => {
                idx = map.xy_idx(pos.x, pos.y);
                if !map.opaque[idx] | !map.grass[idx] {
                    panic!("Attempted to delete long grass but map arrays are unsynchronized.")
                }
            }
            None => panic!("Attempted to delete long grass, but long grass has no position.")
        }
    }
    { // Contain second borrow of ECS.
        let mut map = ecs.fetch_mut::<Map>();
        map.opaque[idx] = false;
        map.grass[idx] = false;
    }
    ecs.delete_entity(*entity).expect("Unable to remove long grass entity.");
}