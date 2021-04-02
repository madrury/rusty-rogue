
use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    EntitySpawnKind, Hazard, Opaque, color, noise
};
use rltk::{RGB};
use specs::prelude::*;


const SHALLOW_WATER_THRESHOLD: f32 = 0.4;
const DEEP_WATER_THRESHOLD: f32 = 0.6;


pub fn spawn_lakes(ecs: &mut World, map: &Map) {
    let water_noise = noise::water_noisemap(map, 0.1);
    for x in 0..map.width {
        for y in 0..map.height {
            let idx = map.xy_idx(x, y);
            let (vnoise, wnoise) = water_noise[idx];
            if vnoise > DEEP_WATER_THRESHOLD {
                let colorseeds = (vnoise + 0.6, 0.7 * vnoise + 0.2 * wnoise + 0.4);
                let fgcolor = color::water_fg_from_noise(colorseeds.0);
                let bgcolor = color::water_bg_from_noise(colorseeds.1);
                deep_water(ecs, x, y, fgcolor, bgcolor);
            } else if vnoise > SHALLOW_WATER_THRESHOLD {
                let colorseeds = (vnoise + 0.4, 0.5 * vnoise + 0.1 * wnoise + 0.4);
                let fgcolor = color::water_fg_from_noise(colorseeds.0);
                let bgcolor = color::shallow_water_bg_from_noise(colorseeds.1);
                shallow_water(ecs, x, y, fgcolor, bgcolor);
            }
        }
    }
}


pub fn shallow_water(ecs: &mut World, x: i32, y: i32, fgcolor: RGB, bgcolor: RGB) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('~'),
            fg: fgcolor,
            bg: bgcolor,
            order: 3,
            visible_out_of_fov: true
        })
        .with(Name {name: "Shallow Water".to_string()})
        .with(Hazard {})
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
            order: 3,
            visible_out_of_fov: true
        })
        .with(Name {name: "Deep Water".to_string()})
        .with(Hazard {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let mut map = ecs.write_resource::<Map>();
    let idx = map.xy_idx(x, y);
    map.ok_to_spawn[idx] = false;
    Some(entity)
}
