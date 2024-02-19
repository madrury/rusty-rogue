use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, BlocksTile, TileType, StatusIsImmuneToChill,
    StatusIsImmuneToFire
};
use crate::map_builders::StatueSpawnData;
use rltk::RGB;
use specs::prelude::*;

pub fn spawn_statues_from_table(
    ecs: &mut World,
    statue_spawn_table: &Vec<StatueSpawnData>
) {
    for e in statue_spawn_table {
        {
            let map = ecs.read_resource::<Map>();
            let idx = map.xy_idx(e.x, e.y);
            if map.blocked[idx] || map.deep_water[idx] { continue; }
        }
        statue(ecs, e.x, e.y);
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
        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    Some(entity)
}