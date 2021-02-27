use super::{
    BlocksTile, CombatStats, Monster, MonsterMovementAI, Name, Player, Position, Rectangle,
    Renderable, Viewshed, MAP_WIDTH,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

const MAX_MONSTERS_IN_ROOM: i32 = 4;
const MAX_ITEMS_IN_ROOM: i32 = 2;

// Spawn the player in the center of the first room.
pub fn spawn_player(ecs: &mut World, px: i32, py: i32) -> Entity {
    ecs.create_entity()
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
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build()
}

// Populate a room with monsters and items.
pub fn spawn_room(ecs: &mut World, room: &Rectangle) {
    let num_monsters: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        num_monsters = rng.roll_dice(1, MAX_MONSTERS_IN_ROOM + 1) - 1;
    }

    let mut monster_spawn_idxs: Vec<usize> = Vec::new();
    for _i in 1..=num_monsters {
        let mut added = false;
        while !added {
            let (x, y) = room.random_point(ecs);
            let idx = ((y * MAP_WIDTH) as usize) + x as usize;
            if !monster_spawn_idxs.contains(&idx) {
                monster_spawn_idxs.push(idx);
                added = true;
            }
        }
    }

    let mut monster_spawn_points: Vec<(i32, i32)> = Vec::new();
    for idx in monster_spawn_idxs.iter() {
        let (x, y) = (*idx as i32 % MAP_WIDTH, *idx as i32 / MAP_WIDTH);
        monster_spawn_points.push((x, y))
    }

    for (x, y) in monster_spawn_points.iter() {
        spawn_random_monster(ecs, *x, *y);
    }
}

// Spawns a random monster at a specified location.
pub fn spawn_random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => orc(ecs, x, y),
        _ => goblin(ecs, x, y),
    }
}

//----------------------------------------------------------------------------
// Individual Monster Spawning Logic
//----------------------------------------------------------------------------
struct MonsterSpawnData<S: ToString> {
    x: i32,
    y: i32,
    name: S,
    glyph: rltk::FontCharType,
    color: RGB,
    view_range: i32,
    movement_ai: MonsterMovementAI,
    combat_stats: CombatStats,
}

const DEFAULT_VIEW_RANGE: i32 = 8;
const DEFAULT_MOVEMENT_AI: MonsterMovementAI = MonsterMovementAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    lost_visibility_keep_following_turns_max: 2,
    lost_visibility_keep_following_turns_remaining: 2,
};
const DEFAULT_COMBAT_STATS: CombatStats = CombatStats {
    max_hp: 16,
    hp: 16,
    defense: 1,
    power: 4,
};

// Spawn a generic monster.
fn spawn_monster<S: ToString>(ecs: &mut World, data: MonsterSpawnData<S>) {
    ecs.create_entity()
        .with(Position {
            x: data.x,
            y: data.y,
        })
        .with(Monster {})
        .with(Renderable {
            glyph: data.glyph,
            fg: data.color,
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: data.view_range,
            dirty: true,
        })
        .with(Name {
            name: data.name.to_string(),
        })
        .with(data.movement_ai)
        .with(data.combat_stats)
        .with(BlocksTile {})
        .build();
}

// Individual monster types: Orc.
fn orc(ecs: &mut World, x: i32, y: i32) {
    spawn_monster(
        ecs,
        MonsterSpawnData {
            x: x,
            y: y,
            name: "Orc",
            glyph: rltk::to_cp437('O'),
            color: RGB::named(rltk::RED),
            view_range: DEFAULT_VIEW_RANGE,
            movement_ai: MonsterMovementAI {
                ..DEFAULT_MOVEMENT_AI
            },
            combat_stats: CombatStats {
                ..DEFAULT_COMBAT_STATS
            },
        },
    )
}

// Individual monster types: Goblin.
fn goblin(ecs: &mut World, x: i32, y: i32) {
    spawn_monster(
        ecs,
        MonsterSpawnData {
            x: x,
            y: y,
            name: "Goblin",
            glyph: rltk::to_cp437('g'),
            color: RGB::named(rltk::RED),
            view_range: DEFAULT_VIEW_RANGE,
            movement_ai: MonsterMovementAI {
                ..DEFAULT_MOVEMENT_AI
            },
            combat_stats: CombatStats {
                ..DEFAULT_COMBAT_STATS
            },
        },
    )
}
