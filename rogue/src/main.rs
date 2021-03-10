use rltk::{GameState, Rltk, Point};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder, SimpleMarkerAllocator};

mod components;
pub use components::*;
pub mod map_builders;
mod map;
pub use map::*;
mod spawner;
mod save_load;
mod random_table;
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
mod targeted_effect_system;
use targeted_effect_system::*;
mod untargeted_effect_system;
use untargeted_effect_system::*;
mod map_indexing_system;
use map_indexing_system::*;
mod particle_system;
use particle_system::*;
mod animation_system;
use animation_system::*;
mod hunger_system;
use hunger_system::*;
mod spell_charge_system;
use spell_charge_system::*;
mod gamelog;
use gamelog::{GameLog};

// Render all entities all the time.
const DEBUG_RENDER_ALL: bool = false;


#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    MainMenu {current: MainMenuSelection},
    SaveGame,
    PreGame,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    ShowUseInventory,
    ShowThrowInventory,
    ShowEquipInventory,
    ShowSpellbook,
    ShowTargetingMouse {
        range: i32,
        radius: Option<i32>,
        thing: Entity
    },
    ShowTargetingKeyboard {
        range: i32,
        radius: Option<i32>,
        thing: Entity,
        current: Option<Point>
    },
    NextLevel
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
        let mut untargeteds = UntargetedSystem{};
        untargeteds.run_now(&self.ecs);
        let mut targeteds = TargetedSystem{};
        targeteds.run_now(&self.ecs);
        let mut removes = ItemRemoveSystem{};
        removes.run_now(&self.ecs);
        let mut equips = ItemEquipSystem{};
        equips.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        let mut hunger = HungerSystem{};
        hunger.run_now(&self.ecs);
        let mut charges = SpellChargeSystem{};
        charges.run_now(&self.ecs);
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

    fn generate_map(&mut self, depth: i32) {
        let mut builder = map_builders::random_builder(depth);
        builder.build_map();
        let player_start;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.map();
            player_start = builder.starting_position();
        }
        builder.spawn_entities(&mut self.ecs);

        // Place the player and update the player's associated ECS resources.
        let (player_x, player_y) = (player_start.x, player_start.y);
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }
    }

    fn descend_level(&mut self) {
        let to_delete = self.entities_to_delete_when_descending();
        for target in to_delete {
            self.ecs.delete_entity(target)
                .expect("Unable to delete entity when descending.");
        }

        let current_depth;
        {
            let map_resource = self.ecs.fetch::<Map>();
            current_depth = map_resource.depth;
        }
        self.generate_map(current_depth + 1);

        let mut log = self.ecs.fetch_mut::<gamelog::GameLog>();
        log.entries.push("You descend.".to_string());
    }

    fn entities_to_delete_when_descending(&mut self) -> Vec<Entity> {
        // This is our keeplist. Eveything but the player and what they are
        // holding is going to be removed.
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let spellbook = self.ecs.read_storage::<InSpellBook>();
        let equipped = self.ecs.read_storage::<Equipped>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let delete_me = player.get(entity).is_none()
                && backpack.get(entity).is_none()
                && backpack.get(entity).map_or(true, |b| b.owner != *player_entity)
                && spellbook.get(entity).is_none()
                && spellbook.get(entity).map_or(true, |b| b.owner != *player_entity)
                && equipped.get(entity).is_none()
                && equipped.get(entity).map_or(true, |e| e.owner != *player_entity);
            if delete_me {
                to_delete.push(entity);
            }
        }
        to_delete
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

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        // Guard against rendering the game map if we are in the main menu.
        match newrunstate {
            RunState::MainMenu {..} => {},
            // We're playng the game.
            _ => {
                update_particle_lifetimes(&mut self.ecs, ctx);
                self.run_particle_render_systems();
                draw_map(&self.ecs, ctx);
                self.render_all(ctx);
                self::draw_ui(&self.ecs, ctx)
             }
        }

        // The big switch.
        match newrunstate {
            RunState::MainMenu {..} => {
                let result = gui::main_menu(&mut self.ecs, ctx);
                match result {
                    MainMenuResult::NoSelection {current} =>
                        newrunstate = RunState::MainMenu {current},
                    MainMenuResult::Selected {selected} => match selected {
                        MainMenuSelection::NewGame => newrunstate = RunState::PreGame,
                        MainMenuSelection::LoadGame => {
                            save_load::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                        }
                        MainMenuSelection::Quit => std::process::exit(0)
                    }
                }
            }
            RunState::SaveGame {} => {
                save_load::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    current: MainMenuSelection::LoadGame
                };
            },
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
            RunState::ShowUseInventory => {
                let result = gui::show_inventory::<Useable>(&mut self.ecs, ctx, "Useable");
                match result {
                    MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    MenuResult::NoResponse => {},
                    MenuResult::Selected {thing} => {
                        let mut intent = self.ecs.write_storage::<WantsToUseUntargeted>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(), // Player.
                            WantsToUseUntargeted {thing: thing}
                        ).expect("Unable to insert intent to use item.");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowEquipInventory => {
                let result = gui::show_inventory::<Equippable>(&mut self.ecs, ctx, "Equippable");
                match result {
                    MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    MenuResult::NoResponse => {},
                    MenuResult::Selected {thing} => {
                        let equippables = self.ecs.read_storage::<Equippable>();
                        let equipped = self.ecs.read_storage::<Equipped>();
                        let equipment = equippables.get(thing).unwrap(); // We can only get here if the item is equippable.
                        let player_entity = self.ecs.read_resource::<Entity>();
                        let is_equipped = equipped
                            .get(thing)
                            .map_or(false, |e| e.owner == *player_entity);
                        if is_equipped {
                            let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                            intent.insert(
                                *self.ecs.fetch::<Entity>(), // Player.
                                WantsToRemoveItem {item: thing, slot: equipment.slot}
                            ).expect("Unable to insert intent to remove item.");
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToEquipItem>();
                            intent.insert(
                                *self.ecs.fetch::<Entity>(), // Player.
                                WantsToEquipItem {item: thing, slot: equipment.slot}
                            ).expect("Unable to insert intent to equip item.");
                        }
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowThrowInventory => {
                let result = gui::show_inventory::<Throwable>(&mut self.ecs, ctx, "Throwable");
                match result {
                    MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    MenuResult::NoResponse => {},
                    MenuResult::Selected {thing} => {
                        // We need the area of effect radius of the item to draw
                        // the targeting system.
                        let aoes = self.ecs.read_storage::<AreaOfEffectWhenTargeted>();
                        let radius = aoes.get(thing).map(|x| x.radius);
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range: 6,
                            thing: thing,
                            radius: radius,
                            current: None
                        };
                    }
                }
            }
            RunState::ShowSpellbook => {
                let result = gui::show_spellbook(&mut self.ecs, ctx);
                match result {
                    MenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    MenuResult::NoResponse => {},
                    MenuResult::Selected {thing} => {
                        let targeted = self.ecs.read_storage::<Targeted>();
                        let is_targeted = targeted.get(thing).is_some();
                        if is_targeted {
                            let aoes = self.ecs.read_storage::<AreaOfEffectWhenTargeted>();
                            let radius = aoes.get(thing).map(|x| x.radius);
                            newrunstate = RunState::ShowTargetingKeyboard {
                                range: 6,
                                thing: thing,
                                radius: radius,
                                current: None
                            };
                        }
                        let untargeted = self.ecs.read_storage::<Untargeted>();
                        let is_untargeted = untargeted.get(thing).is_some();
                        if is_untargeted {
                            let mut intent = self.ecs.write_storage::<WantsToUseUntargeted>();
                            intent.insert(
                                *self.ecs.fetch::<Entity>(), // Player.
                                WantsToUseUntargeted {thing: thing}
                            ).expect("Unable to insert intent to use item.");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowTargetingMouse {range, thing, radius} => {
                match gui::ranged_target_mouse(&mut self.ecs, ctx, range, radius) {
                    TargetingResult::Cancel => newrunstate = RunState::AwaitingInput,
                    TargetingResult::SwitchModality => {
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range: range, thing: thing, radius: radius, current: None
                        }
                    }
                    TargetingResult::Selected {pos} => {
                        let mut intent = self.ecs.write_storage::<WantsToUseTargeted>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(), // Player.
                            WantsToUseTargeted {thing: thing, target: pos}
                        ).expect("Unable to insert intent to throw item.");
                        newrunstate = RunState::PlayerTurn;
                    },
                    _ => {},
                }
            }
            RunState::ShowTargetingKeyboard {range, thing, radius, current} => {
                match gui::ranged_target_keyboard(&mut self.ecs, ctx, range, radius, current) {
                    TargetingResult::Cancel => newrunstate = RunState::AwaitingInput,
                    TargetingResult::SwitchModality => {
                        newrunstate = RunState::ShowTargetingMouse {
                            range: range, thing: thing, radius: radius
                        }
                    },
                    TargetingResult::MoveCursor {pos} => {
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range: range, thing: thing, radius: radius, current: Some(pos)
                        }
                    },
                    TargetingResult::Selected {pos} => {
                        let mut intent = self.ecs.write_storage::<WantsToUseTargeted>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(), // Player.
                            WantsToUseTargeted {thing: thing, target: pos}
                        ).expect("Unable to insert intent to use targeted item.");
                        newrunstate = RunState::PlayerTurn;
                    },
                    TargetingResult::NoResponse => {},
                }
            }
            RunState::NextLevel => {
                self.descend_level();
                newrunstate = RunState::PreGame;
            }
        }

        let mut runwriter = self.ecs.write_resource::<RunState>();
        *runwriter = newrunstate;
    }
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        // .with_fps_cap(120.0)
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
    };

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();

    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<Useable>();
    gs.ecs.register::<Throwable>();
    gs.ecs.register::<PickUpable>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Castable>();
    gs.ecs.register::<Targeted>();
    gs.ecs.register::<Untargeted>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InSpellBook>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<MonsterMovementAI>();
    gs.ecs.register::<SpellCharges>();
    gs.ecs.register::<WantsToMeleeAttack>();
    gs.ecs.register::<WantsToTakeDamage>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseUntargeted>();
    gs.ecs.register::<WantsToUseTargeted>();
    gs.ecs.register::<WantsToEquipItem>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<WantsToMoveToRandomPosition>();
    gs.ecs.register::<ProvidesFullHealing>();
    gs.ecs.register::<ProvidesFullFood>();
    gs.ecs.register::<IncreasesMaxHpWhenUsed>();
    gs.ecs.register::<MovesToRandomPosition>();
    gs.ecs.register::<AreaOfEffectWhenTargeted>();
    gs.ecs.register::<InflictsDamageWhenTargeted>();
    gs.ecs.register::<InflictsFreezingWhenTargeted>();
    gs.ecs.register::<InflictsBurningWhenTargeted>();
    gs.ecs.register::<GrantsMeleeAttackBonus>();
    gs.ecs.register::<GrantsMeleeDefenseBonus>();
    gs.ecs.register::<StatusIsFrozen>();
    gs.ecs.register::<StatusIsBurning>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<AreaOfEffectAnimationWhenTargeted>();

    // Placeholder values which we will replace upon map generation.
    gs.ecs.insert(Point::new(0, 0)); // Player position.
    gs.ecs.insert(Map::new(1));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(
        RunState::MainMenu {current: MainMenuSelection::NewGame}
    );
    gs.ecs.insert(GameLog::new());
    gs.ecs.insert(AnimationBuilder::new());
    gs.ecs.insert(ParticleBuilder::new());

    let player = spawner::spawn_player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player);

    gs.generate_map(1);

    rltk::main_loop(context, gs)
}