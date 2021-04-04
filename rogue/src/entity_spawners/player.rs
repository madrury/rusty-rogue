use super::{
    Point, Map, CombatStats, SwimStamina, HungerClock, HungerState, Name, Player,
    Position, Renderable, Viewshed, SimpleMarker, SerializeMe, MarkedBuilder,
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

pub fn get_spawn_position(ecs: &World) -> Option<Point> {
    let mut rng = RandomNumberGenerator::new();
    let map = ecs.fetch::<Map>();
    let mut start: Option<Point> = None;
    while start.is_none() {
        let maybe = map.random_unblocked_point(250, &mut rng);
        start = match maybe {
            None => {None}
            Some(maybe) => {
                let idx = map.xy_idx(maybe.0, maybe.1);
                let ok_to_spawn = map.ok_to_spawn[idx];
                if ok_to_spawn {
                    Some(Point {x: maybe.0, y: maybe.1})
                } else {
                    None
                }
            }
        }
    }
    start
}
