use rltk::{GameState, Rltk, Point};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder, SimpleMarkerAllocator};

mod components;
pub use components::*;
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

    fn descend_level(&mut self) {
        let to_delete = self.entities_to_delete_when_descending();
        for target in to_delete {
            self.ecs.delete_entity(target)
                .expect("Unable to delete entity when descending.");
        }

        // Create the new Map object and replace the resource in the ECS.
        let newmap;
        let depth;
        {
            let mut map_resource = self.ecs.write_resource::<Map>();
            depth = map_resource.depth;
            *map_resource = Map::new_rooms_and_corridors(depth + 1);
            newmap = map_resource.clone();
        }

        for room in newmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, depth + 1);
        }

        // Spawn the player in the new map, and replace the associated resources
        // in the ECS.
        let (px, py) = newmap.rooms[0].center();
        let mut ppos = self.ecs.write_resource::<Point>();
        *ppos = Point::new(px, py);
        let mut positions = self.ecs.write_storage::<Position>();
        let player = self.ecs.fetch::<Entity>();
        let player_position = positions.get_mut(*player);
        if let Some(player_position) = player_position {
            player_position.x = px;
            player_position.y = py;
        }

        // We're in a new position, so our field of view is dirty.
        let mut viewsheds = self.ecs.write_storage::<Viewshed>();
        let pvs = viewsheds.get_mut(*player);
        if let Some(pvs) = pvs {
            pvs.dirty = true;
        }

        let mut log = self.ecs.fetch_mut::<gamelog::GameLog>();
        log.entries.push("You descend.".to_string());
    }

    fn entities_to_delete_when_descending(&mut self) -> Vec<Entity> {
        // This is our keeplist. Eveything but the player and what they are
        // holding is going to be removed.
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let equipped = self.ecs.read_storage::<Equipped>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let delete_me = player.get(entity).is_none()
                && backpack.get(entity).is_none();
                && backpack.get(entity).map_or(true, |b| b.owner != *player_entity);
                && equipped.get(entity).is_none();
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
                draw_map(&self.ecs, ctx);
                self.render_all(ctx);
                self.run_particle_render_systems();
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

    let map = Map::new_rooms_and_corridors(1);
    let (px, py) = map.rooms[0].center();

    // Spawning the player is deterministic, so no RNG is needed.
    let player = spawner::spawn_player(&mut gs.ecs, px, py);
    gs.ecs.insert(player);

    // We need to insert the RNG here so spawning logic can make use of it.
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room, 1);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(
        RunState::MainMenu {current: MainMenuSelection::NewGame}
    );
    gs.ecs.insert(GameLog::new());
    gs.ecs.insert(AnimationBuilder::new());
    gs.ecs.insert(ParticleBuilder::new());
    gs.ecs.insert(Point::new(px, py)); // Player position.

    rltk::main_loop(context, gs)
}