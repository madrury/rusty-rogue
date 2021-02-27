use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use super::{Position, Player, Map, State, Viewshed, CombatStats, RunState, WantsToMeleeAttack};
use std::cmp::{min, max};


pub fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut players = ecs.write_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMeleeAttack>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut ppos = ecs.write_resource::<Point>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, vs) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        let destination_idx = map.xy_idx(pos.x + dx, pos.y + dy);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            match target {
                None => {}
                Some(_t) => {
                    wants_to_melee
                        .insert(entity, WantsToMeleeAttack {target: *potential_target})
                        .expect("Insert of WantsToMelee into ECS failed.");
                    return; // Do not move after attacking.
                }
            }
        }
        // Move the player if the destination is not blocked.
        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + dx));
            pos.y = min(79, max(0, pos.y + dy));
            vs.dirty = true;
        }
        ppos.x = pos.x;
        ppos.y = pos.y;
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => {
            return RunState::AwaitingInput
        }
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::H
                => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right | VirtualKeyCode::L
                => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up | VirtualKeyCode::K
                => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down | VirtualKeyCode::J
                => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Y
                => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::U
                => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::N
                => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::B
                => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::Z
                => try_move_player(0, 0, &mut gs.ecs),
            _   => {}
        }
    }
    RunState::PlayerTurn
}