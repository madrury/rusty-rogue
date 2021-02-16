use specs::prelude::*;
use rltk::{field_of_view, Point};
use super::{Viewshed, Position, Map};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut viewshed, pos) = data;
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed.visible_tiles.retain(
                |p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height
            );
        }
    }
}