use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    EntitySpawnKind, Hazard, color, noise
};
use rand::seq::SliceRandom;
use rltk::{RGB};
use specs::prelude::*;



const GRASS_NOISE_THRESHOLD: f32 = 0.0;
const SPORADIC_TALL_GRASS_NOISE_THRESHOLD: f32 = 0.8;


// Spawn a patch of short grass.
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
                grass(ecs, x, y, gcolor, RGB::named(rltk::BLACK));
            }
        }
    }
}

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
                    tall_grass(ecs, x, y, gcolor, RGB::named(rltk::BLACK));
                } else {
                    let colorseed = vnoise + 0.3 * wnoise + 0.6;
                    let gcolor = color::grass_green_from_noise(colorseed);
                    grass(ecs, x, y, gcolor, RGB::named(rltk::BLACK));
                }
            }
        }
    }
}


// Grass growing serenely in a tile. Does not do much but offer kindling for
// other effects.
pub fn grass(ecs: &mut World, x: i32, y: i32, fgcolor: RGB, bgcolor: RGB) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('"'),
            fg: fgcolor,
            bg: bgcolor,
            order: 3,
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

// Grass growing serenely in a tile. Does not do much but offer kindling for
// other effects.
pub fn tall_grass(ecs: &mut World, x: i32, y: i32, fgcolor: RGB, bgcolor: RGB) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♣'),
            fg: fgcolor,
            bg: bgcolor,
            order: 3,
        })
        .with(Name {name: "Tall Grass".to_string()})
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
    let mut map = ecs.write_resource::<Map>();
    let idx = map.xy_idx(x, y);
    map.ok_to_spawn[idx] = false;
    Some(entity)
}