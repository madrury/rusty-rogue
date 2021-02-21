use specs::prelude::*;
use super::{Viewshed, Monster, Name, Position, Map};
use rltk::{Point, console};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, mut viewsheds, monsters, names, mut pos) = data;
        for  entitiy_data in (&mut viewsheds, &monsters, &names, &mut pos).join() {
            let (mut viewshed, _monster, name, mut pos) = entitiy_data;
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                Point::new(pos.x, pos.y),
                *player_pos
            );
            if viewshed.visible_tiles.contains(&*player_pos) && distance < 1.5 {
                console::log(format!("{} shouts insults!", name.name));
                // We're adjacent to the player, so we don't need to move.
                return;
            }
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(player_pos.x, player_pos.y) as i32,
                &mut *map
            );
            if path.success && path.steps.len() > 1 {
                // (pos.x, pos.y) = map.idx_xy(path.steps[1]);
                pos.x = path.steps[1] as i32 % map.width;
                pos.y = path.steps[1] as i32 / map.width;
                viewshed.dirty = true;
            }
        }
    }
}