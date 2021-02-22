use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

// Updates internal data stores on Map:
//   - blocked: Tracks what indexes are blocked for movement.
//   - tile_content: Vector of entities residing in a tile.
impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, entities) = data;

        map.populate_blocked();
        map.clear_tile_content();

        for (pos, entity) in (&positions, &entities).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            // If the entity blocks...
            let _p : Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }
            // Push the entity to the appropriate slot in in the tile_content
            // map vector.
            map.tile_content[idx].push(entity);
        }
    }
}