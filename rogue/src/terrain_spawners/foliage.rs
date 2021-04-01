use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    EntitySpawnKind, Hazard, Opaque, color, noise
};
use rand::seq::SliceRandom;
use rltk::{RGB};
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

// Spawn a filed of short grass on the map.
pub fn spawn_short_grass(ecs: &mut World, map: &Map) {
    let grass_noise = noise::grass_noisemap(map);
    for x in 0..map.width {
        for y in 0..map.height {
            let idx = map.xy_idx(x, y);
            let (vnoise, wnoise) = grass_noise[idx];
            if vnoise > GRASS_NOISE_THRESHOLD && map.ok_to_spawn[idx] {
                // Trial and error.
                let colorseed = vnoise + 0.3 * wnoise + 0.6;
                let gcolor = color::grass_green_from_noise(colorseed);
                grass(ecs, x, y, gcolor);
            }
        }
    }
}

// Spawn a filed of short grass with spradically placed tall grass.
pub fn spawn_sporadic_grass(ecs: &mut World, map: &Map) {
    let grass_noise = noise::grass_noisemap(map);
    for x in 0..map.width {
        for y in 0..map.height {
            let idx = map.xy_idx(x, y);
            let (vnoise, wnoise) = grass_noise[idx];
            if vnoise > GRASS_NOISE_THRESHOLD && map.ok_to_spawn[idx] {
                if wnoise > SPORADIC_TALL_GRASS_NOISE_THRESHOLD {
                    let colorseed = vnoise + 0.3 * wnoise;
                    let gcolor = color::grass_green_from_noise(colorseed);
                    tall_grass(ecs, x, y, gcolor);
                } else {
                    let colorseed = vnoise + 0.3 * wnoise + 0.6;
                    let gcolor = color::grass_green_from_noise(colorseed);
                    grass(ecs, x, y, gcolor);
                }
            }
        }
    }
}

// Spawn a filed of short grass with tall grass placed in the densest areas.
pub fn spawn_grove_grass(ecs: &mut World, map: &Map) {
    let grass_noise = noise::grass_noisemap(map);
    for x in 0..map.width {
        for y in 0..map.height {
            let idx = map.xy_idx(x, y);
            let (vnoise, wnoise) = grass_noise[idx];
            if vnoise > GRASS_NOISE_THRESHOLD && map.ok_to_spawn[idx] {
                if vnoise > GROVE_TALL_GRASS_NOISE_THRESHOLD {
                    let colorseed = vnoise + 0.3 * wnoise;
                    let gcolor = color::grass_green_from_noise(colorseed);
                    tall_grass(ecs, x, y, gcolor);
                } else {
                    let colorseed = vnoise + 0.3 * wnoise + 0.6;
                    let gcolor = color::grass_green_from_noise(colorseed);
                    grass(ecs, x, y, gcolor);
                }
            }
        }
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
        // Hard to justify? Well, it needs to take a turn ok?
        .with(Hazard {})
        .with(DissipateWhenBurning {})
        .with(ChanceToSpawnEntityWhenBurning {
            kind: EntitySpawnKind::Fire {
                spread_chance: 50,
                dissipate_chance: 50
            },
            chance: 100
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
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
        // Hard to justify? Well, it needs to take a turn ok?
        .with(Hazard {})
        .with(Opaque {})
        .with(DissipateWhenBurning {})
        .with(DissipateWhenEnchroachedUpon {})
        .with(SpawnEntityWhenEncroachedUpon {
            chance: 100,
            kind: EntitySpawnKind::Grass {fg: fgcolor}
        })
        .with(ChanceToSpawnEntityWhenBurning {
            kind: EntitySpawnKind::Fire {
                spread_chance: 50,
                dissipate_chance: 50
            },
            chance: 100
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let mut map = ecs.write_resource::<Map>();
    let idx = map.xy_idx(x, y);
    map.ok_to_spawn[idx] = false;
    map.opaque[idx] = true;
    Some(entity)
}

pub fn destroy_long_grass(ecs: &mut World, entity: &Entity) {
    let idx;
    { // Contain first borrow of ECS.
        let positions = ecs.read_storage::<Position>();
        let map = ecs.fetch::<Map>();
        let pos = positions.get(*entity);
        match pos {
            Some(pos) => {
                idx = map.xy_idx(pos.x, pos.y);
                if !map.opaque[idx] {
                    panic!("Attempted to delete long grass but map arrays are unsynchronized.")
                }
            }
            None => panic!("Attempted to delete long grass, but long grass has no position.")
        }
    }
    { // Contain second borrow of ECS.
        let mut map = ecs.fetch_mut::<Map>();
        map.opaque[idx] = false;
    }
    ecs.delete_entity(*entity).expect("Unable to remove long_grass entity.");
}