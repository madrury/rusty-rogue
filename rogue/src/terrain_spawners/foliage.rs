use super::{
    Map, TileType, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, MAP_WIDTH
};
use rand::seq::SliceRandom;
use rltk::{RGB};
use specs::prelude::*;


pub fn spawn_short_grass_patch(ecs: &mut World, map: &Map, region: &[usize]) {
    let idx: Option<&usize> = region.choose(&mut rand::thread_rng());
    if let Some(idx) = idx {
        let patch = grow_patch_from_seed(*idx, map, 40);
        for idx_ in patch {
            let (x, y) = map.idx_xy(idx_);
            let _g = grass(ecs, x, y);
        }
    }
}

fn grow_patch_from_seed(seed: usize, map: &Map, N: i32) -> Vec<usize> {
    let mut patch = vec![seed];
    for _ in 0..N {
        let pt: Option<(i32, i32)> = patch.choose(&mut rand::thread_rng())
            .map(|idx| map.idx_xy(*idx));
        if let Some(pt) = pt {
            let next = map.random_adjacent_unblocked_point(pt.0, pt.1);
            if let Some(next) = next {
                patch.push(map.xy_idx(next.0, next.1));
            }
        }
    }
    patch
}

pub fn grass(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let entity = ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('"'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            order: 2,
        })
        .with(Name {name: "Grass".to_string()})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Some(entity)
}