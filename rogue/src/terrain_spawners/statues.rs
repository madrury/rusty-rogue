use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, BlocksTile, TileType, noise
};
use rltk::{RGB};
use specs::prelude::*;

const STATUE_NOISE_THRESHOLD: f32 = 0.98;

//----------------------------------------------------------------------------
// Statues.
//----------------------------------------------------------------------------
struct StatueSpawnData {
    x: i32, y: i32
}

// Spawn statues randomly throughout the map.
pub fn spawn_statues(ecs: &mut World) {
    let mut statue_spawn_buffer = Vec::<StatueSpawnData>::new();
    { // Contain the borrow of the ECS.
        let map = ecs.read_resource::<Map>();
        let statue_noise = noise::statue_noisemap(&map);
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                let wnoise = statue_noise[idx];
                if wnoise > STATUE_NOISE_THRESHOLD && map.ok_to_spawn[idx] {
                    statue_spawn_buffer.push(StatueSpawnData {x: x, y: y})
                }
            }
        }
    }
    for data in statue_spawn_buffer {
        statue(ecs, data.x, data.y);
    }
}

// An inanimate statue. Sees nothing, does nothing.
pub fn statue(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    // Statues are immobile and block tiles, so we want to carve out some room
    // around a spawning statue to make sure entities can navigate around them.
    {
        let mut map = ecs.write_resource::<Map>();
        let adjacent_tiles = map.get_adjacent_tiles(x, y);
        for (xadj, yadj) in adjacent_tiles {
            let idx = map.xy_idx(xadj, yadj);
            if map.is_edge_tile(xadj, yadj) {continue;}
            if map.tiles[idx] == TileType::Wall {
                map.tiles[idx] = TileType::Floor;
                map.ok_to_spawn[idx] = true;
            }
        }
        let idx = map.xy_idx(x, y);
        map.ok_to_spawn[idx] = false;
        map.blocked[idx] = true;
    }

    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('&'),
            fg: RGB::named(rltk::GOLD),
            bg: RGB::named(rltk::BLACK),
            order: 1,
            visible_out_of_fov: true
        })
        .with(Name {name: "Statue".to_string()})
        .with(BlocksTile {})
        // Hard to justify? Well, it needs to take a turn ok?
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    Some(entity)
}