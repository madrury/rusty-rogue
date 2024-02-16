use crate::{MeeleAttackFormation, MeeleAttackWepon, WantsToMoveToPosition};

use super::{
    CombatStats, GameLog, PickUpable, Map, Monster, Position, RunState,
    Renderable, Equipped, State, HungerClock, HungerState, Viewshed,
    MeeleAttackRequestBuffer, MeeleAttackRequest, AnimationRequestBuffer,
    AnimationRequest, WantsToPickupItem, StatusIsFrozen, TileType,
    WeaponSpecial, WeaponSpecialKind, StatusInvisibleToPlayer, MAP_HEIGHT,
    MAP_WIDTH
};
use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::{prelude::*, storage::GenericReadStorage};
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
const DESCRIPTION_PICKUP: &'static str = "Pickup item";
const DESCRIPTION_DESCEND: &'static str = "Descend";
const DESCRIPTION_INVENTORIES_SECTION: &'static str = "Inventories";
const DESCRIPTION_THROW: &'static str = "Throw item";
const DESCRIPTION_INVENTORY: &'static str = "Inventory";
const DESCRIPTION_EQUIP: &'static str = "Equipment";
const DESCRIPTION_SPELLBOOK: &'static str = "Spellbook";
const DESCRIPTION_MENU_SECTION: &'static str = "Game Menus";
const DESCRIPTION_HELP: &'static str = "Help menu";
const DESCRIPTION_MENU: &'static str = "Game menu";

const DETAILS_MOVE_LEFT: &'static str = "Move left";
const DETAILS_MOVE_RIGHT: &'static str = "Move right";
const DETAILS_MOVE_UP: &'static str = "Move up";
const DETAILS_MOVE_DOWN: &'static str = "Move down";
const DETAILS_MOVE_LEFT_UP: &'static str = "Move left-up";
const DETAILS_MOVE_RIGHT_UP: &'static str = "Move right-up";
const DETAILS_MOVE_LEFT_DOWN: &'static str = "Move left-down";
const DETAILS_MOVE_RIGHT_DOWN: &'static str = "Move right-down";
const DETAILS_SKIP: &'static str = r#"Skip turn
Pass your turn without taking action. NPCs will take a turn as normal.
If no monsters are nearby and you are not hungry then you will heal a small amount.
If you're hungry, try finding and eating some food."#;
const DETAILS_PICKUP: &'static str = r#"Pickup Item
Pickup items from the current tile.
Available items can be viewed in your inventory."#;
const DETAILS_DESCEND: &'static str = r#"Descend
Attempt to descend to the next level through the current tile"#;
const DETAILS_THROW: &'static str = r#"Throw item
Throw potions to cause a targeted effect. Not all potions are throwable."#;
const DETAILS_INVENTORY: &'static str = r#"Inventory
Show combined inventory"#;
const DETAILS_EQUIP: &'static str = r#"Equipment
Manage equipment"#;
const DETAILS_SPELLBOOK: &'static str = r#"Spellbook
Cast a spells by using scrolls stored in your spellbook.
Scrolls may store multiple charges and recharge naturally over time."#;
const DETAILS_HELP: &'static str = r#"Help menu
Show this menu"#;
const DETAILS_MENU: &'static str = r#"Main menu
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
    fn exec(&self, _ecs: &mut World) -> RunState {
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
                return RunState::AwaitingInput
            }
            _ => RunState::AwaitingInput
        },
    }
}

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) -> RunState {
    let player = ecs.fetch::<Entity>();
    let map = ecs.fetch_mut::<Map>();
    let pt = ecs.write_resource::<Point>();
    let renderables = ecs.read_storage::<Renderable>();
    let equipped = ecs.read_storage::<Equipped>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let weapons = ecs.read_storage::<MeeleAttackWepon>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let invisibles = ecs.read_storage::<StatusInvisibleToPlayer>();
    let mut specials = ecs.write_storage::<WeaponSpecial>();
    let mut moves = ecs.write_storage::<WantsToMoveToPosition>();
    let mut meele_buffer = ecs.write_resource::<MeeleAttackRequestBuffer>();
    let mut animation_buffer = ecs.write_resource::<AnimationRequestBuffer>();

    let destination_idx = map.xy_idx(pt.x + dx, pt.y + dy);
    let destination_is_blocked = map.blocked[destination_idx];
    let playerrender = renderables.get(*player)
        .expect("Failed to get Renderable component for player.");
    let formation = (&weapons, &equipped).join()
        .filter(|(_, eq,)| eq.owner == *player)
        .map(|(w, _)| w.formation)
        .next()
        .unwrap_or(MeeleAttackFormation::Basic);

    match formation {
        // The basic meele formation attacks only the tile in the direction of
        // movement.
        MeeleAttackFormation::Basic => {
            let targets = get_meele_targets_in_tile(
                &*map, destination_idx, &combat_stats, &invisibles, true
            );
            meele_buffer.request_many(*player, &targets, false);
            if !targets.is_empty() {
                return RunState::PlayerTurn
            }
        }
        // This meele attack formation, shamelessly ripped off from Brogue,
        // allows both moving and attacking in a single turn. If a monster is
        // located exactly one tile away (but not adjacent) to the player in the
        // direction of movement, we will both step into the open tile, and
        // attack the monster.
        MeeleAttackFormation::Dash => {
            let destination_idx = map.xy_idx(pt.x + dx, pt.y + dy);
            let dash_idx = map.xy_idx(pt.x + 2*dx, pt.y + 2*dy);
            // We still want to meele attack anything immeidately adjacent to
            // us, but *dont* trigger the dash if there is an invisible in the
            // dash target tile.
            let destination_targets = get_meele_targets_in_tile(
                &*map, destination_idx, &combat_stats, &invisibles, true
            );
            let dash_targets = get_meele_targets_in_tile(
                &*map, dash_idx, &combat_stats, &invisibles, false
            );
            if !destination_targets.is_empty() {
                meele_buffer.request_many(*player, &destination_targets, false);
                return RunState::PlayerTurn
            } else if !destination_is_blocked && !dash_targets.is_empty() {
                meele_buffer.request_many(*player, &dash_targets, false);
                moves.insert(
                    *player,
                    WantsToMoveToPosition {
                        pt: Point{x: pt.x + dx, y: pt.y + dy},
                        force: false
                    }
                ).expect("Failed to insert dash meele attack move.");
                animation_buffer.request(AnimationRequest::AlongRay {
                    source_x: pt.x,
                    source_y: pt.y,
                    target_x: pt.x + 2*dx,
                    target_y: pt.y + 2*dy,
                    fg: RGB::named(rltk::PURPLE),
                    bg: playerrender.bg,
                    glyph: playerrender.glyph,
                    until_blocked: true
                });
                return RunState::PlayerTurn
            }
        }
    };

    // Dash Weapon Special Attack.
    // An extended dash attack along the ray from the player in the direction of
    // movement.
    let ds = (&equipped, &mut specials).join()
        .filter(|(eq, _)| eq.owner == *player)
        .filter(|(_, s)| matches!(s.kind, WeaponSpecialKind::Dash) && s.is_charged())
        .next();
    if let Some((_, special)) = ds {
        let vs = viewsheds.get(*player).expect("Failed to get players viewshed.");
        let mut searchpt = Point{x: pt.x + dx, y: pt.y + dy};
        // Search along the ray from the player position in the direction of
        // movement for any meele target. We stop the search as soon as we find:
        //   - A tile outside the player's view.
        //   - Any blocked tile that is not meele attackable.
        //   - A meele attack target.
        // In the final case, we *know* the prevous search tile is unblocked,
        // because we encountered it in the seatch.
        loop {
            let search_idx = map.xy_idx(searchpt.x, searchpt.y);
            let tile_not_visible =  !vs.visible_tiles.contains(&searchpt);
            let is_blocked = map.blocked[search_idx];
            // Again, don't count invisible meele targets in the dash tile.
            let dash_targets = get_meele_targets_in_tile(
                &*map, search_idx, &combat_stats, &invisibles, false
            );
            if tile_not_visible {
                break;
            } else if dash_targets.is_empty() && is_blocked {
                break
            // Found a target: DASH!
            } else if !dash_targets.is_empty() {
                special.expend();
                meele_buffer.request_many(*player, &dash_targets, true);
                moves.insert(
                    *player,
                    WantsToMoveToPosition {
                        pt: Point{x: searchpt.x - dx, y: searchpt.y - dy},
                        force: false
                    }
                ).expect("Failed to insert dash meele attack move.");
                animation_buffer.request(AnimationRequest::AlongRay {
                    source_x: pt.x,
                    source_y: pt.y,
                    target_x: searchpt.x,
                     target_y: searchpt.y,
                     fg: RGB::named(rltk::PURPLE),
                     bg: playerrender.bg,
                     glyph: playerrender.glyph,
                     until_blocked: true
                });
                return RunState::PlayerTurn
            }
            searchpt = Point {x: searchpt.x + dx, y: searchpt.y + dy};
        }
    }

    // If the destination tile is unblocked, we can move the player into it, and
    // then pass the turn.
    if !destination_is_blocked {
        moves.insert(
            *player,
            WantsToMoveToPosition {pt: Point{x: pt.x + dx, y: pt.y + dy}, force: false}
        ).expect("Failed to insert player meele attack move.");;
        return RunState::PlayerTurn;
    }

    // If the desination tile is blocked, yet no meele happened, then you're
    // trying to move against a wall or something and we give you a freebie,
    // don't pass the turn.
    if destination_is_blocked {
        return RunState::AwaitingInput;
    }
    RunState::AwaitingInput
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player = ecs.fetch::<Entity>();
    let ppos = ecs.read_resource::<Point>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let renderables = ecs.read_storage::<Renderable>();
    let invisibles = ecs.read_storage::<StatusInvisibleToPlayer>();
    let monsters = ecs.read_storage::<Monster>();
    let hunger = ecs.read_storage::<HungerClock>();
    let equipped = ecs.read_storage::<Equipped>();
    let mut specials = ecs.write_storage::<WeaponSpecial>();
    let mut meele_buffer = ecs.write_resource::<MeeleAttackRequestBuffer>();
    let mut animation_buffer = ecs.write_resource::<AnimationRequestBuffer>();
    let map = ecs.fetch::<Map>();

    // Passing the turn with anjacent monsters starts a spin attack when a
    // sword's special is charged. Meele attacks all adjacent monsters, with a
    // free critical hit. Like in Link to the Past.
    let ws = (&equipped, &renderables, &mut specials).join()
        .filter(|(eq, _, _)| eq.owner == *player)
        .filter(|(_, _, s)| matches!(s.kind, WeaponSpecialKind::SpinAttack) && s.is_charged())
        .next();
    if let Some((_, render, special)) = ws {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let adjacent: Vec<Entity> = map.get_l_infinity_circle_around(*ppos, 1)
            .iter()
            .map(|pt| map.xy_idx(pt.x, pt.y))
            .map(|idx| get_meele_targets_in_tile(&*map, idx, &combat_stats, &invisibles, true))
            .flatten()
            .collect();
        meele_buffer.request_many(*player, &adjacent, true);
        if !adjacent.is_empty() {
            special.expend();
            animation_buffer.request(AnimationRequest::SpinAttack {
                x: ppos.x, y: ppos.y, fg: render.fg, glyph: render.glyph
            });
            return RunState::PlayerTurn
        }
    }

    // Check for monsters nearby, we can't wait heal if we know there are any.
    let mut can_heal = true;
    let pviewshed = viewsheds.get(*player).expect("Failed to obtain player viewshed.");
    for tile in pviewshed.visible_tiles.iter() {
        let idx = map.xy_idx(tile.x, tile.y);
        for entity in map.tile_content[idx].iter() {
            let is_monster = monsters.get(*entity).is_some();
            let is_invisible = invisibles.get(*entity).is_some();
            can_heal &= !is_monster || is_invisible;
        }
    }
    // If we're hungry or starving, we also cannot heal.
    can_heal &= hunger.get(*player)
        .map_or(
            true,
            |h| h.state == HungerState::WellFed || h.state == HungerState::Normal
        );

    if can_heal {
        let mut combat_stats = ecs.write_storage::<CombatStats>();
        let pstats = combat_stats.get_mut(*player)
            .expect("Failed to obtain combat stats for player when wail healing.");
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
        None => {
            log.entries.push("There is nothing here to pickup.".to_string());
            RunState::AwaitingInput
        },
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(*player, WantsToPickupItem{by: *player, item: item})
                .expect("Unable to pickup item.");
            RunState::PlayerTurn
        }
    }
}

fn get_meele_targets_in_tile(
    map: &Map,
    idx: usize,
    combat_stats: &ReadStorage<CombatStats>,
    invisible: &ReadStorage<StatusInvisibleToPlayer>,
    include_invisible: bool
) -> Vec<Entity> {
    let mut targets: Vec<Entity> = Vec::new();
    for target in map.tile_content[idx].iter() {
        let target_has_stats = combat_stats.get(*target).is_some();
        let target_is_visible = invisible.get(*target).is_none();
        if target_has_stats && (include_invisible || target_is_visible) {
            targets.push(*target);
        }
    }
    targets
}