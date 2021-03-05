use super::{
    CombatStats, GameLog, PickUpable, Map, Player, Monster, Position,
    RunState, State, Viewshed, WantsToMeleeAttack, WantsToPickupItem,
    TileType
};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};


const WAIT_HEAL_AMOUNT: i32 = 1;


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
            VirtualKeyCode::Z => skip_turn(&mut gs.ecs),
            VirtualKeyCode::G => pickup_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowUseInventory,
            VirtualKeyCode::T => return RunState::ShowThrowInventory,
            VirtualKeyCode::E => return RunState::ShowEquipInventory,
            VirtualKeyCode::Escape => return RunState::SaveGame,
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel
                }
            }
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
        // Initiate a melee attack if the requested position is blocked.
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
            // The blocked array updates in the map_indexing_system. We don't
            // need to update it now, since no other entities are moving at the
            // moment.
            pos.x = min(79, max(0, pos.x + dx));
            pos.y = min(79, max(0, pos.y + dy));
            vs.dirty = true;
        }
        ppos.x = pos.x;
        ppos.y = pos.y;
    }
}

fn skip_turn(ecs: &mut World) {
    let player = ecs.fetch::<Entity>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();
    let map = ecs.fetch::<Map>();

    // Check for monsters nearby, we can't heal if there are any.
    let mut can_heal = true;
    let pviewshed = viewsheds.get(*player).unwrap(); // The player always has a viewshed.
    for tile in pviewshed.visible_tiles.iter() {
        let idx = map.xy_idx(tile.x, tile.y);
        for entity in map.tile_content[idx].iter() {
            can_heal &= monsters.get(*entity).is_none();
        }
    }

    if can_heal {
        let mut stats = ecs.write_storage::<CombatStats>();
        let pstats = stats.get_mut(*player).unwrap(); // The player always has stats.
        pstats.heal_amount(WAIT_HEAL_AMOUNT);
    }
}

fn try_next_level(ecs: &mut World) -> bool {
    let ppos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let pidx = map.xy_idx(ppos.x, ppos.y);
    if map.tiles[pidx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        false
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