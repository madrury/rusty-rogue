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
use components::targeting::*;
use components::melee::*;

pub mod entity_spawners;
pub mod terrain_spawners;
mod save_load;
mod random_table;

pub mod map_builders;
use map_builders::{ColorMaps, FgColorMap, BgColorMap};

mod map;
pub use map::*;
mod player;
use player::*;
mod gui;
use gui::*;
mod blessings;
use blessings::*;
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
mod synchronization_systems;
use synchronization_systems::*;
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
mod weapon_special_system;
use weapon_special_system::*;
mod gamelog;
use gamelog::GameLog;

//--------------------------------------------------------------------
// Debug flags.
//--------------------------------------------------------------------
const DEBUG_DRAW_ALL_MAP: bool = false;
const DEBUG_RENDER_ALL: bool = true;
const DEBUG_VISUALIZE_MAPGEN: bool = false;
const DEBUG_HIGHLIGHT_STAIRS: bool = false;
const DEBUG_HIGHLIGHT_BLOCKED: bool = false;
const DEBUG_HIGHLIGHT_FLOOR: bool = false;
const DEBUG_HIGHLIGHT_FIRE: bool = false;
const DEBUG_HIGHLIGHT_DEEP_WATER: bool = false;
const DEBUG_HIGHLIGHT_SHALLOW_WATER: bool = false;
const DEBUG_HIGHLIGHT_GRASS: bool = false;
const DEBUG_HIGHLIGHT_BLOOD: bool = false;
const DEBUG_DESCEND_ANYWHERE: bool = false;

const MAPGEN_FRAME_TIME: f32 = 100.0;

//--------------------------------------------------------------------
// States in the Main State Machine.
//--------------------------------------------------------------------
// The following states are used in a match statement in the main game loop (see
// the `tick` method in the implementation). This is the main state machine of
// the game, responsibile for coordinating the sequence of actions taken by
// entites in the dungeon.
//
// When starting the game, we begin in the MainMenu state, and assuming we
// choose "new game", the state transition diagram is:
//
//        MainMenu (Start)
//        v
//        PreGame
//        v
//   +--->HazardTurn--->ShowBlessingSelectionMenu
//   |    |                         |
//   |    v                         |
//   |    Awaiting Input --> Show{Item|Throw|Equip|Spell|Help}Menu --+
//   |    |                         |             |                  |
//   |    |                         |  ShowTargeting{Keyboard|Mouse} |
//   |    |                         |             |                  |
//   |    v    +-----------------------------------------------------+
//   |    |    v                    |             |
//   |    PlayerTurn <--------------+-------------+
//   |    v
//   |    MonsterTurn
//   |    v
//   +----UpkeepTurn
//
// When descending to a new floor, the state is reset to PreGame and the machine
// begins anew.
//--------------------------------------------------------------------
#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    // Render the main menu with selections: New Game, Load Game, Exit.
    // Wait for player to choose a selection.
    MainMenu {current: MainMenuSelection},
    // Animate the map generation. A debug state only used when the debug flag
    // DEBUG_VISUALIZE_MAPGEN is true.
    MapGeneration,
    // Waiting state while game object are serialized.
    SaveGame,
    // Entered exactly one time each floor to initialize the map. Runs the map
    // indexing system and computes initial visibility of all relevent entities.
    PreGame,
    // A turn for actions taken by non-player, non-monster entities in the
    // dungeon. For example:
    //   - Fire or gas spreads.
    //   - Check encroachment (when two entities occupy the same tile), grass
    //   provides cloaking, fire inflicts burning, chill inflicts frezing, etc.
    HazardTurn,
    // Waiting for player input. Re-entered every frame until input is detected.
    // Can optionally transition this state into the various player menus to use
    // items, cast spells, equip objects, get help, etc.
    AwaitingInput,
    // Post player input systems run. Process immediate consequences of player
    // input.
    PlayerTurn,
    // Monsters take all their turns, then we process immediate consequences of
    // their actions.
    MonsterTurn,
    // Systems that run once ever turn cycle. Tick status effects and spell
    // charges.
    UpkeepTrun,
    // We're rendering the help menu.
    ShowHelpMenu { details: Option<&'static str> },
    // We're rendering the blessing selection menu.
    ShowBlessingSelectionMenu,
    // We're rendering the use (untargeted) item menu.
    ShowUseInventory,
    // We're rendering the throw (targeted) item menu.
    ShowThrowInventory,
    // We're rendering the equip item menu.
    ShowEquipInventory,
    // We're rendering the cast spell menu.
    ShowSpellbook,
    // We're asking the player to select a target for a targeted effect using
    // the mouse.
    ShowTargetingMouse {
        range: f32,
        kind: TargetingKind,
        verb: TargetingVerb,
        thing: Entity,
    },
    // We're asking the player to select a target for a targeted effect using
    // the keyboard.
    ShowTargetingKeyboard {
        range: f32,
        kind: TargetingKind,
        verb: TargetingVerb,
        thing: Entity,
        current: Option<Point>
    },
    // We're playing an animation, no input is allowed. This can be entered from
    // any of the main game states, and buffers returning to the "next" state
    // depending on the state it was entered from.
    PlayingAnimation,
    // Descend. Remove entities from the current floor from the ECS, sample a
    // new map, let's go.
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
        let entities = self.ecs.entities();
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let invisibles = self.ecs.read_storage::<StatusInvisibleToPlayer>();
        let fg_colormaps = self.ecs.read_storage::<UseFgColorMap>();
        let bg_colormaps = self.ecs.read_storage::<UseBgColorMap>();
        let sets_bg = self.ecs.read_storage::<SetsBgColor>();
        let map = self.ecs.fetch::<Map>();
        let cmaps = self.ecs.fetch::<ColorMaps>();
        // First loop through the entities in reverse render order and draw them
        // all. Later calls to ctx.set overwrite previous writes, we'll use this
        // to make a second pass and update background colors as needed.
        let mut render_data = (&entities, &positions, &renderables).join().collect::<Vec<_>>();
        render_data.sort_by(|&a, &b| b.2.order.cmp(&a.2.order));
        for (e, pos, render) in render_data {
            // Bail out early if the entity is invisible.
            let invisible = invisibles.get(e).is_some();
            if invisible { continue; }
            // Lookup the colors for the entity. There are two sourced, the
            // colormaps or a fallback to a fixed color in the entities
            // Renderable component.
            let idx = map.xy_idx(pos.x, pos.y);
            let fgcmap = fg_colormaps.get(e).map_or(FgColorMap::None, |fg| fg.cmap);
            let fg = match fgcmap {
                FgColorMap::None => render.fg,
                _ => cmaps.get_fg_color(idx, fgcmap)
            };
            let bgcmap = bg_colormaps.get(e).map_or(BgColorMap::None, |bg| bg.cmap);
            let bg = match bgcmap {
                BgColorMap::None => render.bg,
                _ => cmaps.get_bg_color(idx, bgcmap)
            };
            // Draw the glyphs on the console.
            if map.visible_tiles[idx] || DEBUG_RENDER_ALL {
                ctx.set(pos.x, pos.y, fg, bg, render.glyph);
            } else if map.revealed_tiles[idx] && render.visible_out_of_fov {
                ctx.set(pos.x, pos.y, fg.to_greyscale(), bg.to_greyscale(), render.glyph);
            }
        }
        // Loop through the enetities that override bg colors, and fill in the
        // backgrounds of their tiles. For example, water should always render
        // its tile background as blue.
        let mut bg_data = (&entities, &positions, &renderables, &sets_bg).join().collect::<Vec<_>>();
        bg_data.sort_by(|&a, &b| b.2.order.cmp(&a.2.order));
        for (e, pos, render, _bg) in bg_data {
            // Bail out early if the entity is invisible.
            let invisible = invisibles.get(e).is_some();
            if invisible { continue; }
            // Render.
            let idx = map.xy_idx(pos.x, pos.y);
            let bgcmap = bg_colormaps.get(e).map_or(BgColorMap::None, |bg| bg.cmap);
            let bg = match bgcmap {
                BgColorMap::None => render.bg,
                _ => cmaps.get_bg_color(idx, bgcmap)
            };
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] || DEBUG_RENDER_ALL {
                ctx.set_bg(pos.x, pos.y, bg);
            } else if map.revealed_tiles[idx] && render.visible_out_of_fov {
                ctx.set_bg(pos.x, pos.y, bg.to_greyscale());
            }
        }

        // DEBUG FLAGS: Highlight entities of various types according to debug
        // flags.
        if DEBUG_HIGHLIGHT_STAIRS {
            self.debug_highlight_stairs(ctx);
        }
        if DEBUG_HIGHLIGHT_BLOCKED {
            self.debug_highlight_blocked(ctx);
        }
        if DEBUG_HIGHLIGHT_FIRE {
            self.debug_highlight_fire(ctx);
        }
        if DEBUG_HIGHLIGHT_SHALLOW_WATER {
            self.debug_highlight_shallow_water(ctx);
        }
        if DEBUG_HIGHLIGHT_DEEP_WATER {
            self.debug_highlight_deep_water(ctx);
        }
        if DEBUG_HIGHLIGHT_GRASS {
            self.debug_highlight_grass(ctx);
        }
        if DEBUG_HIGHLIGHT_FLOOR {
            self.debug_highlight_floor(ctx);
        }
        if DEBUG_HIGHLIGHT_BLOOD {
            self.debug_highlight_blood(ctx);
        }
    }

    fn run_synchronization_systems(&mut self) {
        let mut mss = MapSynchronizationSystem{};
        mss.run_now(&self.ecs);
        let mut css = ColorSynchronizationSystem{};
        css.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn run_pregame_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
    }

    fn run_upkeep_turn_systems(&mut self) {
        // The ordering here is important, but for stupid reasons that should
        // probably be re-evaluated at some point: the EncroachmentSystem run
        // must occur *after* the status tick system.
        //
        // This is because we use StatusInvisibleToPlayer to hide monsters that
        // are occupying deep water or tall grass tiles. The order of operations
        // is:
        //
        //   EncroachmentSystem sees that the monster should be hidden, and
        //   creates the status effect *with timer zero*.
        //   ...Terrain Turn...Player Turn...Monster Turn...
        //   StatusTickSystem sees the timer of zero, the status falls off.
        //   EncroachmentSystem sees...
        //
        // If these run in the *other* order, the status is added and
        // immediately removed, doing nothing.
        //
        // It's important for the StatusTickSystem and the EncroachmentSystem to
        // reside in the same RunState, otherwise at least one frame will be
        // rendered between when the status falls off, and when it is reapplied,
        // causing the "hidden" entity to flicker.
        let mut status = StatusTickSystem{};
        status.run_now(&self.ecs);
        let mut specials = WeaponSpecialTickSystem{};
        specials.run_now(&self.ecs);
        let mut charges = SpellChargeSystem{};
        charges.run_now(&self.ecs);
        let mut encroachment = EncroachmentSystem{};
        encroachment.run_now(&self.ecs);
        let mut new_animations = AnimationParticlizationSystem{};
        new_animations.run_now(&self.ecs);
        let mut new_particles = ParticleInitSystem{};
        new_particles.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn run_hazard_turn_systems(&mut self) {
        let mut status_effects = StatusEffectSystem{};
        status_effects.run_now(&self.ecs);
        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);
        let mut dissipates = DissipationSystem{};
        dissipates.run_now(&self.ecs);
        let mut spawns = EntitySpawnSystem{};
        spawns.run_now(&self.ecs);
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
        let mut new_animations = AnimationParticlizationSystem{};
        new_animations.run_now(&self.ecs);
        let mut new_particles = ParticleInitSystem{};
        new_particles.run_now(&self.ecs);
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
        let mut new_animations = AnimationParticlizationSystem{};
        new_animations.run_now(&self.ecs);
        let mut new_particles = ParticleInitSystem{};
        new_particles.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn run_cleanup_systems(&mut self) {
        let mut pos = PositionMovementSystem {};
        pos.run_now(&self.ecs);
        DissipationSystem::clean_up_dissipated_entities(&mut self.ecs);
        DamageSystem::clean_up_the_dead(&mut self.ecs);
        process_entity_spawn_request_buffer(&mut self.ecs);
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
        let (noisemap, colormap) = builder.build_map();

        // Insert the noisemap and the colormap into he ECS as resources.
        // The noisemap is used for random data needed when spawning entities.
        // The colormap is used durning frame rendering to determine fg and bg
        // colors for some entities.
        self.ecs.insert(noisemap);
        self.ecs.insert(colormap);

        self.mapgen.history = builder.snapshot_history();

        // Place the built map into the ecs. From here on, we will work with the
        // copy of the map stored in the ECS.
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.map();
        }
        builder.spawn_blessing_tile(&mut self.ecs);
        builder.spawn_water(&mut self.ecs);
        builder.spawn_terrain(&mut self.ecs);
        builder.spawn_entities(&mut self.ecs);

        // Place the stairs and update the associated ECS resources.
        let stairs_position = builder.stairs_position(&self.ecs);
        {
            let mut map = self.ecs.write_resource::<Map>();
            let idx = map.xy_idx(stairs_position.x, stairs_position.y);
            map.tiles[idx] = TileType::DownStairs;
            map.blocked[idx] = false;
            map.ok_to_spawn[idx] = false;
        }

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
        let mut player_start_position;
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
    fn debug_highlight_blocked(&self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                if map.blocked[idx] {
                    ctx.set_bg(x, y, RGB::named(rltk::PURPLE));
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

    #[allow(dead_code)]
    fn debug_highlight_shallow_water(&self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                if map.shallow_water[idx] {
                    ctx.set_bg(x, y, RGB::named(rltk::PURPLE));
                }
            }
        }
    }

    #[allow(dead_code)]
    fn debug_highlight_deep_water(&self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                if map.deep_water[idx] {
                    ctx.set_bg(x, y, RGB::named(rltk::PURPLE));
                }
            }
        }
    }

    #[allow(dead_code)]
    fn debug_highlight_grass(&self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                if map.grass[idx] {
                    ctx.set_bg(x, y, RGB::named(rltk::GREEN));
                }
            }
        }
    }

    #[allow(dead_code)]
    fn debug_highlight_blood(&self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        for x in 0..map.width {
            for y in 0..map.height {
                let idx = map.xy_idx(x, y);
                if map.blood[idx] {
                    ctx.set_bg(x, y, RGB::named(rltk::RED));
                }
            }
        }
    }

}

impl GameState for State {

    //------------------------------------------------------------------------
    // Implements the main state machine for running the game tick-by-tick. See
    // the commentary on the RunState enum for details on each state, and a map
    // of state transitions.
    //------------------------------------------------------------------------
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        // Update the current game state at the start of each tick.
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        // Guards against rendering the game map if we are in the main menu.
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

        // The state machine.
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
                self.run_synchronization_systems();
                newrunstate = RunState::HazardTurn;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayingAnimation => {
                if is_any_animation_alive(&self.ecs) {
                    newrunstate = RunState::PlayingAnimation;
                } else {
                    self.run_cleanup_systems();
                    self.run_synchronization_systems();
                    let next_state = self.next_state
                        .expect("Returning from animation, but no next_state to return to.");
                    newrunstate = next_state;
                }
            }
            RunState::PlayerTurn => {
                self.run_player_turn_systems();
                if is_any_animation_alive(&self.ecs) {
                    self.next_state = Some(RunState::MonsterTurn);
                    newrunstate = RunState::PlayingAnimation;
                } else {
                    self.run_cleanup_systems();
                    self.run_synchronization_systems();
                    newrunstate = RunState::MonsterTurn;
                }
            }
            RunState::HazardTurn => {
                self.run_hazard_turn_systems();
                let get_blessing = is_player_encroaching_blessing_tile(&self.ecs)
                    && does_player_have_sufficient_orbs_for_blessing(&self.ecs);
                let nextstate = if get_blessing {
                    create_offered_blessings(&mut self.ecs);
                    RunState::ShowBlessingSelectionMenu
                } else {
                    RunState::AwaitingInput
                };
                if is_any_animation_alive(&self.ecs) {
                    self.next_state = Some(nextstate);
                    newrunstate = RunState::PlayingAnimation;
                } else {
                    self.run_cleanup_systems();
                    self.run_synchronization_systems();
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
                    self.run_synchronization_systems();
                    newrunstate = RunState::UpkeepTrun
                }
            }
            RunState::UpkeepTrun => {
                self.run_upkeep_turn_systems();
                self.run_synchronization_systems();
                newrunstate = RunState::HazardTurn;
            }
            RunState::ShowBlessingSelectionMenu => {
                let result = gui::show_blessings(&mut self.ecs, ctx);
                match result {
                    // The player declines to choose a blessing. We do not use
                    // their next turn, and allow a follow up action.
                    MenuResult::Cancel => {
                        clean_up_offered_blessings(&mut self.ecs);
                        // Choosing a blessing uses up the player's turn.
                        newrunstate = RunState::AwaitingInput
                    },
                    // Waiting...
                    MenuResult::NoResponse => {},
                    // The player chooses a new blessing. This uses up the
                    // player's turn, so we just *over* the AwaitingInput state,
                    // and hop to PlayerTurn to allow status systems and such to
                    // tick for the player.
                    MenuResult::Selected {thing} => {
                        receive_blessing(&mut self.ecs, thing);
                        cash_in_orbs_for_blessing(&mut self.ecs);
                        {
                            let mut offereds = self.ecs.write_storage::<OfferedBlessing>();
                            offereds.remove(thing);
                        }
                        clean_up_offered_blessings(&mut self.ecs);
                        newrunstate = RunState::PlayerTurn;
                    }
                }
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
                        let targeted = self.ecs.read_storage::<TargetedWhenThrown>();
                        let (range, kind) = targeted.get(thing)
                            .map(|x| (x.range, x.kind))
                            .expect("Attempted to throw item without Targeted components.");
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range: range,
                            thing: thing,
                            kind: kind,
                            verb: TargetingVerb::Throw,
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
                        let targeted = self.ecs.read_storage::<TargetedWhenCast>();
                        let is_targeted = targeted.get(thing).is_some();
                        if is_targeted {
                            let (range, kind) = targeted.get(thing)
                                .map(|x| (x.range, x.kind))
                                .expect("Attempted to throw item without Targeted components.");
                            newrunstate = RunState::ShowTargetingKeyboard {
                                range: range,
                                thing: thing,
                                kind: kind,
                                verb: TargetingVerb::Cast,
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
            RunState::ShowTargetingMouse {range, kind, verb, thing} => {
                match gui::ranged_target_mouse(&mut self.ecs, ctx, range, kind) {
                    TargetingResult::Cancel => newrunstate = RunState::AwaitingInput,
                    TargetingResult::SwitchModality => {
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range, thing, kind, verb, current: None
                        }
                    }
                    TargetingResult::Selected {pos} => {
                        let mut intent = self.ecs.write_storage::<WantsToUseTargeted>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(), // Player.
                            WantsToUseTargeted {thing: thing, target: pos, verb: verb}
                        ).expect("Unable to insert intent to throw item.");
                        newrunstate = RunState::PlayerTurn;
                    }
                    _ => {}
                }
            }
            RunState::ShowTargetingKeyboard {range, kind, verb, thing, current} => {
                match gui::ranged_target_keyboard(&mut self.ecs, ctx, range, kind, current) {
                    TargetingResult::Cancel => newrunstate = RunState::AwaitingInput,
                    TargetingResult::SwitchModality => {
                        newrunstate = RunState::ShowTargetingMouse {
                            range, thing, kind, verb
                        }
                    },
                    TargetingResult::MoveCursor {pos} => {
                        newrunstate = RunState::ShowTargetingKeyboard {
                            range, thing, kind, verb, current: Some(pos)
                        }
                    },
                    TargetingResult::Selected {pos} => {
                        let mut intent = self.ecs.write_storage::<WantsToUseTargeted>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(), // Player.
                            WantsToUseTargeted {thing: thing, target: pos, verb: verb}
                        ).expect("Unable to insert intent to use targeted item.");
                        newrunstate = RunState::PlayerTurn;
                    },
                    TargetingResult::NoResponse => {},
                }
            }
            RunState::ShowHelpMenu{details} => {
                let result = gui::show_help(&mut self.ecs, ctx, details);
                match result {
                    HelpMenuResult::Cancel => match details {
                        None => { newrunstate = RunState::AwaitingInput }
                        Some(_) => { newrunstate = RunState::ShowHelpMenu{details: None} }
                    }
                    HelpMenuResult::NoSelection {current: _} => {},
                    HelpMenuResult::Selected {selected: s} => {
                        let command = COMMANDS[s];
                        if !command.key_codes().is_empty() {
                            newrunstate = RunState::ShowHelpMenu{details: Some(command.details())}
                        }
                    }
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
        .with_font("modified-terminal8x8.png".to_string(), 8, 8)
        .with_simple_console(80, 50, "modified-terminal8x8.png".to_string())
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
    gs.ecs.register::<UseFgColorMap>();
    gs.ecs.register::<UseFgColorMapWhenBloodied>();
    gs.ecs.register::<UseBgColorMap>();
    gs.ecs.register::<UseBgColorMapWhenBloodied>();
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
    gs.ecs.register::<Tramples>();
    gs.ecs.register::<Bloodied>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Castable>();
    gs.ecs.register::<TargetedWhenThrown>();
    gs.ecs.register::<TargetedWhenCast>();
    gs.ecs.register::<Untargeted>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InSpellBook>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<BlessingOrbBag>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Hazard>();
    gs.ecs.register::<BlessingOrb>();
    gs.ecs.register::<BlessingSelectionTile>();
    gs.ecs.register::<OfferedBlessing>();
    gs.ecs.register::<IsEntityKind>();
    gs.ecs.register::<CanAct>();
    gs.ecs.register::<CanNotAct>();
    gs.ecs.register::<MovementRoutingAvoids>();
    gs.ecs.register::<MovementRoutingBounds>();
    gs.ecs.register::<MonsterBasicAI>();
    gs.ecs.register::<MonsterAttackSpellcasterAI>();
    gs.ecs.register::<MonsterSupportSpellcasterAI>();
    gs.ecs.register::<SpellCharges>();
    gs.ecs.register::<RemovedFromSpellBookWhenCast>();
    gs.ecs.register::<IncresesSpellRechargeRateWhenEquipped>();
    gs.ecs.register::<WeaponSpecial>();
    gs.ecs.register::<ExpendWeaponSpecialWhenThrown>();
    gs.ecs.register::<ExpendWeaponSpecialWhenCast>();
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
    gs.ecs.register::<DissipateWhenTrampledUpon>();
    gs.ecs.register::<ProvidesFullHealing>();
    gs.ecs.register::<IncreasesMaxHpWhenUsed>();
    gs.ecs.register::<ProvidesFullFood>();
    gs.ecs.register::<ProvidesFullSpellRecharge>();
    gs.ecs.register::<DecreasesSpellRechargeWhenUsed>();
    gs.ecs.register::<ProvidesFireImmunityWhenUsed>();
    gs.ecs.register::<ProvidesChillImmunityWhenUsed>();
    gs.ecs.register::<MovesToRandomPosition>();
    gs.ecs.register::<InflictsDamageWhenThrown>();
    gs.ecs.register::<InflictsDamageWhenCast>();
    gs.ecs.register::<InflictsDamageWhenEncroachedUpon>();
    gs.ecs.register::<RemoveBurningWhenEncroachedUpon>();
    gs.ecs.register::<DissipateFireWhenEncroachedUpon>();
    gs.ecs.register::<RemoveBurningOnUpkeep>();
    gs.ecs.register::<InflictsFreezingWhenThrown>();
    gs.ecs.register::<InflictsFreezingWhenCast>();
    gs.ecs.register::<InflictsBurningWhenThrown>();
    gs.ecs.register::<InflictsBurningWhenCast>();
    gs.ecs.register::<MoveToPositionWhenCast>();
    gs.ecs.register::<BuffsMeleeAttackWhenCast>();
    gs.ecs.register::<BuffsPhysicalDefenseWhenCast>();
    gs.ecs.register::<InflictsBurningWhenEncroachedUpon>();
    gs.ecs.register::<InflictsFreezingWhenEncroachedUpon>();
    gs.ecs.register::<SpawnsEntityInAreaWhenThrown>();
    gs.ecs.register::<SpawnsEntityInAreaWhenCast>();
    gs.ecs.register::<SpawnEntityWhenEncroachedUpon>();
    gs.ecs.register::<SpawnEntityWhenTrampledUpon>();
    gs.ecs.register::<SpawnEntityWhenMeleeAttacked>();
    gs.ecs.register::<SpawnEntityWhenKilled>();
    gs.ecs.register::<ChanceToSpawnAdjacentEntity>();
    gs.ecs.register::<ChanceToSpawnEntityWhenBurning>();
    gs.ecs.register::<ChanceToInflictBurningOnAdjacentEntities>();
    gs.ecs.register::<ChanceToDissipate>();
    gs.ecs.register::<SkipRandomDissipationForOneTurn>();
    gs.ecs.register::<MeeleAttackWepon>();
    gs.ecs.register::<GrantsMeleeDefenseBonus>();
    gs.ecs.register::<StatusIsFrozen>();
    gs.ecs.register::<StatusIsBurning>();
    gs.ecs.register::<StatusIsImmuneToFire>();
    gs.ecs.register::<StatusIsImmuneToChill>();
    gs.ecs.register::<StatusIsMeleeAttackBuffed>();
    gs.ecs.register::<StatusIsPhysicalDefenseBuffed>();
    gs.ecs.register::<StatusInvisibleToPlayer>();
    gs.ecs.register::<InvisibleWhenEncroachingEntityKind>();
    gs.ecs.register::<AnimationParticle>();
    gs.ecs.register::<AnimationWhenUsed>();
    gs.ecs.register::<AnimationWhenThrown>();
    gs.ecs.register::<AnimationWhenCast>();

    // Placeholder values which we will replace upon map generation.
    gs.ecs.insert(Point::new(0, 0)); // Player position.
    gs.ecs.insert(Map::new(1));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(GameLog::new());

    // Buffers for accumulating requests for ECS changes throughout a turn.
    // Think like it's a database, we queue operations, then commit the changes
    // in batch.
    gs.ecs.insert(AnimationSequenceBuffer::new());
    gs.ecs.insert(AnimationParticleBuffer::new());
    gs.ecs.insert(EntitySpawnRequestBuffer::new());
    gs.ecs.insert(MeeleAttackRequestBuffer::new());

    // Create the player entity. Note that we're not computing the correct
    // position to place the player here, since that needs to happen after we've
    // generated terrain and monsters.
    let player = entity_spawners::player::spawn_player(&mut gs.ecs, 0, 0);
    // The player is the only Entity object inserted intot he ECS as a resource,
    // so it is easily fetchable since fetching the player is such a common
    // operation.
    gs.ecs.insert(player);
    gs.ecs.insert(RunState::MapGeneration {});

    gs.generate_map(1);

    rltk::main_loop(context, gs)
}
