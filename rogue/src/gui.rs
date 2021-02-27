use super::{CombatStats, GameLog, InBackpack, Map, Name, Player, Position};
use rltk::{Point, Rltk, RGB, VirtualKeyCode};
use specs::prelude::*;

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

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mpos = ctx.mouse_pos();
    if mpos.0 >= map.width || mpos.1 >= map.height {
        return;
    }

    // Grab the names of the entities at the mouse position,
    let mut tooltip: Vec<String> = Vec::new();
    for (name, pos) in (&names, &positions).join() {
        let idx = map.xy_idx(pos.x, pos.y);
        if pos.x == mpos.0 && pos.y == mpos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    // Draw the tooltip.
    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 2;

        if mpos.0 > map.width / 2 {
            let arrow_pos = Point::new(mpos.0 - 1, mpos.1);
            let left_x = mpos.0 - width;
            let mut y = mpos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = width - s.len() as i32;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.set(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                26,
            );
        } else {
            let arrow_pos = Point::new(mpos.0 + 1, mpos.1);
            let left_x = mpos.0 + 3;
            let mut y = mpos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = width - s.len() as i32;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.set(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                27,
            );
        }
    }
}

// Item Menu
//----------------------------------------------------------------------------

const ITEM_MENU_X_POSITION: i32 = 15;
const ITEM_MENU_Y_POSTION: i32 = 25;
const ITEM_MENU_WIDTH: i32 = 31;

pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected { item: Entity },
}

pub fn show_inventory(ecs: &mut World, ctx: &mut Rltk) -> ItemMenuResult {
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let inbackpacks = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();

    let inventory = (&inbackpacks, &names)
        .join()
        .filter(|(item, _name)| item.owner == *player_entity);
    let count = inventory.count();

    let mut y = ITEM_MENU_Y_POSTION - (count / 2) as i32;

    // Draw the outline of the menu, and the helper text.
    ctx.draw_box(ITEM_MENU_X_POSITION, y - 2, ITEM_MENU_WIDTH, (count + 3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(ITEM_MENU_X_POSITION + 3, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(ITEM_MENU_X_POSITION + 3, y + count as i32 + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let inventory = (&entities, &inbackpacks, &names)
        .join()
        .filter(|(_e, item, _name)| item.owner == *player_entity)
        .enumerate();
    let mut usable: Vec<Entity> = Vec::new();
    for (i, (entity, _pack, name)) in inventory {
        // Draw the selection information. (a), (b), (c) etc.
        let selection_char = 97 + i as rltk::FontCharType;
        ctx.set(ITEM_MENU_X_POSITION + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(ITEM_MENU_X_POSITION + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), selection_char);
        ctx.set(ITEM_MENU_X_POSITION + 3, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));
        ctx.print(ITEM_MENU_X_POSITION + 5, y, &name.name.to_string());
        usable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => ItemMenuResult::Cancel,
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return ItemMenuResult::Selected {
                            item: usable[selection as usize]
                        }
                    }
                    ItemMenuResult::NoResponse
                }
            }
        }
    }
}
