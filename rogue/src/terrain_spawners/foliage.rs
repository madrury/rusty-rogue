use super::{
    Map, Position, Renderable, Name, SimpleMarker, SerializeMe,
    MarkedBuilder, DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    EntitySpawnKind, Hazard
};
use rand::seq::SliceRandom;
use rltk::{RGB};
use specs::prelude::*;


// Spawn a patch of short grass.
pub fn spawn_short_grass_patch(ecs: &mut World, map: &Map, region: &[usize]) {
    let idx: Option<&usize> = region.choose(&mut rand::thread_rng());
    if let Some(idx) = idx {
        let patch = grow_patch_from_seed(*idx, map, 80);
        for idx_ in patch {
            let (x, y) = map.idx_xy(idx_);
            let _g = grass(ecs, x, y);
        }
    }
}

// Grow a contiguous patch of tiles outwards from a seed tile.
// TODO: Maybe we should use the noise capabilities of RLTK here?
fn grow_patch_from_seed(seed: usize, map: &Map, n: i32) -> Vec<usize> {
    let mut patch = vec![seed];
    for _ in 0..n {
        let pt: Option<(i32, i32)> = patch.choose(&mut rand::thread_rng())
            .map(|idx| map.idx_xy(*idx));
        if let Some(pt) = pt {
            let next = map.random_adjacent_unblocked_point(pt.0, pt.1);
            if let Some(next) = next {
                let next_idx = map.xy_idx(next.0, next.1);
                if !patch.contains(&next_idx) {
                    patch.push(next_idx);
                }
            }
        }
    }
    patch
}

// Grass growing serenely in a tile. Does not do much but offer kindling for
// other effects.
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