use super::{
    get_status_indicators, CombatStats, GameLog, InBackpack, Map, Name, Player, Position,
    Renderable, RunState, StatusIndicatorGlyph, Viewshed, Equipped
};
use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

//----------------------------------------------------------------------------
// Main menu, where the player can select between:
//   - New Game.
//   - Load Game.
//   - Quit.
//----------------------------------------------------------------------------
#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { current: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(ecs: &mut World, ctx: &mut Rltk) -> MainMenuResult {
    let save_exists = super::save_load::does_save_exist();
    let runstate = ecs.fetch::<RunState>();

    ctx.print_color_centered(
        10,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Rusty Roguelike",
    );

    if let RunState::MainMenu { current } = *runstate {
        if current == MainMenuSelection::NewGame {
            ctx.draw_box(
                20,
                18,
                40,
                4,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
            );
            ctx.print_color_centered(
                20,
                RGB::named(rltk::YELLOW),
                RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        } else {
            ctx.print_color_centered(
                20,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        }

        if save_exists {
            if current == MainMenuSelection::LoadGame {
                ctx.draw_box(
                    20,
                    21,
                    40,
                    4,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                );
                ctx.print_color_centered(
                    23,
                    RGB::named(rltk::YELLOW),
                    RGB::named(rltk::BLACK),
                    "Load Game",
                );
            } else {
                ctx.print_color_centered(
                    23,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    "Load Game",
                );
            }
        }

        if current == MainMenuSelection::Quit {
            ctx.draw_box(
                20,
                24,
                40,
                4,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
            );
            ctx.print_color_centered(
                26,
                RGB::named(rltk::YELLOW),
                RGB::named(rltk::BLACK),
                "Quit",
            );
        } else {
            ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection { current: current },
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        current: MainMenuSelection::Quit,
                    }
                }
                VirtualKeyCode::Up => {
                    let mut selection;
                    match current {
                        MainMenuSelection::NewGame => selection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => selection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => selection = MainMenuSelection::LoadGame,
                    }
                    if selection == MainMenuSelection::LoadGame && !save_exists {
                        selection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection { current: selection };
                }
                VirtualKeyCode::Down => {
                    let mut selection;
                    match current {
                        MainMenuSelection::NewGame => selection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => selection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => selection = MainMenuSelection::NewGame,
                    }
                    if selection == MainMenuSelection::LoadGame && !save_exists {
                        selection = MainMenuSelection::Quit;
                    }
                    return MainMenuResult::NoSelection { current: selection };
                }
                VirtualKeyCode::Return => return MainMenuResult::Selected { selected: current },
                _ => return MainMenuResult::NoSelection { current: current },
            },
        }
    }
    MainMenuResult::NoSelection {
        current: MainMenuSelection::NewGame,
    }
}

//----------------------------------------------------------------------------
// Headsup UI.
// Displays player state information (HP), and the game log.
//----------------------------------------------------------------------------
const XPOSITION: i32 = 0;
const YPOSITION: i32 = 43;
const BOXWIDTH: i32 = 79;
const BOXHEIGHT: i32 = 6;

const HEALTH_TEXT_XPOSITION: i32 = 12;
const HEALTH_TEXT_YPOSITION: i32 = 43;
const HEALTH_BAR_XPOSITION: i32 = 28;
const HEALTH_BAR_YPOSITION: i32 = 43;
const HEALTH_BAR_WIDTH: i32 = 51;

const N_LOG_LINES: i32 = 5;

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    // Draw the outline for the gui: A rectangle at the bottom of the console.
    ctx.draw_box(
        XPOSITION,
        YPOSITION,
        BOXWIDTH,
        BOXHEIGHT,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    // Display the current dungeon level.
    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(
        XPOSITION + 2,
        YPOSITION,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        &depth,
    );

    // Draw the players healthbar at the top of the gui.
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            HEALTH_TEXT_XPOSITION,
            HEALTH_TEXT_YPOSITION,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health,
        );
        ctx.draw_bar_horizontal(
            HEALTH_BAR_XPOSITION,
            HEALTH_BAR_YPOSITION,
            HEALTH_BAR_WIDTH,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }

    // Draw the gamelog inside the gui.
    let log = ecs.fetch::<GameLog>();
    for (i, s) in log.entries.iter().rev().enumerate() {
        if i < N_LOG_LINES as usize {
            ctx.print(2, YPOSITION as usize + i + 1, s)
        }
    }

    // Render the mouse position.
    let mpos = ctx.mouse_pos();
    ctx.set_bg(mpos.0, mpos.1, RGB::named(rltk::MAGENTA));

    draw_tooltips(ecs, ctx);
}

//----------------------------------------------------------------------------
// Tooltips.
// Gives information about an entity when the mouse cursor is over it. Displays
// the eneity's name and any status effects they are currenly under.
//----------------------------------------------------------------------------
fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let bg_color = RGB::named(rltk::DIM_GREY);

    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mpos = ctx.mouse_pos();
    if mpos.0 >= map.width || mpos.1 >= map.height {
        return;
    }

    // Grab info about the entities at the mouse positions:
    //   - Names
    //   - Glyphs representing current status effects.
    let mut tooltip: Vec<String> = Vec::new();
    let mut status_indicators: Vec<Vec<StatusIndicatorGlyph>> = Vec::new();
    for (entity, name, pos) in (&ecs.entities(), &names, &positions).join() {
        let idx = map.xy_idx(pos.x, pos.y);
        if pos.x == mpos.0 && pos.y == mpos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
            status_indicators.push(get_status_indicators(&ecs, &entity))
        }
    }

    // Draw the tooltip.
    if !tooltip.is_empty() {
        // Calculate the width needed to draw the tooltip. We need to fit the
        // text (entity names), and status indicators.
        let mut width: i32 = 0;
        for (s, inds) in tooltip.iter().zip(status_indicators.iter()) {
            if width < s.len() as i32 + inds.len() as i32 {
                width = s.len() as i32 + inds.len() as i32;
            }
        }
        // Buffer room for spacing and an arrow charecter.
        width += 2;

        // Cursor is on the right half of the screen, so render the tooltip to
        // the left.
        // (NAME OF ENTITY)(STATUS GLYPHS) →ENTITY
        if mpos.0 > map.width / 2 {
            let arrow_pos = Point::new(mpos.0 - 1, mpos.1);
            let left_x = mpos.0 - width;
            let mut y = mpos.1;
            for (s, inds) in tooltip.iter().zip(status_indicators.iter()) {
                // Print the entities name.
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), bg_color, s);
                // Print indicators for the entities current status.
                for (i, ind) in inds.iter().enumerate() {
                    ctx.set(
                        left_x + s.len() as i32 + i as i32,
                        y,
                        ind.color,
                        bg_color,
                        ind.glyph,
                    );
                }
                let padding = width - s.len() as i32 - inds.len() as i32;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        bg_color,
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.set(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                bg_color,
                rltk::to_cp437('→'),
            );
        // Tooltip is on the left half of the screen, so render the tooltip to
        // the right.
        // (ENTITY)← (NAME OF ENTITY)(STATUS GLYPHS)
        } else {
            let arrow_pos = Point::new(mpos.0 + 1, mpos.1);
            let left_x = mpos.0 + 3;
            let mut y = mpos.1;
            for (s, inds) in tooltip.iter().zip(status_indicators.iter()) {
                // Print the entities name.
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), bg_color, s);
                // Print indicators for the entities current status.
                for (i, ind) in inds.iter().enumerate() {
                    ctx.set(
                        left_x + s.len() as i32 + i as i32,
                        y,
                        ind.color,
                        bg_color,
                        ind.glyph,
                    );
                }
                let padding = width - s.len() as i32 - inds.len() as i32;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + i,
                        y,
                        RGB::named(rltk::WHITE),
                        bg_color,
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.set(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                bg_color,
                rltk::to_cp437('←'),
            );
        }
    }
}

//----------------------------------------------------------------------------
// Inventory Menus
// Allows the selection of an item to use/throw it. This method is generic over
// a component type. The menu will display the entities in the player's backpack
// with that component.
//----------------------------------------------------------------------------
const ITEM_MENU_X_POSITION: i32 = 15;
const ITEM_MENU_Y_POSTION: i32 = 25;
const ITEM_MENU_WIDTH: i32 = 31;

pub enum MenuResult {
    Cancel,
    NoResponse,
    Selected {thing: Entity},
}

// The inventory menu is generic over a type parameter. This supports different
// menus for items with different tags.
pub fn show_inventory<T: Component>(ecs: &mut World, ctx: &mut Rltk, typestr: &str) -> MenuResult {
    let player = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let inbackpacks = ecs.read_storage::<InBackpack>();
    let equipped = ecs.read_storage::<Equipped>();
    let doables = ecs.read_storage::<T>();
    let renderables = ecs.read_storage::<Renderable>();
    let entities = ecs.entities();

    let inventory = (&inbackpacks, &doables, &names)
        .join()
        .filter(|(item, _use, _do)| item.owner == *player);
    let count = inventory.count();

    let mut y = ITEM_MENU_Y_POSTION - (count / 2) as i32;

    // Draw the outline of the menu, and the helper text.
    ctx.draw_box(
        ITEM_MENU_X_POSITION,
        y - 2,
        ITEM_MENU_WIDTH,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        ITEM_MENU_X_POSITION + 3,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        format!("{} Items", typestr),
    );
    ctx.print_color(
        ITEM_MENU_X_POSITION + 3,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Press ESC to cancel",
    );

    // Iterate through all items in the player's backpack with the Useable component and:
    //   - Draw an inventory selection letter for that item: (a), (b), (c), etc...
    //   - Add the item to a vector for later lookup upon selection.
    let inventory = (&entities, &inbackpacks, &names, &doables)
        .join()
        .filter(|(_e, inpack, _name, _do)| inpack.owner == *player)
        .enumerate();
    // Vector to keep track of the positions of items in the inventory. When the
    // player selects an item, we need to retrieve that associated item entity.
    let mut useable: Vec<Entity> = Vec::new();
    for (i, (item, _pack, name, _use)) in inventory {
        let render = renderables.get(item);
        // Draw the selection information. (a), (b), (c) etc.
        let selection_char = 97 + i as rltk::FontCharType;
        ctx.set(
            ITEM_MENU_X_POSITION + 1,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            ITEM_MENU_X_POSITION + 2,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            selection_char,
        );
        ctx.set(
            ITEM_MENU_X_POSITION + 3,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );
        // Draw the item glyph if one exists.
        match render {
            Some(render) => ctx.set(
                ITEM_MENU_X_POSITION + 4,
                y,
                render.fg,
                RGB::named(rltk::BLACK),
                render.glyph,
            ),
            None => ctx.set(
                ITEM_MENU_X_POSITION + 4,
                y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                rltk::to_cp437(' '),
            ),
        }

        // Render the item name, with a color indicating if the item is
        // equipped.
        let is_equipped = equipped.get(item).map_or(false, |e| e.owner == *player);
        let text_fg = if is_equipped {
            RGB::named(rltk::GREEN)
        } else {
            RGB::named(rltk::WHITE)
        };
        ctx.print_color(
            ITEM_MENU_X_POSITION + 6,
            y,
            text_fg,
            RGB::named(rltk::BLACK),
            &name.name.to_string()
        );

        useable.push(item);
        y += 1;
    }

    // If we've got input, we can get to using the thing.
    match ctx.key {
        None => MenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => MenuResult::Cancel,
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return MenuResult::Selected {
                        thing: useable[selection as usize],
                    };
                }
                MenuResult::NoResponse
            }
        },
    }
}

//----------------------------------------------------------------------------
// Targeting system.
// Allows the selection of a target with the mouse.
//----------------------------------------------------------------------------
pub enum TargetingResult {
    Cancel,
    NoResponse,
    SwitchModality,
    Selected {pos: Point},
    MoveCursor {pos: Point}
}

// Select a target witin a range using the mouse.
pub fn ranged_target_mouse(
    ecs: &mut World,
    ctx: &mut Rltk,
    range: i32,
    radius: Option<i32>,
) -> TargetingResult {
    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Select a Target With the Mouse:",
    );

    let mouse_pos = ctx.mouse_pos();
    let mouse_pos_point = Point {
        x: mouse_pos.0,
        y: mouse_pos.1,
    };
    // Compute the vactor of cells within range and draw the targeting
    // recepticle.
    let available_cells = draw_targeting_system(ecs, ctx, range, radius, &mouse_pos_point);
    // Draw mouse cursor and check for clicks within range.
    let mut valid_target = false;
    for pt in available_cells.iter() {
        if pt.x == mouse_pos.0 && pt.y == mouse_pos.1 {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::GREEN));
        if ctx.left_click {
            return TargetingResult::Selected {
                pos: mouse_pos_point,
            };
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return TargetingResult::Cancel;
        }
    }
    // Let the player ESC out.
    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Escape => return TargetingResult::Cancel,
            VirtualKeyCode::Tab => return TargetingResult::SwitchModality,
            _ => return TargetingResult::NoResponse,
        }
    }
    // Nothing happened, nothing changes.
    TargetingResult::NoResponse
}

// Select a target witin a range using the keyboard.
pub fn ranged_target_keyboard(
    ecs: &mut World,
    ctx: &mut Rltk,
    range: i32,
    radius: Option<i32>,
    current: Option<Point>
) -> TargetingResult {
    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Select a Target With the Keyboard:",
    );

    let cursor: Point;
    let ppos = ecs.read_resource::<Point>();
    match current {
        Some(current) => cursor = current,
        None => cursor = Point {x: ppos.x, y: ppos.y}
    }
    // Compute the vactor of cells within range and draw the targeting
    // recepticle.
    let available_cells = draw_targeting_system(ecs, ctx, range, radius, &cursor);
    // Draw mouse cursor and check for clicks within range.
    let mut valid_target = false;
    for pt in available_cells.iter() {
        if *pt == cursor {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(cursor.x, cursor.y, RGB::named(rltk::GREEN));
    } else {
        ctx.set_bg(cursor.x, cursor.y, RGB::named(rltk::RED));
    }
    // Let the player ESC out.
    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Escape => return TargetingResult::Cancel,
            VirtualKeyCode::Tab => return TargetingResult::SwitchModality,
            VirtualKeyCode::Return => return TargetingResult::Selected {pos: cursor},
            VirtualKeyCode::Left | VirtualKeyCode::H =>
                return TargetingResult::MoveCursor {pos: Point {x: cursor.x - 1, y: cursor.y}},
            VirtualKeyCode::Right | VirtualKeyCode::L =>
                return TargetingResult::MoveCursor {pos: Point {x: cursor.x + 1, y: cursor.y}},
            VirtualKeyCode::Up | VirtualKeyCode::K =>
                return TargetingResult::MoveCursor {pos: Point {x: cursor.x, y: cursor.y - 1}},
            VirtualKeyCode::Down | VirtualKeyCode::J =>
                return TargetingResult::MoveCursor {pos: Point {x: cursor.x, y: cursor.y + 1}},
            _ => return TargetingResult::NoResponse,
        }
    }
    TargetingResult::NoResponse
}

// Draw the targeting system.
// We want to communicate both the avalable throwing range around the player,
// and any area of effect of the item shown. To that end, we render a circle
// around the player. Then, if the current targeted postion is within the range
// and the item has an area of effect, we also draw the aoe range.
fn draw_targeting_system(
    ecs: &World,
    ctx: &mut Rltk,
    range: i32,
    radius: Option<i32>,
    target: &Point,
) -> Vec<Point> {
    let viewsheds = ecs.read_storage::<Viewshed>();
    let player = ecs.read_resource::<Entity>();
    let ppos = ecs.read_resource::<Point>();
    let map = ecs.read_resource::<Map>();
    // Container for the points withing range.
    let mut available_cells = Vec::new();
    // This is a safe unwrap, since the player *always* has a viewshed.
    let visible = viewsheds.get(*player).unwrap();
    let mouse_within_range =
        rltk::DistanceAlg::Pythagoras.distance2d(*ppos, *target) <= range as f32;
    for point in visible.visible_tiles.iter() {
        let dplayer = rltk::DistanceAlg::Pythagoras.distance2d(*ppos, *point);
        // The tile is within the throwable range.
        if dplayer <= range as f32 {
            ctx.set_bg(point.x, point.y, RGB::named(rltk::LIGHT_GREY));
            available_cells.push(*point);
        }
        // When Some(radius), the item has an area of effect, so we highlight
        // the aoe range as well.
        if let Some(radius) = radius {
            let blast = rltk::field_of_view(*target, radius, &*map);
            if mouse_within_range && blast.contains(point) {
                ctx.set_bg(point.x, point.y, RGB::named(rltk::YELLOW));
            }
        }
    }
    available_cells
}