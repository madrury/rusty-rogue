use std::matches;
use specs::prelude::*;
use super::{
    Map, Position, BlocksTile, Opaque, IsEntityKind, EntitySpawnKind, Bloodied,
    UseFgColorMap, UseFgColorMapWhenBloodied, FgColorMap
};

pub struct MapSynchronizationSystem {}

//----------------------------------------------------------------------------
// Updates/sychronizes tile classification vectors on the map.
//   This system runs each turn and synchronizes various attributes of Map with
//   the positions of entities currently in the ECS. These vectors are used when
//   computing routing for movement.
//----------------------------------------------------------------------------
impl<'a> System<'a> for MapSynchronizationSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        ReadStorage<'a, Opaque>,
        ReadStorage<'a, IsEntityKind>,
        Entities<'a>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, is_opaque, kind, entities) = data;

        // Re-initialize each of the map attributes.
        map.synchronize_blocked();
        map.synchronize_opaque();
        for e in map.fire.iter_mut() {*e = false}
        for e in map.chill.iter_mut() {*e = false}
        for e in map.shallow_water.iter_mut() {*e = false}
        for e in map.deep_water.iter_mut() {*e = false}
        for e in map.steam.iter_mut() {*e = false}
        for e in map.grass.iter_mut() {*e = false}
        map.clear_tile_content();

        for (pos, entity) in (&positions, &entities).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            map.blocked[idx] |= blockers.get(entity).is_some();
            map.opaque[idx] |= is_opaque.get(entity).is_some();
            let is_fire = kind.get(entity)
                .map_or(false, |k| matches!(k.kind, EntitySpawnKind::Fire {..}));
            map.fire[idx] |= is_fire;
            let is_chill = kind.get(entity)
                .map_or(false, |k| matches!(k.kind, EntitySpawnKind::Chill {..}));
            map.chill[idx] |= is_chill;
            let is_shallow_water = kind.get(entity)
                .map_or(false, |k| matches!(k.kind, EntitySpawnKind::ShallowWater {..}));
            map.shallow_water[idx] |= is_shallow_water;
            let is_deep_water = kind.get(entity)
                .map_or(false, |k| matches!(k.kind, EntitySpawnKind::DeepWater {..}));
            map.deep_water[idx] |= is_deep_water;
            let is_steam = kind.get(entity)
                .map_or(false, |k| matches!(k.kind, EntitySpawnKind::Steam {..}));
            map.steam[idx] |= is_steam;
            let is_grass = kind.get(entity)
                .map_or(false, |k| matches!(k.kind, EntitySpawnKind::ShortGrass {..} | EntitySpawnKind::TallGrass {..} ));
            map.grass[idx] |= is_grass;
            // Update tile content.
            map.tile_content[idx].push(entity);
        }
    }
}

pub struct ColorSynchronizationSystem {}

//----------------------------------------------------------------------------
// Updates/sychronizes tile classification vectors on the map.
//   This system runs each turn and synchronizes various attributes of Map with
//   the positions of entities currently in the ECS. These vectors are used when
//   computing routing for movement.
//----------------------------------------------------------------------------
impl<'a> System<'a> for ColorSynchronizationSystem {
    type SystemData = (
        ReadStorage<'a, Bloodied>,
        ReadStorage<'a, UseFgColorMapWhenBloodied>,
        WriteStorage<'a, UseFgColorMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (bloodieds, use_fg_when_bloodieds, mut use_fg_cmap) = data;

        for (_, _, fg_cmap) in (&bloodieds, &use_fg_when_bloodieds, &mut use_fg_cmap).join() {
            fg_cmap.set(FgColorMap::Blood)
        }
    }
}