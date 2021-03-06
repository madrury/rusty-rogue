use super::{
    BlocksTile, CombatStats, Monster, MonsterMovementAI, Name, Player,
    Position, Rectangle, Renderable, Viewshed, PickUpable, Useable,
    Equippable, EquipmentSlot, Throwable, Targeted, UnTargeted, Consumable,
    ProvidesFullHealing, IncreasesMaxHpWhenUsed, AreaOfEffectWhenTargeted,
    InflictsDamageWhenTargeted, InflictsFreezingWhenTargeted,
    InflictsBurningWhenTargeted, AreaOfEffectAnimationWhenTargeted,
    MovesToRandomPosition, GrantsMeleeAttackBonus, GrantsMeleeDefenseBonus,
    SimpleMarker, SerializeMe, MarkedBuilder,
    MAP_WIDTH, random_table
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

const MAX_MONSTERS_IN_ROOM: i32 = 4;
const MAX_ITEMS_IN_ROOM: i32 = 2;

// Spawn the player in the center of the first room.
pub fn spawn_player(ecs: &mut World, px: i32, py: i32) -> Entity {
    ecs.create_entity()
        .with(Position {x: px, y: py})
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

    let monster_spawn_points;
    let item_spawn_points;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        monster_spawn_points = generate_spawn_points(num_monsters, room, &mut *rng);
        item_spawn_points = generate_spawn_points(num_items, room, &mut *rng);
    }

    for (x, y) in monster_spawn_points.iter() {
        spawn_random_monster(ecs, *x, *y, depth);
    }
    for (x, y) in item_spawn_points.iter() {
        spawn_random_item(ecs, *x, *y, depth);
    }
}

// Returns a vector of positions in a room to spawn entities.
fn generate_spawn_points(n: i32, room: &Rectangle, rng: &mut RandomNumberGenerator) -> Vec<(i32, i32)> {
    // First generate some random indexes.
    let mut spawn_idxs: Vec<usize> = Vec::new();
    for _i in 1..=n {
        let mut added = false;
        while !added {
            let (x, y) = room.random_point(rng);
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
    TeleportationPotion,
    FirePotion,
    FreezingPotion,
    Dagger,
    LeatherArmor,
}
fn spawn_random_item(ecs: &mut World, x: i32, y: i32, depth: i32) {
    let item: Option<ItemType>;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        // TODO: Make this table in a less stupid place.
        item = random_table::RandomTable::new()
            .insert(ItemType::HealthPotion, 10)
            .insert(ItemType::TeleportationPotion, 5 + depth)
            .insert(ItemType::FirePotion, 5+ depth)
            .insert(ItemType::FreezingPotion, 5 + depth)
            .insert(ItemType::Dagger, 3 + depth)
            .insert(ItemType::LeatherArmor, 3 + depth)
            .insert(ItemType::None, 60 - 2 * depth)
            .roll(&mut rng);
    }
    match item {
        Some(ItemType::HealthPotion) => health_potion(ecs, x, y),
        Some(ItemType::TeleportationPotion) => teleportation_potion(ecs, x, y),
        Some(ItemType::FirePotion) => fire_potion(ecs, x, y),
        Some(ItemType::FreezingPotion) => freezing_potion(ecs, x, y),
        Some(ItemType::Dagger) => dagger(ecs, x, y),
        Some(ItemType::LeatherArmor) => leather_armor(ecs, x, y),
        Some(ItemType::None) => {},
        None => {}
    }
}

//----------------------------------------------------------------------------
// Individual Monsters
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
    max_hp: 15,
    hp: 15,
    defense: 1,
    power: 3,
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

// Individual monster types: Goblin.
fn goblin(ecs: &mut World, x: i32, y: i32) {
    spawn_monster(
        ecs,
        MonsterSpawnData {
            x: x,
            y: y,
            name: "Goblin",
            glyph: rltk::to_cp437('g'),
            color: RGB::named(rltk::CORAL),
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

// Individual monster types: Orc.
fn orc(ecs: &mut World, x: i32, y: i32) {
    spawn_monster(
        ecs,
        MonsterSpawnData {
            x: x,
            y: y,
            name: "Orc",
            glyph: rltk::to_cp437('O'),
            color: RGB::named(rltk::YELLOW_GREEN),
            view_range: DEFAULT_VIEW_RANGE,
            movement_ai: MonsterMovementAI {
                ..DEFAULT_MOVEMENT_AI
            },
            combat_stats: CombatStats {
                max_hp: 20,
                hp: 20,
                power: 5,
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
    .with(UnTargeted {verb: "use".to_string()})
    .with(Throwable {})
    .with(Targeted {verb: "throw".to_string()})
    .with(Consumable {})
    // TODO: We probably want a component for triggering the healing animation.
    .with(ProvidesFullHealing {})
    .with(IncreasesMaxHpWhenUsed {amount: 5})
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
    .with(Targeted {verb: "throw".to_string()})
    .with(Consumable {})
    .with(InflictsDamageWhenTargeted {damage: 10})
    .with(InflictsBurningWhenTargeted {turns: 4, tick_damage: 4})
    .with(AreaOfEffectWhenTargeted {radius: 2})
    .with(AreaOfEffectAnimationWhenTargeted {
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
    .with(Targeted {verb: "throw".to_string()})
    .with(Consumable {})
    .with(InflictsDamageWhenTargeted {damage: 10})
    .with(InflictsFreezingWhenTargeted {turns: 6})
    .with(AreaOfEffectWhenTargeted {radius: 2})
    .with(AreaOfEffectAnimationWhenTargeted {
        radius: 2,
        fg: RGB::named(rltk::WHITE),
        bg: RGB::named(rltk::LIGHT_BLUE),
        glyph: rltk::to_cp437('*')
    })
    .marked::<SimpleMarker<SerializeMe>>()
    .build();
}

fn teleportation_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437('¿'),
        fg: RGB::named(rltk::MEDIUM_PURPLE),
        bg: RGB::named(rltk::BLACK),
        order: 2,
    })
    .with(Name {name: "Potion of Teleportation".to_string()})
    .with(PickUpable {})
    .with(Useable {})
    .with(UnTargeted{ verb: "use".to_string()})
    .with(Throwable {})
    .with(Targeted {verb: "throw".to_string()})
    .with(Consumable {})
    .with(MovesToRandomPosition {})
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
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            order: 2
        })
        .with(Name {name : "Dagger".to_string()})
        .with(PickUpable {})
        .with(Equippable {slot: EquipmentSlot::Melee})
        .with(GrantsMeleeAttackBonus {bonus: 2})
        .with(Throwable {})
        .with(Targeted {verb: "throw".to_string()})
        .with(Consumable {})
        .with(InflictsDamageWhenTargeted {damage: 15})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn leather_armor(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(']'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            order: 2
        })
        .with(Name {name : "Leather Armor".to_string()})
        .with(PickUpable {})
        .with(Equippable {slot: EquipmentSlot::Armor})
        .with(GrantsMeleeDefenseBonus {bonus: 1})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}