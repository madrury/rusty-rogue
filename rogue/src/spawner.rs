use super::{
    BlocksTile, CombatStats, Monster, MonsterMovementAI, Name, Player,
    Position, Rectangle, Renderable, Viewshed, PickUpable, Useable,
    Equippable, Throwable, Consumable, ProvidesHealing,
    AreaOfEffectWhenThrown, InflictsDamageWhenThrown,
    InflictsFreezingWhenThrown, InflictsBurningWhenThrown,
    AreaOfEffectAnimationWhenThrown, SimpleMarker, SerializeMe,
    MarkedBuilder,
    MAP_WIDTH, random_table
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
            order: 0
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

// Populate a room with monsters and items.
pub fn spawn_room(ecs: &mut World, room: &Rectangle, depth: i32) {
    let num_monsters: i32;
    let num_items: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        num_monsters = rng.roll_dice(1, MAX_MONSTERS_IN_ROOM + 1) + depth;
        num_items = rng.roll_dice(1, MAX_ITEMS_IN_ROOM + 1) + depth / 2;
    }

    let monster_spawn_points = generate_spawn_points(ecs, num_monsters, room);
    let item_spawn_points = generate_spawn_points(ecs, num_items, room);

    for (x, y) in monster_spawn_points.iter() {
        spawn_random_monster(ecs, *x, *y, depth);
    }
    for (x, y) in item_spawn_points.iter() {
        spawn_random_item(ecs, *x, *y, depth);
    }
}

// Returns a vector of positions in a room to spawn entities.
fn generate_spawn_points(ecs: &mut World, n: i32, room: &Rectangle) -> Vec<(i32, i32)> {
    // First generate some random indexes.
    let mut spawn_idxs: Vec<usize> = Vec::new();
    for _i in 1..=n {
        let mut added = false;
        while !added {
            let (x, y) = room.random_point(ecs);
            let idx = ((y * MAP_WIDTH) as usize) + x as usize;
            if !spawn_idxs.contains(&idx) {
                spawn_idxs.push(idx);
                added = true;
            }
        }
    }
    // Then convert these to points.
    let mut spawn_points: Vec<(i32, i32)> = Vec::new();
    for idx in spawn_idxs.iter() {
        let (x, y) = (*idx as i32 % MAP_WIDTH, *idx as i32 / MAP_WIDTH);
        spawn_points.push((x, y))
    }
    spawn_points
}

// Spawns a random monster at a specified location.
#[derive(Clone, Copy)]
enum MonsterType {
    None,
    Goblin,
    Orc
}
fn spawn_random_monster(ecs: &mut World, x: i32, y: i32, depth: i32) {
    let monster: Option<MonsterType>;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        // TODO: Make this table in a less stupid place.
        monster = random_table::RandomTable::new()
            .insert(MonsterType::Goblin, 25)
            .insert(MonsterType::Orc, 5 + 5 * depth)
            .insert(MonsterType::None, 70 - depth)
            .roll(&mut rng);
    }
    match monster {
        Some(MonsterType::Orc) => orc(ecs, x, y),
        Some(MonsterType::Goblin) => goblin(ecs, x, y),
        _ => {}
    }
}

// Spawns a random item at a specified location.
#[derive(Clone, Copy)]
enum ItemType {
    None,
    HealthPotion,
    FirePotion,
    FreezingPotion,
    Dagger,
}
fn spawn_random_item(ecs: &mut World, x: i32, y: i32, depth: i32) {
    let item: Option<ItemType>;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        // TODO: Make this table in a less stupid place.
        item = random_table::RandomTable::new()
            .insert(ItemType::HealthPotion, 20)
            .insert(ItemType::FirePotion, 5 + depth)
            .insert(ItemType::FreezingPotion, 5 + depth)
            .insert(ItemType::Dagger, 1000)
            .insert(ItemType::None, 60 - 2 * depth)
            .roll(&mut rng);
    }
    match item {
        Some(ItemType::HealthPotion) => health_potion(ecs, x, y),
        Some(ItemType::FirePotion) => fire_potion(ecs, x, y),
        Some(ItemType::FreezingPotion) => freezing_potion(ecs, x, y),
        Some(ItemType::Dagger) => dagger(ecs, x, y),
        Some(ItemType::None) => {},
        None => {}
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
            order: 1
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

//----------------------------------------------------------------------------
// Individual Monsters
//----------------------------------------------------------------------------
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

//----------------------------------------------------------------------------
// Individual Items
//----------------------------------------------------------------------------
fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437('¿'),
        fg: RGB::named(rltk::RED),
        bg: RGB::named(rltk::BLACK),
        order: 2,
    })
    .with(Name {name: "Potion of Healing".to_string()})
    .with(PickUpable {})
    .with(Useable {})
    .with(Throwable {})
    .with(Consumable {})
    // TODO: We probably want a component for triggering the healing animation.
    .with(ProvidesHealing {})
    .marked::<SimpleMarker<SerializeMe>>()
    .build();
}

fn fire_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437('¿'),
        fg: RGB::named(rltk::ORANGE),
        bg: RGB::named(rltk::BLACK),
        order: 2,
    })
    .with(Name {name: "Potion of Fire".to_string()})
    .with(PickUpable {})
    .with(Throwable {})
    .with(Consumable {})
    .with(InflictsDamageWhenThrown {damage: 10})
    .with(InflictsBurningWhenThrown {turns: 4, tick_damage: 4})
    .with(AreaOfEffectWhenThrown {radius: 2})
    .with(AreaOfEffectAnimationWhenThrown {
        radius: 2,
        fg: RGB::named(rltk::ORANGE),
        bg: RGB::named(rltk::RED),
        glyph: rltk::to_cp437('^')
    })
    .marked::<SimpleMarker<SerializeMe>>()
    .build();
}

fn freezing_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437('¿'),
        fg: RGB::named(rltk::LIGHT_BLUE),
        bg: RGB::named(rltk::BLACK),
        order: 2,
    })
    .with(Name {name: "Potion of Freezing".to_string()})
    .with(PickUpable {})
    .with(Throwable {})
    .with(Consumable {})
    .with(InflictsDamageWhenThrown {damage: 10})
    .with(InflictsFreezingWhenThrown {turns: 6})
    .with(AreaOfEffectWhenThrown {radius: 2})
    .with(AreaOfEffectAnimationWhenThrown {
        radius: 2,
        fg: RGB::named(rltk::WHITE),
        bg: RGB::named(rltk::LIGHT_BLUE),
        glyph: rltk::to_cp437('*')
    })
    .marked::<SimpleMarker<SerializeMe>>()
    .build();
}

//----------------------------------------------------------------------------
// Individual Equipment
//----------------------------------------------------------------------------
fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('↑'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            order: 2
        })
        .with(Name {name : "Dagger".to_string()})
        .with(PickUpable {})
        .with(Equippable {})
        .with(Throwable {})
        .with(Consumable {})
        .with(InflictsDamageWhenThrown {damage: 20})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}