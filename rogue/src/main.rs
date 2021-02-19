use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod visibility_system;
use visibility_system::*;
mod rectangle;
pub use rectangle::{Rectangle};


pub struct State {
    pub ecs: World,
}

impl State {

    fn render_all(&self, ctx: &mut Rltk) {
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }

    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }

}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        player_input(self, ctx);
        self.run_systems();
        draw_map(&self.ecs, ctx);
        self.render_all(ctx)
    }
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    let map = Map::new_rooms_and_corridors();

    // Construct the player entitiy.
    let (px, py) = map.rooms[0].center();
    gs.ecs
        .create_entity()
        .with(Position { x: px, y: py })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true
        })
        .build();

    let mut rng = rltk::RandomNumberGenerator::new();
    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center();
        let glyph: rltk::FontCharType;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {glyph = rltk::to_cp437('g')}
            _ => {glyph = rltk::to_cp437('O')}
        }
        gs.ecs.create_entity()
            .with(Position {x, y})
            .with(Renderable {
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true
            })
            .build();
    }

    gs.ecs.insert(map);
    rltk::main_loop(context, gs)
}