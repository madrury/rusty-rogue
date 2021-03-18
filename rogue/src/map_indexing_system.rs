use std::matches;
use specs::prelude::*;
use super::{Map, Position, BlocksTile, IsEntityKind, EntitySpawnKind};

pub struct MapIndexingSystem {}

//----------------------------------------------------------------------------
// Updates/sychronizes internal data stores on Map.
//   This system runs each turn and synchronizes various attributes of Map with
//   the entities currently on the game map:
//
//   - blocked: A boolean array that tracks what indexes are blocked for movement.
//   - fire: A boolean array that tracks if fire currently burning in the tile?
//   - tile_content: Vector of entities residing in a tile.
//----------------------------------------------------------------------------
impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        ReadStorage<'a, IsEntityKind>,
        Entities<'a>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, kind, entities) = data;

        // Re-initialize each of the map attributes.
        map.populate_blocked();
        for e in map.fire.iter_mut() {*e = false}
        map.clear_tile_content();

        for (pos, entity) in (&positions, &entities).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            // Syncronize map.blocked.
            map.blocked[idx] |= blockers.get(entity).is_some();
            // Syncronize map.fire.
            let is_fire = kind.get(entity)
                .map_or(false, |k| matches!(k.kind, EntitySpawnKind::Fire {..}));
            map.fire[idx] |= is_fire;
            // Syncronize map.chill.
            let is_chill = kind.get(entity)
                .map_or(false, |k| matches!(k.kind, EntitySpawnKind::Chill {..}));
            map.chill[idx] |= is_chill;
            // Update tile content.
            map.tile_content[idx].push(entity);
        }
    }
}