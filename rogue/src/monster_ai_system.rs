use specs::prelude::*;
use super::{Viewshed, Monster, MonsterMovementAI, Name, Position, Map, WantsToMeleeAttack, StatusIsFrozen};
use rltk::{Point, RandomNumberGenerator};

pub struct MonsterMovementSystem {
}

impl MonsterMovementSystem {

    // Move a monster to a new postions.
    // **THIS METHOD ASSUMES THE NEW POSITION IS SAFE TO MOVE INTO!**
    fn move_monster(map: &mut Map, pos: &mut Position, newposx: i32, newposy: i32, viewshed: &mut Viewshed) {
        let new_idx = map.xy_idx(newposx, newposy);
        let old_idx = map.xy_idx(pos.x, pos.y);
        // We need to update the blocking information *now*, since we do
        // not want later monsters in the move queue to move into the
        // same position as this monster.
        map.blocked[old_idx] = false;
        map.blocked[new_idx] = true;
        pos.x = newposx;
        pos.y = newposy;
        viewshed.dirty = true;
    }

    // Return a random adjcaent position to pos that is not currently blocked.
    fn random_adjacent_position(map: &Map, pos: &Position) -> (i32, i32) {
        let mut rng = RandomNumberGenerator::new();
        let dx = rng.range(-1, 2);
        let dy = rng.range(-1, 2);
        let idx = map.xy_idx(pos.x + dx, pos.y + dy);
        if !map.blocked[idx] {
            return (pos.x + dx, pos.y + dy)
        } else {
            return (pos.x, pos.y)
        }
    }
}

// System for handling monster movement.
// Iterates through all monsters and updates their position according to their
// MonsterMovementAI.
impl<'a> System<'a> for MonsterMovementSystem {

    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>, // Player position.
        ReadExpect<'a, Entity>, // Player entity.
        Entities<'a>,
        WriteStorage<'a, StatusIsFrozen>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, MonsterMovementAI>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMeleeAttack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player,
            entities,
            mut status_frozen,
            mut viewsheds,
            monsters,
            mut movement_ais,
            names,
            mut pos,
            mut melee_attack,
        ) = data;

        let iter = (&entities, &mut viewsheds, &monsters, &mut movement_ais, &names, &mut pos).join();
        for  (entity, mut viewshed, _monster, movement_ai, _name, mut pos) in iter {

            //Check for status effects that would prevent the monster from
            //taking actions.
            let is_frozen = status_frozen.get_mut(entity);
            if let Some(is_frozen) = is_frozen {
                if is_frozen.remaining_turns <= 0 {
                    status_frozen.remove(entity);
                    continue;
                } else {
                    &is_frozen.tick();
                    continue;
                }
            }

            // The monster is free to take actions.
            let in_viewshed = viewshed.visible_tiles.contains(&*player_pos);
            let keep_following = movement_ai.do_keep_following();
            let next_to_player = rltk::DistanceAlg::Pythagoras.distance2d(
                Point::new(pos.x, pos.y),
                *player_pos
            ) < 1.5;
            // The monster has the frozen status, and cannot take any action.
            // Monster next to player branch:
            //   If we're already next to player, we enter into melee combat.
            if next_to_player {
                melee_attack
                    .insert(entity, WantsToMeleeAttack {target: *player})
                    .expect("Failed to insert player as melee target.");
                continue;
            // Monster seeking player branch:
            //   This branch is taken if the monster is currently seeking the
            //   player, i.e., the monster is currently attempting to move towards
            //   the player until they are adjacent.
            } else if in_viewshed || keep_following {
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map
                );
                if path.success && path.steps.len() > 1 {
                    let new_x = path.steps[1] as i32 % map.width;
                    let new_y = path.steps[1] as i32 / map.width;
                    MonsterMovementSystem::move_monster(&mut map, &mut pos, new_x, new_y, &mut viewshed);
                }
                // Update our monster's propensity to keep following the player
                // when they lose visual contact. After a specified amount of
                // time, the monster will switch to idling.
                if in_viewshed {
                    movement_ai.reset_keep_following();
                } else {
                    movement_ai.decrement_keep_following();
                }
            // Monster idling branch.
            //   This branch is taken if the monster can not currently see the
            //   player, and are flagged to wander when the player is out of
            //   visible range.
            } else if !in_viewshed && movement_ai.no_visibility_wander {
                let new_pos = MonsterMovementSystem::random_adjacent_position(&map, pos);
                MonsterMovementSystem::move_monster(&mut map, &mut pos, new_pos.0, new_pos.1, &mut viewshed)
            }
        }
    }
}