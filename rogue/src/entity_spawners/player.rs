use crate::Tramples;

use super::{
    Point, Map, CombatStats, SwimStamina, HungerClock, HungerState, Name, Player,
    Monster, Position, Renderable, Viewshed, BlessingOrbBag, SimpleMarker, SerializeMe, MarkedBuilder,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

const PLAYER_VIEW_RANGE: i32 = 12;
const PLAYER_MAX_HP: i32 = 50;
const PLAYER_DEFENSE: i32 = 2;
const PLAYER_ATTACK_POWER: i32 = 5;
const PLAYER_HUNGER_STATE_DURATION: i32 = 300;
const PLAYER_HUNGER_TICK_DAMAGE: i32 = 1;
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
        .with(BlessingOrbBag {count: 0})
        .with(Tramples {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

//----------------------------------------------------------------------------
// Find a postition to spawn the player.
//
// It is assumed this function is called after the map has been generated,
// terrain has been spawned, and monsters have been spawned. We search for a
// position that is far away from any monsters.
//
// The strategy here is crude, but seems to work alright. We attempt to spawn at
// a random point and check how far away we are from the closest monster. If
// we're too close, we try again for some number of times, and if we fail
// repeatedly, we lower the acceptable threshold and try again until. We
// continue like this until we succeed.
//----------------------------------------------------------------------------
const INITIAL_DISTANCE_BOUND: i32 = 15;
const N_TIMES_TO_TRY_EACH_DISTANCE: i32 = 5;

pub fn get_spawn_position(ecs: &World) -> Option<Point> {
    let mut rng = RandomNumberGenerator::new();
    let map = ecs.fetch::<Map>();
    let mut start: Option<Point> = None;
    let mut n_tries: i32 = 0;
    while start.is_none() {
        let maybe = map.random_unblocked_point(250, &mut rng);
        start = match maybe {
            None => {
                n_tries += 1;
                None
            }
            Some(maybe) => {
                let idx = map.xy_idx(maybe.x, maybe.y);
                let acceptable_distance =
                    INITIAL_DISTANCE_BOUND -  n_tries / N_TIMES_TO_TRY_EACH_DISTANCE;
                let distance_is_acceptable =
                    distance_to_closest_monster(ecs, maybe) >= acceptable_distance as f32;
                if distance_is_acceptable && map.ok_to_spawn[idx] && !map.deep_water[idx] {
                    Some(maybe)
                } else {
                    n_tries += 1;
                    None
                }
            }
        }
    }
    start
}

fn distance_to_closest_monster(ecs: &World, maybe: Point) -> f32 {
    let monsters = ecs.read_storage::<Monster>();
    let positions = ecs.read_storage::<Position>();
    (&monsters, &positions).join()
        .map(|(_m, pos)| rltk::DistanceAlg::Pythagoras.distance2d(pos.to_point(), maybe))
        .fold(f32::INFINITY, |a, b| a.min(b))
}
