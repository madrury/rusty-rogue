use specs::prelude::*;
use rltk::{field_of_view, Point};
use super::{Viewshed, Position, Map, Player};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(
                    |p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height
                );
                // When the entitiy is the player, mark the tiles in the entitiy's
                // viewshed as visible on the map.
                match player.get(ent) {
                    Some(_p) => {
                        for tile in map.visible_tiles.iter_mut() {
                            *tile = false;
                        }
                        for vis in viewshed.visible_tiles.iter() {
                            let idx = map.xy_idx(vis.x, vis.y);
                            map.revealed_tiles[idx] = true;
                            map.visible_tiles[idx] = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}