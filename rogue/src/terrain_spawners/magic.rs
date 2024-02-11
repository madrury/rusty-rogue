use super::{
    Map, Position, Renderable, Name, BlessingSelectionTile, SimpleMarker,
    SerializeMe, MarkedBuilder, TileType, StatusIsImmuneToChill,
    StatusIsImmuneToFire, water, color
};
use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;


pub fn spawn_blessing_tile_aparatus(ecs: &mut World) -> Option<Entity> {
    let tile;
    {
        let map = ecs.read_resource::<Map>();
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        tile = map.random_unblocked_point(100, &mut rng);
    }
    match tile {
        Some(tile) => spawn_blessing_tile_aparatus_at_position(ecs, tile.0, tile.1),
        None => None
    }
}

const BLESSING_LAKE_NOISE_FREQUENCY: f32 = 0.25;

fn spawn_blessing_tile_aparatus_at_position(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    // First - spawn the actual blessing tile in the center of the aparatus.
    let mtile = blessing_tile(ecs, x, y);
    {
        let mut map = ecs.write_resource::<Map>();
        let idx = map.xy_idx(x, y);
        map.ok_to_spawn[idx] = false;
    }
    // Compute the coordinates and indexes of all the surrounding tiles.
    let adjacent_tiles: Vec<(i32, i32)>;
    let adjacent_tile_idxs: Vec<usize>;
    let water_noise: Vec<(f32, f32)>;
    {
        let map = ecs.read_resource::<Map>();
        adjacent_tiles = map.get_adjacent_tiles(x, y);
        adjacent_tile_idxs = map.get_adjacent_tiles(x, y).iter()
            .map(|(xx, yy)| map.xy_idx(*xx, *yy))
            .collect();
        // water_noise = noise::water_noisemap(&map, BLESSING_LAKE_NOISE_FREQUENCY);
    }
    // Carve out any walls surrounding the tile spawn position.
    for tile_idx in adjacent_tile_idxs.iter() {
        let mut map = ecs.write_resource::<Map>();
        if map.tiles[*tile_idx] == TileType::Wall {
            map.tiles[*tile_idx] = TileType::Floor;
            map.ok_to_spawn[*tile_idx] = true;
        }
    }
    // Spawn a ring of water around the blessing tile.
    // for (tile, tile_idx) in adjacent_tiles.into_iter().zip(adjacent_tile_idxs) {
    //     let ok_to_spawn;
    //     {
    //         let map = ecs.read_resource::<Map>();
    //         ok_to_spawn = map.ok_to_spawn[tile_idx];
    //     }
    //     if ok_to_spawn {
    //         let (vnoise, wnoise) = water_noise[tile_idx];
    //         let colorseeds = (vnoise + 1.0, 0.7 * vnoise + 0.2 * wnoise + 0.4);
    //         let fgcolor = color::water_fg_from_noise(colorseeds.0);
    //         let bgcolor = color::water_bg_from_noise(colorseeds.1);
    //         water::deep_water(ecs, tile.0, tile.1, fgcolor, bgcolor);
    //         {
    //             let mut map = ecs.write_resource::<Map>();
    //             map.ok_to_spawn[tile_idx] = false;
    //         }
    //     }
    // }
    mtile
}


pub fn blessing_tile(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('☼'),
            fg: RGB::named(rltk::BLACK),
            bg: RGB::named(rltk::GOLD),
            order: 1,
            visible_out_of_fov: true
        })
        .with(BlessingSelectionTile {})
        .with(Name {name: "Tile of Blessing".to_string()})

        .with(StatusIsImmuneToFire {remaining_turns: i32::MAX, render_glyph: false})
        .with(StatusIsImmuneToChill {remaining_turns: i32::MAX, render_glyph: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}