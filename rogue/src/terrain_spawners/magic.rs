use super::{
    Map, Position, Renderable, Name, BlessingSelectionTile, SimpleMarker,
    SerializeMe, MarkedBuilder, statues, water
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


fn spawn_blessing_tile_aparatus_at_position(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    statues::statue(ecs, x, y);
    let adjacent_tiles: Vec<(i32, i32)>;
    let adjacent_tile_idxs: Vec<usize>;
    let magic_tile_vec_idx;
    {
        let map = ecs.read_resource::<Map>();
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        adjacent_tiles = map.get_adjacent_tiles(x, y);
        adjacent_tile_idxs = map.get_adjacent_tiles(x, y).iter()
            .map(|(xx, yy)| map.xy_idx(*xx, *yy))
            .collect();
        let n_adjacent_tiles = adjacent_tile_idxs.len();
        // Which tile in the adjacent_tile_idx's vector should be the magic
        // tile.
        magic_tile_vec_idx = rng.roll_dice(1, n_adjacent_tiles as i32) - 1;
    }
    let mut mtile = None;
    for (i, (tile, tile_idx)) in adjacent_tiles.into_iter().zip(adjacent_tile_idxs).enumerate() {
        let ok_to_spawn;
        {
            let map = ecs.read_resource::<Map>();
            ok_to_spawn = map.ok_to_spawn[tile_idx];
        }
        if i == magic_tile_vec_idx as usize && ok_to_spawn{
            mtile = blessing_tile(ecs, tile.0, tile.1);
            {
                let mut map = ecs.write_resource::<Map>();
                map.ok_to_spawn[tile_idx] = false;
            }
        } else if ok_to_spawn {
            water::shallow_water(ecs, tile.0, tile.1, RGB::named(rltk::ROYALBLUE), RGB::named(rltk::BLUE));
            {
                let mut map = ecs.write_resource::<Map>();
                map.ok_to_spawn[tile_idx] = false;
            }
        }
    }
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}