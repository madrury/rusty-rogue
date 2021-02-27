use rltk::{GameState, Rltk, Point};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod spawner;
mod player;
use player::*;
mod gui;
use gui::*;
mod visibility_system;
use visibility_system::*;
mod monster_ai_system;
use monster_ai_system::*;
mod melee_combat_system;
use melee_combat_system::*;
mod damage_system;
use damage_system::*;
mod inventory_system;
use inventory_system::*;
mod map_indexing_system;
use map_indexing_system::*;
mod gamelog;
use gamelog::{GameLog};

// Render all entities all the time.
const DEBUG_RENDER_ALL: bool = false;


#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    PreGame,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
}

pub struct State {
    pub ecs: World,
}

impl State {

    // Draw all entities with a Renderable component on the console.
    fn render_all(&self, ctx: &mut Rltk) {
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();
        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.order.cmp(&a.1.order));
        for (pos, render) in data {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] || DEBUG_RENDER_ALL {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }

    fn run_map_indexing_system(&mut self) {
        let mut mis = MapIndexingSystem{};
        mis.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn run_pregame_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
    }

    fn run_player_turn_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut pickups = ItemCollectionSystem{};
        pickups.run_now(&self.ecs);
        let mut items = ItemUseSystem{};
        items.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        DamageSystem::clean_up_the_dead(&mut self.ecs);
        self.ecs.maintain();
    }

    fn run_monster_turn_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterMovementSystem{};
        mob.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        DamageSystem::clean_up_the_dead(&mut self.ecs);
        self.ecs.maintain();
    }

    #[allow(dead_code)]
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
        draw_map(&self.ecs, ctx);
        self.render_all(ctx);
        self::draw_ui(&self.ecs, ctx);

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreGame => {
                self.run_pregame_systems();
                self.run_map_indexing_system();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_player_turn_systems();
                self.run_map_indexing_system();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_monster_turn_systems();
                self.run_map_indexing_system();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(&mut self.ecs, ctx);
                match result {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {},
                    ItemMenuResult::Selected {item: item} => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(),
                            WantsToUseItem{item: item}
                        ).expect("Unable to insert intent to drink potion.");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
        }
        let mut runwriter = self.ecs.write_resource::<RunState>();
        *runwriter = newrunstate;
    }

}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_fps_cap(60.0)
        .with_title("Roguelike Tutorial")
        .build()?;
    // context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
    };

    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<MonsterMovementAI>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMeleeAttack>();
    gs.ecs.register::<ApplyMeleeDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<ProvidesHealing>();

    let map = Map::new_rooms_and_corridors();
    let (px, py) = map.rooms[0].center();

    // Spawning the player is deterministic, so no RNG is needed.
    let player = spawner::spawn_player(&mut gs.ecs, px, py);
    gs.ecs.insert(player);

    // We need to insert the RNG here so spawning logic can make use of it.
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(RunState::PreGame);
    gs.ecs.insert(GameLog::new());
    gs.ecs.insert(Point::new(px, py)); // Player position.

    rltk::main_loop(context, gs)
}