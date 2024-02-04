
use super::{
    Map, Position, Renderable, SetsBgColor, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, Hazard, ChanceToSpawnEntityWhenBurning,
    RemoveBurningWhenEncroachedUpon, RemoveBurningOnUpkeep,
    DissipateFireWhenEncroachedUpon, EntitySpawnKind, StatusIsImmuneToChill,
    IsEntityKind, TileType, color, noise
};
use rltk::{RGB};
use specs::prelude::*;


const LARGE_LAKES_SHALLOW_WATER_THRESHOLD: f32 = 0.4;
const LARGE_LAKES_DEEP_WATER_THRESHOLD: f32 = 0.6;
const LARGE_LAKES_NOISE_FREQUENCY: f32 = 0.1;
const SMALL_LAKES_SHALLOW_WATER_THRESHOLD: f32 = 0.6;
const SMALL_LAKES_DEEP_WATER_THRESHOLD: f32 = 0.7;
const SMALL_LAKES_NOISE_FREQUENCY: f32 = 0.25;

struct WaterSpawnData {
    x: i32, y: i32, fgcolor: RGB, bgcolor: RGB
}

pub fn spawn_large_lakes(ecs: &mut World) {
    spawn_bodies_of_water(
        ecs,
        LARGE_LAKES_DEEP_WATER_THRESHOLD,
        LARGE_LAKES_SHALLOW_WATER_THRESHOLD,
        LARGE_LAKES_NOISE_FREQUENCY
    )
}

pub fn spawn_small_lakes(ecs: &mut World) {
    spawn_bodies_of_water(
        ecs,
        SMALL_LAKES_DEEP_WATER_THRESHOLD,
        SMALL_LAKES_SHALLOW_WATER_THRESHOLD,
        SMALL_LAKES_NOISE_FREQUENCY
    )
}

// Generic function for spawning lake like bodies of water.
fn spawn_bodies_of_water(
    ecs: &mut World,
    deep_water_threshold: f32,
    shallow_water_threshold: f32,
    frequency: f32
) {
    let mut shallow_water_spawn_buffer = Vec::<WaterSpawnData>::new();
    let mut deep_water_spawn_buffer = Vec::<WaterSpawnData>::new();
    { // Contain the borrow of the ECS.
        let map = ecs.read_resource::<Map>();
        let water_noise = noise::water_noisemap(&map, frequency);
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                // Guard against spawning water on the edge of the map, or over the DownStairs.
                // Note that we neglect to check the ok_to_spawn array here,
                // since we want water to be able to carve out walls
                if map.is_edge_tile(x, y) {continue;}
                if map.tiles[idx] == TileType::DownStairs {continue;}
                let (vnoise, wnoise) = water_noise[idx];
                if vnoise > deep_water_threshold {
                    let colorseeds = (vnoise + 0.6, 0.7 * vnoise + 0.2 * wnoise + 0.4);
                    let fgcolor = color::water_fg_from_noise(colorseeds.0);
                    let bgcolor = color::water_bg_from_noise(colorseeds.1);
                    deep_water_spawn_buffer.push(WaterSpawnData {
                        x: x, y: y, fgcolor: fgcolor, bgcolor: bgcolor
                    })
                } else if vnoise > shallow_water_threshold {
                    let colorseeds = (vnoise + 0.4, 0.5 * vnoise + 0.1 * wnoise + 0.4);
                    let fgcolor = color::water_fg_from_noise(colorseeds.0);
                    let bgcolor = color::shallow_water_bg_from_noise(colorseeds.1);
                    shallow_water_spawn_buffer.push(WaterSpawnData {
                        x: x, y: y, fgcolor: fgcolor, bgcolor: bgcolor
                    })
                }
            }
        }
    }
    // Actually spawn.
    for data in shallow_water_spawn_buffer {
        shallow_water(ecs, data.x, data.y, data.fgcolor, data.bgcolor);
        {
            let mut map = ecs.write_resource::<Map>();
            let idx = map.xy_idx(data.x, data.y);
            map.tiles[idx] = TileType::Floor;
            map.ok_to_spawn[idx] = true;
        }
    }
    for data in deep_water_spawn_buffer {
        deep_water(ecs, data.x, data.y, data.fgcolor, data.bgcolor);
        {
            let mut map = ecs.write_resource::<Map>();
            let idx = map.xy_idx(data.x, data.y);
            map.tiles[idx] = TileType::Floor;
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
        .with(IsEntityKind {kind: EntitySpawnKind::Water})
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
    map.water[idx] = true;
    Some(entity)
}
