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
                continue;
            }
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(player_pos.x, player_pos.y) as i32,
                &mut *map
            );
            if path.success && path.steps.len() > 1 {
                // Move the monster to a new location one step along the path
                // towards the player.
                let new_x = path.steps[1] as i32 % map.width;
                let new_y = path.steps[1] as i32 / map.width;
                let new_idx = map.xy_idx(new_x, new_y);
                let old_idx = map.xy_idx(pos.x, pos.y);
                // We need to update the blocking information *now*, since we do
                // not want later monsters in the move queue to move into the
                // same position as this monster.
                map.blocked[old_idx] = false;
                map.blocked[new_idx] = true;
                pos.x = new_x;
                pos.y = new_y;
                viewshed.dirty = true;
            }
        }
    }
}