use rltk::{GameState, Rltk, Point, RGB};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder, SimpleMarkerAllocator};

mod components;
use components::*;
use components::animation::*;
use components::hunger::*;
use components::swimming::*;
use components::magic::*;
use components::equipment::*;
use components::ai::*;
use components::game_effects::*;
use components::spawn_despawn::*;
use components::status_effects::*;
use components::signaling::*;

pub mod map_builders;
pub mod entity_spawners;
pub mod terrain_spawners;
pub mod noise;
pub mod color;
mod save_load;
mod random_table;
mod map;
pub use map::*;
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
mod swimming_system;
use swimming_system::*;
mod spell_charge_system;
use spell_charge_system::*;
mod entity_spawn_system;
use dissipation_system::*;
mod dissipation_system;
use entity_spawn_system::*;
mod encroachment_system;
use encroachment_system::*;
mod general_movement_system;
use general_movement_system::*;
mod gamelog;
use gamelog::{GameLog};

// Debug flags.
const DEBUG_DRAW_ALL_MAP: bool = true;
const DEBUG_RENDER_ALL: bool = true;
const DEBUG_VISUALIZE_MAPGEN: bool = false;
const DEBUG_HIGHLIGHT_STAIRS: bool = true;
const DEBUG_HIGHLIGHT_FLOOR: bool = false;
const DEBUG_HIGHLIGHT_FIRE: bool = false;

const MAPGEN_FRAME_TIME: f32 = 50.0;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    MainMenu {current: MainMenuSelection},
    MapGeneration,
    SaveGame,
    PreGame,
    AwaitingInput,
    PlayerTurn,
    HazardTurn,
    MonsterTurn,
    UpkeepTrun,
    ShowMagicSelectionMenu,
    ShowUseInventory,
    ShowThrowInventory,
    ShowEquipInventory,
    ShowSpellbook,
    ShowTargetingMouse {
        range: f32,
        kind: TargetingKind,
        thing: Entity
    },
    ShowTargetingKeyboard {
        range: f32,
        kind: TargetingKind,
        thing: Entity,
        current: Option<Point>
    },
    PlayingAnimation,
    NextLevel
}


pub struct MapGenerationAnimation {
    history : Vec<Map>,
    index : usize,
    timer : f32
}
impl MapGenerationAnimation {
    pub fn reset(&mut self) {
        self.history.clear();
        self.index = 0;
        self.timer = 0.0;
    }
}

pub struct State {
    pub ecs: World,
    pub next_state : Option<RunState>,
    pub mapgen: MapGenerationAnimation
}

impl State {

    // Draw all entities with a Renderable component on the console.
    fn render_all(&self, ctx: &mut Rltk) {
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let sets_bg = self.ecs.read_storage::<SetsBgColor>();
        let map = self.ecs.fetch::<Map>();
        // First loop through the entities in render order and draw them all.
        let mut render_data = (&positions, &renderables).join().collect::<Vec<_>>();
        render_data.sort_by(|&a, &b| b.1.order.cmp(&a.1.order));
        for (pos, render) in render_data {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] || DEBUG_RENDER_ALL {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            } else if map.revealed_tiles[idx] && render.visible_out_of_fov {
                ctx.set(pos.x, pos.y, render.fg.to_greyscale(), render.bg.to_greyscale(), render.glyph);
            }
        }
        // Then loop through the enetities that override bg colors, and fill in
        // the backgrounds of their tiles.
        let mut bg_data = (&positions, &renderables, &sets_bg).join().collect::<Vec<_>>();
        bg_data.sort_by(|&a, &b| b.2.order.cmp(&a.2.order));
        for (pos, render, _bg) in bg_data {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] || DEBUG_RENDER_ALL {
                ctx.set_bg(pos.x, pos.y, render.bg);
            } else if map.revealed_tiles[idx] && render.visible_out_of_fov {
                ctx.set_bg(pos.x, pos.y, render.bg.to_greyscale());
            }
        }
        if DEBUG_HIGHLIGHT_STAIRS {
            self.debug_highlight_stairs(ctx);
        }
        if DEBUG_HIGHLIGHT_FIRE {
            self.debug_highlight_fire(ctx);
        }
        if DEBUG_HIGHLIGHT_FLOOR {
            self.debug_highlight_floor(ctx);
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

    fn run_cleanup_systems(&mut self) {
        let mut pos = PositionMovementSystem {};
        pos.run_now(&self.ecs);
        DissipationSystem::clean_up_dissipated_entities(&mut self.ecs);
        DamageSystem::clean_up_the_dead(&mut self.ecs);
        process_entity_spawn_request_buffer(&mut self.ecs);
        self.ecs.maintain();
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
        let mut status = StatusEffectSystem{};
        status.run_now(&self.ecs);
        let mut hunger = HungerSystem{};
        hunger.run_now(&self.ecs);
        let mut swimming = SwimmingSystem{};
        swimming.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        let mut teleport = TeleportationSystem{};
        teleport.run_now(&self.ecs);
        process_entity_spawn_request_buffer(&mut self.ecs);
        let mut new_animations = AnimationInitSystem{};
        new_animations.run_now(&self.ecs);
        let mut new_particles = ParticleInitSystem{};
        new_particles.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn run_hazard_turn_systems(&mut self) {
        let mut encroachment = EncroachmentSystem{};
        encroachment.run_now(&self.ecs);
        let mut status_effects = StatusEffectSystem{};
        status_effects.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        let mut dissipates = DissipationSystem{};
        dissipates.run_now(&self.ecs);
        let mut spawns = EntitySpawnSystem{};
        spawns.run_now(&self.ecs);
        process_entity_spawn_request_buffer(&mut self.ecs);
        self.ecs.maintain();
    }

    fn run_monster_turn_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut can_act = MonsterCanActSystem{};
        can_act.run_now(&self.ecs);
        let mut mob = MonsterBasicAISystem{};
        mob.run_now(&self.ecs);
        let mut supports = MonsterSupportSpellcasterAISystem{};
        supports.run_now(&self.ecs);
        let mut spellcasters = MonsterAttackSpellcasterAISystem{};
        spellcasters.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut targeteds = TargetedSystem{};
        targeteds.run_now(&self.ecs);
        let mut status = StatusEffectSystem{};
        status.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        let mut teleport = TeleportationSystem{};
        teleport.run_now(&self.ecs);
        let mut spawns = EntitySpawnSystem{};
        spawns.run_now(&self.ecs);
        process_entity_spawn_request_buffer(&mut self.ecs);
        let mut new_animations = AnimationInitSystem{};
        new_animations.run_now(&self.ecs);
        let mut new_particles = ParticleInitSystem{};
        new_particles.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn run_upkeep_turn_systems(&mut self) {
        let mut status = StatusTickSystem{};
        status.run_now(&self.ecs);
        let mut charges = SpellChargeSystem{};
        charges.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn run_particle_render_systems(&mut self) {
        let mut particles = ParticleRenderSystem{};
        particles.run_now(&self.ecs);
    }

    fn generate_map(&mut self, depth: i32) {
        // Build the floor layout, and update the dubug map build animation.
        self.mapgen.reset();
        let mut builder = map_builders::random_builder(depth);
        builder.build_map();
        self.mapgen.history = builder.snapshot_history();

        // Place the built map into the ecs. From here on, we will work with the
        // copy of the map stored in the ECS.
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.map();
        }

        // Place the stairs and update the associated ECS resources.
        let stairs_position = builder.stairs_position(&self.ecs);
        {
            let mut map = self.ecs.write_resource::<Map>();
            let idx = map.xy_idx(stairs_position.x, stairs_position.y);
            map.tiles[idx] = TileType::DownStairs;
            map.blocked[idx] = false;
            map.ok_to_spawn[idx] = false;
        }

        builder.spawn_terrain(&mut self.ecs);
        builder.spawn_entities(&mut self.ecs);

        // Add all our newly spawned enettities to the tile_content array. This
        // is used below when attempting to place the player.
        {
            let mut map = self.ecs.write_resource::<Map>();
            let entities = self.ecs.entities();
            let positions = self.ecs.read_storage::<Position>();
            for (e, pos) in (&entities, &positions).join() {
                let idx = map.xy_idx(pos.x, pos.y);
                map.tile_content[idx].push(e)
            }
        }

        // Compute a position to place the player. We're only computing the
        // position here, so this is stateless, we don't actually update any ECS
        // state yet.
        let mut player_start_position = Point {x: 0, y: 0};
        loop {
            let map = self.ecs.read_resource::<Map>();
            player_start_position = builder.starting_position(&self.ecs);
            if map.is_reachable(player_start_position, stairs_position) {
                break;
            }
        }

        // Place the player and update the player's associated ECS resources.
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = player_start_position.clone();
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_start_position.x;
            player_pos_comp.y = player_start_position.y;
        }

        // Mark the player's visibility as dirty so we re-compute their viewshed
        // on the first turn.
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
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let spellbook = self.ecs.read_storage::<InSpellBook>();
        let equipped = self.ecs.read_storage::<Equipped>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let delete_me = player.get(entity).is_none()
                && backpack.get(entity).map_or(true, |b| b.owner != *player_entity)
                && spellbook.get(entity).map_or(true, |b| b.owner != *player_entity)
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

    #[allow(dead_code)]
    fn debug_highlight_stairs(&self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::DownStairs {
                    ctx.set_bg(x, y, RGB::named(rltk::PINK));
                }
            }
        }
    }

    #[allow(dead_code)]
    fn debug_highlight_floor(&self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    ctx.set_bg(x, y, RGB::named(rltk::PURPLE));
                }
            }
        }
    }

    #[allow(dead_code)]
    fn debug_highlight_fire(&self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                if map.fire[idx] {
                    ctx.set_bg(x, y, RGB::named(rltk::PURPLE));
                }
            }
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
                draw_map(&self.ecs.fetch::<Map>(), ctx);
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
            RunState::MapGeneration => {
                if !DEBUG_VISUALIZE_MAPGEN {
                    newrunstate = self.next_state
                        .expect("Attempting to return from mapgen, but no next state.");
                } else {
                    ctx.cls();
                    draw_map(&self.mapgen.history[self.mapgen.index], ctx);
                    self.mapgen.timer += ctx.frame_time_ms;
                    if self.mapgen.timer > MAPGEN_FRAME_TIME {
                        self.mapgen.timer = 0.0;
                        self.mapgen.index += 1;
                        if self.mapgen.index >= self.mapgen.history.len() {
                            newrunstate = self.next_state
                                .expect("Attempting to return from mapgen, but no next state.");
                        }
                    }
                }
            }
            RunState::SaveGame {} => {
                save_load::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    current: MainMenuSelection::LoadGame
                };
            }
            RunState::PreGame => {
                self.run_pregame_systems();
                self.run_map_indexing_system();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayingAnimation => {
                if is_any_animation_alive(&self.ecs) {
                    newrunstate = RunState::PlayingAnimation;
                } else {
                    self.run_cleanup_systems();
                    self.run_map_indexing_system();
                    let next_state = self.next_state
                        .expect("Returning from animation, but no next_state to return to.");
                    newrunstate = next_state;
                }
            }
            RunState::PlayerTurn => {
                self.run_player_turn_systems();
                if is_any_animation_alive(&self.ecs) {
                    self.next_state = Some(RunState::HazardTurn);
                    newrunstate = RunState::PlayingAnimation;
                } else {
                    self.run_cleanup_systems();
                    self.run_map_indexing_system();
                    newrunstate = RunState::HazardTurn;
                }
            }
            RunState::HazardTurn => {
                self.run_hazard_turn_systems();
                let nextstate = match is_player_encroaching_blessing_tile(&self.ecs) {
                    true => RunState::ShowMagicSelectionMenu,
                    false => RunState::MonsterTurn,
                };
                if is_any_animation_alive(&self.ecs) {
                    self.next_state = Some(nextstate);
                    newrunstate = RunState::PlayingAnimation;
                } else {
                    self.run_cleanup_systems();
                self.run_map_indexing_system();
                    newrunstate = nextstate;
                }
            }
            RunState::MonsterTurn => {
                self.run_monster_turn_systems();
                if is_any_animation_alive(&self.ecs) {
                    self.next_state = Some(RunState::UpkeepTrun);
                    newrunstate = RunState::PlayingAnimation;
                } else {
                    self.run_cleanup_systems();
                    self.run_map_indexing_system();
                    newrunstate = RunState::UpkeepTrun
                }
            }
            RunState::UpkeepTrun => {
                self.run_upkeep_turn_systems();
                self.run_map_indexing_system();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowMagicSelectionMenu => {
                println!("Player recieves a blessing!");
                newrunstate = RunState::MonsterTurn;
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
                        let targeted = self.ecs.read_storage::<Targeted>();
                        let (range, kind) = targeted.get(thing)
                            .map(|x| (x.range, x.kind))
                            .expect("Attempted to throw item without Targeted components.");
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range: range,
                            thing: thing,
                            kind: kind,
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
                            let (range, kind) = targeted.get(thing)
                                .map(|x| (x.range, x.kind))
                                .expect("Attempted to throw item without Targeted components.");
                            newrunstate = RunState::ShowTargetingKeyboard {
                                range: range,
                                thing: thing,
                                kind: kind,
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
            RunState::ShowTargetingMouse {range, kind, thing} => {
                match gui::ranged_target_mouse(&mut self.ecs, ctx, range, kind) {
                    TargetingResult::Cancel => newrunstate = RunState::AwaitingInput,
                    TargetingResult::SwitchModality => {
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range: range, thing: thing, kind: kind, current: None
                        }
                    }
                    TargetingResult::Selected {pos} => {
                        let mut intent = self.ecs.write_storage::<WantsToUseTargeted>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(), // Player.
                            WantsToUseTargeted {thing: thing, target: pos}
                        ).expect("Unable to insert intent to throw item.");
                        newrunstate = RunState::PlayerTurn;
                    }
                    _ => {}
                }
            }
            RunState::ShowTargetingKeyboard {range, kind, thing, current} => {
                match gui::ranged_target_keyboard(&mut self.ecs, ctx, range, kind, current) {
                    TargetingResult::Cancel => newrunstate = RunState::AwaitingInput,
                    TargetingResult::SwitchModality => {
                        newrunstate = RunState::ShowTargetingMouse {
                            range: range, thing: thing, kind: kind
                        }
                    },
                    TargetingResult::MoveCursor {pos} => {
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range: range, thing: thing, kind: kind, current: Some(pos)
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
    let context = RltkBuilder::new()
        .with_fps_cap(60.0)
        .with_resource_path("resources".to_string())
        .with_font("terminal8x8.png".to_string(), 8, 8)
        .with_simple8x8()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        next_state: Some(RunState::MainMenu{
            current: gui::MainMenuSelection::NewGame
        }),
        mapgen: MapGenerationAnimation {
            index: 0,
            history: Vec::new(),
            timer: 0.0
        }
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
    gs.ecs.register::<SetsBgColor>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<SwimStamina>();
    gs.ecs.register::<Useable>();
    gs.ecs.register::<Throwable>();
    gs.ecs.register::<PickUpable>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Opaque>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Castable>();
    gs.ecs.register::<Targeted>();
    gs.ecs.register::<Untargeted>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InSpellBook>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<BlessingOrbBag>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Hazard>();
    gs.ecs.register::<BlessingOrb>();
    gs.ecs.register::<BlessingSelectionTile>();
    gs.ecs.register::<IsEntityKind>();
    gs.ecs.register::<CanAct>();
    gs.ecs.register::<CanNotAct>();
    gs.ecs.register::<MonsterMovementRoutingOptions>();
    gs.ecs.register::<MonsterBasicAI>();
    gs.ecs.register::<MonsterAttackSpellcasterAI>();
    gs.ecs.register::<MonsterSupportSpellcasterAI>();
    gs.ecs.register::<SpellCharges>();
    gs.ecs.register::<WantsToMeleeAttack>();
    gs.ecs.register::<WantsToTakeDamage>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseUntargeted>();
    gs.ecs.register::<WantsToUseTargeted>();
    gs.ecs.register::<WantsToEquipItem>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<WantsToMoveToPosition>();
    gs.ecs.register::<WantsToMoveToRandomPosition>();
    gs.ecs.register::<WantsToDissipate>();
    gs.ecs.register::<DissipateWhenBurning>();
    gs.ecs.register::<DissipateWhenEnchroachedUpon>();
    gs.ecs.register::<ProvidesFullHealing>();
    gs.ecs.register::<ProvidesFullFood>();
    gs.ecs.register::<ProvidesFullSpellRecharge>();
    gs.ecs.register::<IncreasesMaxHpWhenUsed>();
    gs.ecs.register::<DecreasesSpellRechargeWhenUsed>();
    gs.ecs.register::<ProvidesFireImmunityWhenUsed>();
    gs.ecs.register::<ProvidesChillImmunityWhenUsed>();
    gs.ecs.register::<MovesToRandomPosition>();
    gs.ecs.register::<InflictsDamageWhenTargeted>();
    gs.ecs.register::<InflictsDamageWhenEncroachedUpon>();
    gs.ecs.register::<RemoveBurningWhenEncroachedUpon>();
    gs.ecs.register::<DissipateFireWhenEncroachedUpon>();
    gs.ecs.register::<RemoveBurningOnUpkeep>();
    gs.ecs.register::<InflictsFreezingWhenTargeted>();
    gs.ecs.register::<InflictsBurningWhenTargeted>();
    gs.ecs.register::<MoveToPositionWhenTargeted>();
    gs.ecs.register::<BuffsMeleeAttackWhenTargeted>();
    gs.ecs.register::<BuffsPhysicalDefenseWhenTargeted>();
    gs.ecs.register::<InflictsBurningWhenEncroachedUpon>();
    gs.ecs.register::<InflictsFreezingWhenEncroachedUpon>();
    gs.ecs.register::<SpawnsEntityInAreaWhenTargeted>();
    gs.ecs.register::<SpawnEntityWhenEncroachedUpon>();
    gs.ecs.register::<SpawnEntityWhenMeleeAttacked>();
    gs.ecs.register::<SpawnEntityWhenKilled>();
    gs.ecs.register::<ChanceToSpawnAdjacentEntity>();
    gs.ecs.register::<ChanceToSpawnEntityWhenBurning>();
    gs.ecs.register::<ChanceToInflictBurningOnAdjacentEntities>();
    gs.ecs.register::<ChanceToDissipate>();
    gs.ecs.register::<SkipRandomDissipationForOneTurn>();
    gs.ecs.register::<GrantsMeleeAttackBonus>();
    gs.ecs.register::<GrantsMeleeDefenseBonus>();
    gs.ecs.register::<StatusIsFrozen>();
    gs.ecs.register::<StatusIsBurning>();
    gs.ecs.register::<StatusIsImmuneToFire>();
    gs.ecs.register::<StatusIsImmuneToChill>();
    gs.ecs.register::<StatusIsMeleeAttackBuffed>();
    gs.ecs.register::<StatusIsPhysicalDefenseBuffed>();
    gs.ecs.register::<GameAnimationParticle>();
    gs.ecs.register::<AreaOfEffectAnimationWhenTargeted>();
    gs.ecs.register::<AlongRayAnimationWhenTargeted>();

    // Placeholder values which we will replace upon map generation.
    gs.ecs.insert(Point::new(0, 0)); // Player position.
    gs.ecs.insert(Map::new(1));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(GameLog::new());
    gs.ecs.insert(AnimationRequestBuffer::new());
    gs.ecs.insert(ParticleRequestBuffer::new());
    gs.ecs.insert(EntitySpawnRequestBuffer::new());

    // Create the player entity. Note that we're not computing the correct
    // position to place the player here, since that needs to happen after we've
    // generated terrain and monsters.
    let player = entity_spawners::player::spawn_player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player);
    gs.ecs.insert(RunState::MapGeneration {});

    gs.generate_map(1);

    rltk::main_loop(context, gs)
}