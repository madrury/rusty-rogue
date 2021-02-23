use rltk::{ RGB, Rltk, Console };
use specs::prelude::*;
use super::{CombatStats, Player};


const XPOSITION: i32 = 0;
const YPOSITION: i32 = 43;
const BOXWIDTH: i32 = 79;
const BOXHEIGHT: i32 = 6;

const HEALTH_TEXT_XPOSITION: i32 = 12;
const HEALTH_TEXT_YPOSITION: i32 = 43;
const HEALTH_BAR_XPOSITION: i32 = 28;
const HEALTH_BAR_YPOSITION: i32 = 43;
const HEALTH_BAR_WIDTH: i32 = 51;


pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {

    // A rectangle at the bottom of the console.
    ctx.draw_box(XPOSITION,  YPOSITION, BOXWIDTH, BOXHEIGHT, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            HEALTH_TEXT_XPOSITION,
            HEALTH_TEXT_YPOSITION,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health
        );
        ctx.draw_bar_horizontal(
            HEALTH_BAR_XPOSITION,
            HEALTH_BAR_YPOSITION,
            HEALTH_BAR_WIDTH,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK)
        );
    }


}