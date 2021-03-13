use specs::prelude::*;
use super::{Map, Position, BlocksTile, IsEntityKind, EntitySpawnKind};

pub struct MapIndexingSystem {}

// Updates internal data stores on Map:
//   - blocked: Tracks what indexes are blocked for movement.
//   - fire: Is fire currently burning in the tile?
//   - tile_content: Vector of entities residing in a tile.
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

        map.populate_blocked();
        for e in map.fire.iter_mut() {*e = false}
        map.clear_tile_content();

        for (pos, entity) in (&positions, &entities).join() {
            // Syncronize map.blocked.
            let idx = map.xy_idx(pos.x, pos.y);
            map.blocked[idx] = blockers.get(entity).is_some();
            // Syncronize map.fire.
            let is_fire = kind.get(entity)
                .map_or(false, |k| k.kind == EntitySpawnKind::Fire);
            map.fire[idx] |= is_fire;
            // Update tile content.
            map.tile_content[idx].push(entity);
        }
    }
}