use super::{
    CombatStats, GameLog, PickUpable, Map, Player, Monster, Position,
    RunState, State, HungerClock, HungerState, Viewshed, WantsToMeleeAttack,
    WantsToPickupItem, StatusIsFrozen, TileType
};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};


const WAIT_HEAL_AMOUNT: i32 = 1;

const DESCRIPTION_MOVEMENT_SECTION: &'static str = "Movement";
const DESCRIPTION_MOVE_LEFT: &'static str = "Move left";
const DESCRIPTION_MOVE_RIGHT: &'static str = "Move right";
const DESCRIPTION_MOVE_UP: &'static str = "Move up";
const DESCRIPTION_MOVE_DOWN: &'static str = "Move down";
const DESCRIPTION_MOVE_LEFT_UP: &'static str = "Move left-up";
const DESCRIPTION_MOVE_RIGHT_UP: &'static str = "Move right-up";
const DESCRIPTION_MOVE_LEFT_DOWN: &'static str = "Move left-down";
const DESCRIPTION_MOVE_RIGHT_DOWN: &'static str = "Move right-down";
const DESCRIPTION_MAP_INTERACTIONS_SECTION: &'static str = "Map Interactions";
const DESCRIPTION_SKIP: &'static str = "Skip turn";
const DESCRIPTION_PICKUP: &'static str = "Pickup item on tile";
const DESCRIPTION_DESCEND: &'static str = "Descend";
const DESCRIPTION_INVENTORIES_SECTION: &'static str = "Inventories";
const DESCRIPTION_THROW: &'static str = "Throw an item";
const DESCRIPTION_INVENTORY: &'static str = "Show inventory";
const DESCRIPTION_EQUIP: &'static str = "Show equipment";
const DESCRIPTION_SPELLBOOK: &'static str = "Cast a spell";
const DESCRIPTION_MENU_SECTION: &'static str = "Game Menus";
const DESCRIPTION_HELP: &'static str = "Show help menu";
const DESCRIPTION_MENU: &'static str = "Show game menu";

const DETAILS_MOVE_LEFT: &'static str = "Move left";
const DETAILS_MOVE_RIGHT: &'static str = "Move right";
const DETAILS_MOVE_UP: &'static str = "Move up";
const DETAILS_MOVE_DOWN: &'static str = "Move down";
const DETAILS_MOVE_LEFT_UP: &'static str = "Move left-up";
const DETAILS_MOVE_RIGHT_UP: &'static str = "Move right-up";
const DETAILS_MOVE_LEFT_DOWN: &'static str = "Move left-down";
const DETAILS_MOVE_RIGHT_DOWN: &'static str = "Move right-down";
const DETAILS_SKIP: &'static str = r#"Skip
Pass your turn without taking action. NPCs will take a turn as normal.
If no monsters are nearby and you are not hungry then you will heal a small amount.
If you're hungry, try finding and eating some food."#;
const DETAILS_PICKUP: &'static str = r#"Pickup
Pickup items from the current tile.
Available items can be viewed in your inventory."#;
const DETAILS_DESCEND: &'static str = "Attempt to descend to the next level through the current tile";
const DETAILS_THROW: &'static str = r#"Throw
Throw potions to cause a targeted effect. Not all potions are throwable."#;
const DETAILS_INVENTORY: &'static str = r#"Inventory
Show combined inventory"#;
const DETAILS_EQUIP: &'static str = r#"Equipment
Manage equipment"#;
const DETAILS_SPELLBOOK: &'static str = r#"Spellbook.
Cast a spells by using scrolls stored in your spellbook.
Scrolls may store multiple charges and recharge naturally over time."#;
const DETAILS_HELP: &'static str = r#"Help
Show this menu"#;
const DETAILS_MENU: &'static str = r#"Main Menu
Save, load, and exit the game"#;


pub trait KeyboundCommand {
    fn exec(&self, ecs: &mut World) -> RunState;
    fn key_codes(&self) -> &'static [VirtualKeyCode];
    fn description(&self) -> &'static str;
    fn details(&self) -> &'static str;
}

struct TryMovePlayerCommand {
    dx: i32,
    dy: i32,
    codes: &'static [VirtualKeyCode],
    description: &'static str,
    details: &'static str,
}


impl KeyboundCommand for TryMovePlayerCommand {
    fn exec(&self, ecs: &mut World) -> RunState {
        try_move_player(self.dx, self.dy, ecs)
    }

    fn key_codes(&self) -> &'static [VirtualKeyCode] {
        &self.codes
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn details(&self) -> &'static str {
        self.details
    }
}

struct SkipTurnCommand {}
impl KeyboundCommand for SkipTurnCommand {
    fn exec(&self, ecs: &mut World) -> RunState {
        skip_turn(ecs)
    }

    fn key_codes(&self) -> &'static [VirtualKeyCode] {
        &[VirtualKeyCode::Z]
    }

    fn description(&self) -> &'static str {
        DESCRIPTION_SKIP
    }

    fn details(&self) -> &'static str {
        DETAILS_SKIP
    }
}

struct PickupItemCommand {}
impl KeyboundCommand for PickupItemCommand {
    fn exec(&self, ecs: &mut World) -> RunState {
        pickup_item(ecs)
    }

    fn key_codes(&self) -> &'static [VirtualKeyCode] {
        &[VirtualKeyCode::G]
    }

    fn description(&self) -> &'static str {
        DESCRIPTION_PICKUP
    }

    fn details(&self) -> &'static str {
        DETAILS_PICKUP
    }
}


struct DescendCommand {}
impl KeyboundCommand for DescendCommand {
    fn exec(&self, ecs: &mut World) -> RunState {
        if try_next_level(ecs) {
            return RunState::NextLevel
        }
        return RunState::PlayerTurn
    }

    fn key_codes(&self) -> &'static [VirtualKeyCode] {
        &[VirtualKeyCode::Period]
    }

    fn description(&self) -> &'static str {
        DESCRIPTION_DESCEND
    }

    fn details(&self) -> &'static str {
        DETAILS_DESCEND
    }
}


struct GenericRunstateCommand {
    run_state: RunState,
    codes: &'static [VirtualKeyCode],
    description: &'static str,
    details: &'static str,
}
impl KeyboundCommand for GenericRunstateCommand {
    fn exec(&self, _ecs: &mut World) -> RunState {
        self.run_state
    }

    fn key_codes(&self) -> &'static [VirtualKeyCode] {
        return &self.codes
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn details(&self) -> &'static str {
        self.details
    }
}

struct NoopCommand {
    description: &'static str,
}
impl KeyboundCommand for NoopCommand {
    fn exec(&self, ecs: &mut World) -> RunState {
        RunState::AwaitingInput
    }

    fn key_codes(&self) -> &'static [VirtualKeyCode] {
        &[]
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn details(&self) -> &'static str {
        self.description
    }
}

pub const COMMANDS: [&'static dyn KeyboundCommand; 21] = [
    
    &NoopCommand{description: DESCRIPTION_MOVEMENT_SECTION},
    &TryMovePlayerCommand{dx: -1, dy: 0, codes: &[VirtualKeyCode::Left, VirtualKeyCode::H], description: DESCRIPTION_MOVE_LEFT, details: DETAILS_MOVE_LEFT},
    &TryMovePlayerCommand{dx: 1, dy: 0, codes: &[VirtualKeyCode::Right, VirtualKeyCode::L], description: DESCRIPTION_MOVE_RIGHT, details: DETAILS_MOVE_RIGHT},
    &TryMovePlayerCommand{dx: 0, dy: -1, codes: &[VirtualKeyCode::Up, VirtualKeyCode::K], description: DESCRIPTION_MOVE_UP, details: DETAILS_MOVE_UP},
    &TryMovePlayerCommand{dx: 0, dy: 1, codes: &[VirtualKeyCode::Down, VirtualKeyCode::J], description: DESCRIPTION_MOVE_DOWN, details: DETAILS_MOVE_DOWN},
    &TryMovePlayerCommand{dx: -1, dy: -1, codes: &[VirtualKeyCode::Y], description: DESCRIPTION_MOVE_LEFT_UP, details: DETAILS_MOVE_LEFT_UP},
    &TryMovePlayerCommand{dx: 1, dy: -1, codes: &[VirtualKeyCode::U], description: DESCRIPTION_MOVE_RIGHT_UP, details: DETAILS_MOVE_RIGHT_UP},
    &TryMovePlayerCommand{dx: 1, dy: 1, codes: &[VirtualKeyCode::N], description: DESCRIPTION_MOVE_RIGHT_DOWN, details: DETAILS_MOVE_RIGHT_DOWN},
    &TryMovePlayerCommand{dx: -1, dy: 1, codes: &[VirtualKeyCode::B], description: DESCRIPTION_MOVE_LEFT_DOWN, details: DETAILS_MOVE_LEFT_DOWN},
    &NoopCommand{description: DESCRIPTION_MAP_INTERACTIONS_SECTION},
    &SkipTurnCommand{},
    &PickupItemCommand{},
    &DescendCommand{},
    &NoopCommand{description: DESCRIPTION_INVENTORIES_SECTION},
    &GenericRunstateCommand{ run_state: RunState::ShowUseInventory, codes: &[VirtualKeyCode::I], description: DESCRIPTION_INVENTORY, details: DETAILS_INVENTORY },
    &GenericRunstateCommand{ run_state: RunState::ShowThrowInventory, codes: &[VirtualKeyCode::T], description: DESCRIPTION_THROW, details: DETAILS_THROW },
    &GenericRunstateCommand{ run_state: RunState::ShowEquipInventory, codes: &[VirtualKeyCode::E], description: DESCRIPTION_EQUIP, details: DETAILS_EQUIP },
    &GenericRunstateCommand{ run_state: RunState::ShowSpellbook, codes: &[VirtualKeyCode::S], description: DESCRIPTION_SPELLBOOK, details: DETAILS_SPELLBOOK },
    &NoopCommand{description: DESCRIPTION_MENU_SECTION},
    &GenericRunstateCommand{ run_state: RunState::ShowHelpMenu{details: None}, codes: &[VirtualKeyCode::Slash], description: DESCRIPTION_HELP, details: DETAILS_HELP },
    &GenericRunstateCommand{ run_state: RunState::SaveGame, codes: &[VirtualKeyCode::Escape], description: DESCRIPTION_MENU, details: DETAILS_MENU },
];

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Check if the player is frozen, in which case they cannot take any
    // meaningful action until the status expires.
    let player_is_frozen;
    {
        let status_is_frozen = gs.ecs.read_storage::<StatusIsFrozen>();
        let player = gs.ecs.fetch::<Entity>();
        player_is_frozen = status_is_frozen.get(*player).is_some();
    }
    // We still match on the key so the player can pass thier frozen turns
    // manually, otherwise the frozen turns pass rapidly and it's hard to tell
    // what is happening.
    if player_is_frozen {
        match ctx.key {
            Some(_) => return RunState::PlayerTurn,
            None => return RunState::AwaitingInput
        }
    }
    // Match the player input and delegate.
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
            VirtualKeyCode::I => RunState::ShowUseInventory,
            VirtualKeyCode::T => RunState::ShowThrowInventory,
            VirtualKeyCode::E => RunState::ShowEquipInventory,
            VirtualKeyCode::S => RunState::ShowSpellbook,
            VirtualKeyCode::Slash => RunState::ShowHelpMenu{details: None},
            VirtualKeyCode::Escape => RunState::SaveGame,
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel
                }
                return RunState::PlayerTurn
            }
            _ => RunState::AwaitingInput
        },
    }
}

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) -> RunState {
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
                    wants_to_melee.insert(
                        entity, WantsToMeleeAttack {
                            target: *potential_target,
                        },
                    ).expect("Insert of WantsToMelee into ECS failed.");
                    return RunState::PlayerTurn; // Do not move after attacking.
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
        // IMPORTANT: Keeps the players Position component synchronized with
        // their position as a resource in the ECS.
        ppos.x = pos.x;
        ppos.y = pos.y;
    }
    return RunState::PlayerTurn;
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player = ecs.fetch::<Entity>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();
    let hunger = ecs.read_storage::<HungerClock>();
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
    // If we're hungry or starving, we also cannot heal.
    can_heal &= hunger
        .get(*player)
        .map_or(
            true,
            |h| h.state == HungerState::WellFed || h.state == HungerState::Normal
        );

    if can_heal {
        let mut stats = ecs.write_storage::<CombatStats>();
        let pstats = stats.get_mut(*player).unwrap(); // The player always has stats.
        pstats.heal_amount(WAIT_HEAL_AMOUNT);
    }
    return RunState::PlayerTurn
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

fn pickup_item(ecs: &mut World) -> RunState {
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

    return RunState::PlayerTurn
}
