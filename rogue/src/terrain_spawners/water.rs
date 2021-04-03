
use super::{
    Map, Position, Renderable, SetsBgColor, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, Hazard, ChanceToSpawnEntityWhenBurning, EntitySpawnKind,
    TileType, color, noise
};
use rltk::{RGB};
use specs::prelude::*;


const SHALLOW_WATER_THRESHOLD: f32 = 0.4;
const DEEP_WATER_THRESHOLD: f32 = 0.6;

struct WaterSpawnData {
    x: i32, y: i32, fgcolor: RGB, bgcolor: RGB
}

pub fn spawn_lakes(ecs: &mut World) {
    let mut shallow_water_spawn_buffer = Vec::<WaterSpawnData>::new();
    let mut deep_water_spawn_buffer = Vec::<WaterSpawnData>::new();
    { // Contain the borrow of the ECS.
        let map = ecs.read_resource::<Map>();
        let water_noise = noise::water_noisemap(&map, 0.1);
        for x in 0..map.width {
            for y in 0..map.height {
                if is_edge_tile(&map, x, y) {continue;}
                let idx = map.xy_idx(x, y);
                let (vnoise, wnoise) = water_noise[idx];
                if vnoise > DEEP_WATER_THRESHOLD {
                    let colorseeds = (vnoise + 0.6, 0.7 * vnoise + 0.2 * wnoise + 0.4);
                    let fgcolor = color::water_fg_from_noise(colorseeds.0);
                    let bgcolor = color::water_bg_from_noise(colorseeds.1);
                    deep_water_spawn_buffer.push(WaterSpawnData {
                        x: x, y: y, fgcolor: fgcolor, bgcolor: bgcolor
                    })
                } else if vnoise > SHALLOW_WATER_THRESHOLD {
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
    // Clear the buffers.
    for data in shallow_water_spawn_buffer {
        shallow_water(ecs, data.x, data.y, data.fgcolor, data.bgcolor);
        {
            let mut map = ecs.write_resource::<Map>();
            let idx = map.xy_idx(data.x, data.y);
            map.tiles[idx] = TileType::Floor;
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

fn is_edge_tile(map: &Map, x: i32, y: i32) -> bool {
    x == 0 || x == map.width - 1 || y == 0 || y == map.height - 1
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
        .with(SetsBgColor {order: 0})
        .with(Name {name: "Shallow Water".to_string()})
        .with(Hazard {})
        .with(ChanceToSpawnEntityWhenBurning {
            kind: EntitySpawnKind::Steam {
                spread_chance: 75,
                dissipate_chance: 20
            },
            chance: 100
        })
        // TODO: Component: ImmediatleyClearsBurning
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
        .with(SetsBgColor {order: 0})
        .with(Name {name: "Deep Water".to_string()})
        .with(Hazard {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let mut map = ecs.write_resource::<Map>();
    let idx = map.xy_idx(x, y);
    map.ok_to_spawn[idx] = false;
    map.water[idx] = true;
    Some(entity)
}
