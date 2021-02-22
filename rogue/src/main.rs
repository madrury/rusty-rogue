use rltk::{GameState, Rltk, RGB, Point};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod visibility_system;
use visibility_system::*;
mod monster_ai_system;
use monster_ai_system::*;
mod map_indexing_system;
use map_indexing_system::*;
mod rectangle;
pub use rectangle::{Rectangle};

// Render all entities all the time.
const DEBUG_RENDER_ALL: bool = true;


#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

pub struct State {
    pub ecs: World,
    pub state: RunState,
}

impl State {

    // Draw all entities with a Renderable component on the console.
    fn render_all(&self, ctx: &mut Rltk) {
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] || DEBUG_RENDER_ALL {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }

    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterMovementSystem{};
        mob.run_now(&self.ecs);
        let mut mis = MapIndexingSystem{};
        mis.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn debug_print_positions(&mut self) {
        let positions = self.ecs.write_storage::<Position>();
        let names = self.ecs.write_storage::<Name>();
        for (pos, name) in (&positions, &names).join() {
            println!("{} is in position {:?}", name.name, *pos)
        }
    }

}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        if self.state == RunState::Running {
            self.run_systems();
            // self.debug_print_positions();
            self.state = RunState::Paused;
        } else {
            self.state = player_input(self, ctx);
        }
        draw_map(&self.ecs, ctx);
        self.render_all(ctx)
    }
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_fps_cap(30.0)
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
        state: RunState::Running
    };

    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<MonsterMovementAI>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<BlocksTile>();

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
        .with(Name {name: "Player".to_string()})
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5
        })
        .build();

    // Construct monster entities.
    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('O');
                name = "Orc".to_string();
            }
        }
        gs.ecs.create_entity()
            .with(Position {x, y})
            .with(Monster {})
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
            .with(Name {name: format!("{} #{}", &name, i)})
            .with(MonsterMovementAI {
                only_follow_within_viewshed: true,
                no_visibility_wander: true,
                lost_visibility_keep_following_turns_max: 2,
                lost_visibility_keep_following_turns_remaining: 2,
            })
            .with(CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4
            })
            .with(BlocksTile {})
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(px, py));

    rltk::main_loop(context, gs)
}