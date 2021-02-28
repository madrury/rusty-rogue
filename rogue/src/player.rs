use super::{
    CombatStats, GameLog, PickUpable, Map, Player, Position, RunState, State, Viewshed,
    WantsToMeleeAttack, WantsToPickupItem,
};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right | VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up | VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down | VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::Z => try_move_player(0, 0, &mut gs.ecs),
            VirtualKeyCode::G => pickup_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowUseInventory,
            VirtualKeyCode::T => return RunState::ShowThrowInventory,
            _ => {}
        },
    }
    RunState::PlayerTurn
}

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut players = ecs.write_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMeleeAttack>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut ppos = ecs.write_resource::<Point>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, vs) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        let destination_idx = map.xy_idx(pos.x + dx, pos.y + dy);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            match target {
                None => {}
                Some(_t) => {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMeleeAttack {
                                target: *potential_target,
                            },
                        )
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

fn pickup_item(ecs: &mut World) {
    let ppos = ecs.fetch::<Point>(); // Player position.
    let player = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let pickupables = ecs.read_storage::<PickUpable>();
    let positions = ecs.read_storage::<Position>();
    let mut log = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (entity, _pu, pos) in (&entities, &pickupables, &positions).join() {
        if pos.x == ppos.x && pos.y == ppos.y {
            target_item = Some(entity);
            break;
        }
    }

    match target_item {
        None => log.entries.push("There is nothing here to pickup.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(*player, WantsToPickupItem{by: *player, item: item})
                .expect("Unable to pickup item.");
        }
    }
}