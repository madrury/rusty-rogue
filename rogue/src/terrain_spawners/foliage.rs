use crate::IsEntityKind;

use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    EntitySpawnKind, StatusIsImmuneToChill, Hazard, Opaque, color, noise
};
use rltk::RGB;
use specs::prelude::*;

const GRASS_NOISE_THRESHOLD: f32 = 0.0;
const SPORADIC_TALL_GRASS_NOISE_THRESHOLD: f32 = 0.8;
const GROVE_TALL_GRASS_NOISE_THRESHOLD: f32 = 0.4;

//----------------------------------------------------------------------------
// Foliage.
//
// Functions for spawning grass and other foliage. These provide multiple
// options to support different gameply feels.
//----------------------------------------------------------------------------

struct GrassSpawnData {
    x: i32, y: i32, color: RGB
}

// Spawn a filed of short grass on the map.
pub fn spawn_short_grass(ecs: &mut World) {
    let mut grass_spawn_buffer = Vec::<GrassSpawnData>::new();
    { // Contain the borrow of the ECS.
        let map = ecs.read_resource::<Map>();
        let grass_noise = noise::grass_noisemap(&map);
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                let (vnoise, wnoise) = grass_noise[idx];
                if vnoise > GRASS_NOISE_THRESHOLD && map.ok_to_spawn[idx] {
                    // Trial and error.
                    let colorseed = vnoise + 0.3 * wnoise + 0.6;
                    let gcolor = color::grass_green_from_noise(colorseed);
                    grass_spawn_buffer.push(GrassSpawnData {x: x, y: y, color: gcolor})
                }
            }
        }
    }
    for data in grass_spawn_buffer {
        grass(ecs, data.x, data.y, data.color);
    }
}

// Spawn a filed of short grass with spradically placed tall grass.
pub fn spawn_sporadic_grass(ecs: &mut World) {
    let mut short_grass_spawn_buffer = Vec::<GrassSpawnData>::new();
    let mut tall_grass_spawn_buffer = Vec::<GrassSpawnData>::new();
    { // Contain the borrow of the ECS.
        let map = ecs.read_resource::<Map>();
        let grass_noise = noise::grass_noisemap(&map);
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                let (vnoise, wnoise) = grass_noise[idx];
                if vnoise > GRASS_NOISE_THRESHOLD && map.ok_to_spawn[idx] {
                    if wnoise > SPORADIC_TALL_GRASS_NOISE_THRESHOLD {
                        let colorseed = vnoise + 0.3 * wnoise;
                        let gcolor = color::grass_green_from_noise(colorseed);
                        tall_grass_spawn_buffer.push(GrassSpawnData {x: x, y: y, color: gcolor})
                    } else {
                        let colorseed = vnoise + 0.3 * wnoise + 0.6;
                        let gcolor = color::grass_green_from_noise(colorseed);
                        short_grass_spawn_buffer.push(GrassSpawnData {x: x, y: y, color: gcolor})
                    }
                }
            }
        }
    }
    for data in short_grass_spawn_buffer {
        grass(ecs, data.x, data.y, data.color);
    }
    for data in tall_grass_spawn_buffer {
        tall_grass(ecs, data.x, data.y, data.color);
    }
}

// Spawn a filed of short grass with tall grass placed in the densest areas.
pub fn spawn_grove_grass(ecs: &mut World) {
    let mut short_grass_spawn_buffer = Vec::<GrassSpawnData>::new();
    let mut tall_grass_spawn_buffer = Vec::<GrassSpawnData>::new();
    { // Contain the borrow of the ECS.
        let map = ecs.read_resource::<Map>();
        let grass_noise = noise::grass_noisemap(&map);
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                let (vnoise, wnoise) = grass_noise[idx];
                if vnoise > GRASS_NOISE_THRESHOLD && map.ok_to_spawn[idx] {
                    if vnoise > GROVE_TALL_GRASS_NOISE_THRESHOLD {
                        let colorseed = vnoise + 0.3 * wnoise;
                        let gcolor = color::grass_green_from_noise(colorseed);
                        tall_grass_spawn_buffer.push(GrassSpawnData {x: x, y: y, color: gcolor})
                    } else {
                        let colorseed = vnoise + 0.3 * wnoise + 0.6;
                        let gcolor = color::grass_green_from_noise(colorseed);
                        short_grass_spawn_buffer.push(GrassSpawnData {x: x, y: y, color: gcolor})
                    }
                }
            }
        }
    }
    for data in short_grass_spawn_buffer {
        grass(ecs, data.x, data.y, data.color);
    }
    for data in tall_grass_spawn_buffer {
        tall_grass(ecs, data.x, data.y, data.color);
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