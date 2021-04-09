use super::{
    Point, Map, CombatStats, SwimStamina, HungerClock, HungerState, Name, Player,
    Monster, Position, Renderable, Viewshed, SimpleMarker, SerializeMe, MarkedBuilder,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

const PLAYER_VIEW_RANGE: i32 = 12;
const PLAYER_MAX_HP: i32 = 50;
const PLAYER_DEFENSE: i32 = 2;
const PLAYER_ATTACK_POWER: i32 = 5;
const PLAYER_HUNGER_STATE_DURATION: i32 = 300;
const PLAYER_HUNGER_TICK_DAMAGE: i32 = 300;
const PLAYER_SWIM_STAMINA: i32 = 3;
const PLAYER_DROWNING_TICK_DAMAGE: i32 = 5;


// Create the player entity in a specified position. Called only once a game.
pub fn spawn_player(ecs: &mut World, px: i32, py: i32) -> Entity {
    ecs.create_entity()
        .with(Position {x: px, y: py})
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            order: 0,
            visible_out_of_fov: true
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: PLAYER_VIEW_RANGE,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: PLAYER_MAX_HP,
            hp: PLAYER_MAX_HP,
            defense: PLAYER_DEFENSE,
            power: PLAYER_ATTACK_POWER,
        })
        .with(HungerClock {
            state: HungerState::WellFed,
            state_duration: PLAYER_HUNGER_STATE_DURATION,
            time: PLAYER_HUNGER_STATE_DURATION,
            tick_damage: PLAYER_HUNGER_TICK_DAMAGE
        })
        .with(SwimStamina {
            max_stamina: PLAYER_SWIM_STAMINA,
            stamina: PLAYER_SWIM_STAMINA,
            drowning_damage: PLAYER_DROWNING_TICK_DAMAGE
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

// Find a postition to spawn the player.
//
// It is assumed this function is called after the map has been generated,
// terrain has been spawned, and monsters have been spawned. We search for a
// position that is far away from any monsters.

const INITIAL_DISTANCE_BOUND: i32 = 15;
const N_TIMES_TO_TRY_EACH_DISTANCE: i32 = 5;

pub fn get_spawn_position(ecs: &World) -> Option<Point> {
    let mut rng = RandomNumberGenerator::new();
    let map = ecs.fetch::<Map>();
    let mut start: Option<Point> = None;
    let n_tries: i32 = 0;
    while start.is_none() {
        let maybe = map.random_unblocked_point(250, &mut rng);
        start = match maybe {
            None => {None}
            Some(maybe) => {
                let acceptable_distance =
                    INITIAL_DISTANCE_BOUND -  n_tries / N_TIMES_TO_TRY_EACH_DISTANCE;
                let distance_is_acceptable =
                    distance_to_closest_monster(ecs, maybe) >= acceptable_distance as f32;
                if distance_is_acceptable {
                    Some(Point {x: maybe.0, y: maybe.1})
                } else {
                    None
                }
            }
        }
    }
    start
}

fn distance_to_closest_monster(ecs: &World, maybe: (i32, i32)) -> f32 {
    let maybe_point = Point {x: maybe.0, y: maybe.1};
    let monsters = ecs.read_storage::<Monster>();
    let positions = ecs.read_storage::<Position>();
    (&monsters, &positions).join()
        .map(|(_m, pos)| rltk::DistanceAlg::Pythagoras.distance2d(pos.to_point(), maybe_point))
        .fold(f32::INFINITY, |a, b| a.min(b))
}
