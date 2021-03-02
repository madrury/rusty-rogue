use std::{thread, time};

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
mod status_system;
use status_system::*;
mod melee_combat_system;
use melee_combat_system::*;
mod damage_system;
use damage_system::*;
mod inventory_system;
use inventory_system::*;
mod map_indexing_system;
use map_indexing_system::*;
mod particle_system;
use particle_system::*;
mod animation_system;
use animation_system::*;
mod gamelog;
use gamelog::{GameLog};

// Render all entities all the time.
const DEBUG_RENDER_ALL: bool = false;


#[derive(PartialEq)]
pub enum RunState {
    MainMenu {selection: MenuResult},
    PreGame,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    ShowUseInventory,
    ShowThrowInventory,
    ShowTargeting {range: i32, item: Entity},
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
        let mut uses = ItemUseSystem{};
        uses.run_now(&self.ecs);
        let mut throws = ItemThrowSystem{};
        throws.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        let mut new_animations = AnimationInitSystem{};
        new_animations.run_now(&self.ecs);
        let mut new_particles = ParticleInitSystem{};
        new_particles.run_now(&self.ecs);
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
        let mut status = MonsterStatusSystem{};
        status.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        let mut new_animations = AnimationInitSystem{};
        new_animations.run_now(&self.ecs);
        let mut new_particles = ParticleInitSystem{};
        new_particles.run_now(&self.ecs);
        DamageSystem::clean_up_the_dead(&mut self.ecs);
        self.ecs.maintain();
    }

    fn run_particle_render_systems(&mut self) {
        let mut particles = ParticleRenderSystem{};
        particles.run_now(&self.ecs);
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
        update_particle_lifetimes(&mut self.ecs, ctx);
        draw_map(&self.ecs, ctx);
        self.render_all(ctx);
        self.run_particle_render_systems();
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
                // We add a quick pause at the start of a monster turn, which
                // helps with game feel.
                thread::sleep(time::Duration::from_millis(50));
                self.run_monster_turn_systems();
                self.run_map_indexing_system();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowUseInventory => {
                let result = gui::show_inventory::<Useable>(&mut self.ecs, ctx, "Useable");
                match result {
                    MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    MenuResult::NoResponse => {},
                    MenuResult::Selected {item} => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(), // Player.
                            WantsToUseItem{item: item}
                        ).expect("Unable to insert intent to use item.");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowThrowInventory => {
                let result = gui::show_inventory::<Throwable>(&mut self.ecs, ctx, "Throwable");
                match result {
                    MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    MenuResult::NoResponse => {},
                    MenuResult::Selected {item} => {
                        newrunstate = RunState::ShowTargeting {range: 4, item: item};
                    }
                }
            }
            RunState::ShowTargeting {range, item} => {
                match gui::ranged_target(&mut self.ecs, ctx, range) {
                    TargetingResult::Cancel => newrunstate = RunState::AwaitingInput,
                    TargetingResult::NoResponse => {},
                    TargetingResult::Selected {pos} => {
                        let mut intent = self.ecs.write_storage::<WantsToThrowItem>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(), // Player.
                            WantsToThrowItem {item: item, target: pos}
                        ).expect("Unable to insert intent to throw item.");
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

    let mut gs = State {
        ecs: World::new(),
    };

    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<MonsterMovementAI>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<ApplyDamage>();
    gs.ecs.register::<Useable>();
    gs.ecs.register::<Throwable>();
    gs.ecs.register::<PickUpable>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToMeleeAttack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToThrowItem>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<AreaOfEffectWhenThrown>();
    gs.ecs.register::<InflictsDamageWhenThrown>();
    gs.ecs.register::<InflictsFreezingWhenThrown>();
    gs.ecs.register::<InflictsBurningWhenThrown>();
    gs.ecs.register::<StatusIsFrozen>();
    gs.ecs.register::<StatusIsBurning>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<AreaOfEffectAnimationWhenThrown>();

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
    gs.ecs.insert(AnimationBuilder::new());
    gs.ecs.insert(ParticleBuilder::new());
    gs.ecs.insert(Point::new(px, py)); // Player position.

    rltk::main_loop(context, gs)
}