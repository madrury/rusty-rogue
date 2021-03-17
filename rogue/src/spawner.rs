use super::{
    Map, TileType, EntitySpawnKind, BlocksTile, CombatStats, HungerClock,
    HungerState, Monster, IsEntityKind, MonsterMovementAI,
    MovementRoutingOptions, Name, Player, Position, Renderable, Viewshed,
    PickUpable, Useable, Castable, SpellCharges, Equippable, EquipmentSlot,
    Throwable, Targeted, Untargeted, Consumable, ProvidesFullHealing,
    ProvidesFullFood, IncreasesMaxHpWhenUsed, AreaOfEffectWhenTargeted,
    InflictsDamageWhenTargeted, InflictsDamageWhenEncroachedUpon,
    InflictsFreezingWhenTargeted, InflictsBurningWhenTargeted,
    InflictsBurningWhenEncroachedUpon, AreaOfEffectAnimationWhenTargeted,
    MovesToRandomPosition, SpawnsEntityInAreaWhenTargeted,
    ChanceToSpawnAdjacentEntity, ChanceToDissipate, GrantsMeleeAttackBonus,
    GrantsMeleeDefenseBonus, ProvidesFireImmunityWhenUsed, SimpleMarker,
    SerializeMe, MarkedBuilder, ElementalDamageKind,
    MAP_WIDTH, random_table
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

const MAX_MONSTERS_IN_ROOM: i32 = 4;
const MAX_ITEMS_IN_ROOM: i32 = 2;

// Create the player entity in a specified position. Called only once a game.
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
            max_hp: 50,
            hp: 50,
            defense: 2,
            power: 5,
        })
        .with(HungerClock {
            state: HungerState::Normal,
            state_duration: 200,
            time: 200,
            tick_damage: 1
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

// Populate a reigon (defined by a container of map indexes) with monsters and
// items.
pub fn spawn_region(ecs: &mut World, region: &[usize], depth: i32) {
    let mut areas : Vec<usize> = Vec::from(region);
    if areas.is_empty() {return;}

    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS_IN_ROOM + 1) + depth;
        let num_items = rng.roll_dice(1, MAX_ITEMS_IN_ROOM + 1) + depth / 2;
        for i in 0..(num_monsters + num_items) {
            if areas.is_empty() {break;}
            let array_index = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };
            let map_idx = areas[array_index];
            if i < num_monsters {
                monster_spawn_points.push(map_idx);
            } else {
                item_spawn_points.push(map_idx)
            }
            areas.remove(array_index);
        }
    }

    for idx in monster_spawn_points.iter() {
        let (x, y) = (*idx as i32 % MAP_WIDTH, *idx as i32 / MAP_WIDTH);
        spawn_random_monster(ecs, x, y, depth);
    }
    for idx in item_spawn_points.iter() {
        let (x, y) = (*idx as i32 % MAP_WIDTH, *idx as i32 / MAP_WIDTH);
        spawn_random_item(ecs, x, y, depth);
    }
}

// Spawns a randomly chosen monster at a specified location.
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

// Spawns a randomly chosen item at a specified location.
#[derive(Clone, Copy)]
enum ItemType {
    None,
    Turnip,
    Pomegranate,
    HealthPotion,
    TeleportationPotion,
    FirePotion,
    FreezingPotion,
    Dagger,
    LeatherArmor,
    FireblastScroll,
    IceblastScroll,
}
fn spawn_random_item(ecs: &mut World, x: i32, y: i32, depth: i32) {
    let item: Option<ItemType>;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        // TODO: Make this table in a less stupid place.
        item = random_table::RandomTable::new()
            .insert(ItemType::Turnip, 3 + depth)
            .insert(ItemType::Pomegranate, 1 + depth)
            .insert(ItemType::HealthPotion, 3 + depth)
            .insert(ItemType::TeleportationPotion, 2 + depth)
            .insert(ItemType::FirePotion, 2 + depth)
            .insert(ItemType::FreezingPotion, 2 + depth)
            .insert(ItemType::Dagger, depth)
            .insert(ItemType::LeatherArmor, depth)
            .insert(ItemType::FireblastScroll, depth)
            .insert(ItemType::IceblastScroll, depth)
            .insert(ItemType::None, 100)
            .roll(&mut rng);
    }
    match item {
        Some(ItemType::Turnip) => turnip(ecs, x, y),
        Some(ItemType::Pomegranate) => pomegranate(ecs, x, y),
        Some(ItemType::HealthPotion) => health_potion(ecs, x, y),
        Some(ItemType::TeleportationPotion) => teleportation_potion(ecs, x, y),
        Some(ItemType::FirePotion) => fire_potion(ecs, x, y),
        Some(ItemType::FreezingPotion) => freezing_potion(ecs, x, y),
        Some(ItemType::Dagger) => dagger(ecs, x, y),
        Some(ItemType::LeatherArmor) => leather_armor(ecs, x, y),
        Some(ItemType::FireblastScroll) => fireblast(ecs, x, y),
        Some(ItemType::IceblastScroll) => iceblast(ecs, x, y),
        Some(ItemType::None) => {},
        None => {}
    }
}

//----------------------------------------------------------------------------
// Monsters
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
    routing_options: MovementRoutingOptions {
        avoid_blocked: true,
        avoid_fire: true
    }
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
    );
    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
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
    );
    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
}

//----------------------------------------------------------------------------
// Hazards
//----------------------------------------------------------------------------
// Spawn a fire entity in the ecs. All spawning of fire MUST use this function,
// since it handles syncronizing the map.fire array.
pub fn fire(ecs: &mut World, x: i32, y: i32, spread_chance: i32, dissipate_chance: i32) {
    let can_spawn: bool;
    let idx: usize;
    {
        let map = ecs.fetch::<Map>();
        idx = map.xy_idx(x, y);
        can_spawn = !map.fire[idx] && map.tiles[idx] != TileType::Wall;
    }
    if can_spawn {
        ecs.create_entity()
            .with(Position {x, y})
            .with(Renderable {
                glyph: rltk::to_cp437('^'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::ORANGE),
                order: 2,
            })
            .with(Name {name: "Fire".to_string()})
            .with(IsEntityKind {
                kind: EntitySpawnKind::Fire {
                    spread_chance, dissipate_chance
                }
            })
            .with(ChanceToSpawnAdjacentEntity {
                chance: spread_chance,
                kind: EntitySpawnKind::Fire {
                    spread_chance: i32::max(0, spread_chance - 20),
                    dissipate_chance: i32::max(0, dissipate_chance + 20),
                }
            })
            .with(ChanceToDissipate {
                chance: dissipate_chance
            })
            .with(InflictsDamageWhenEncroachedUpon {
                damage: 10,
                kind: ElementalDamageKind::Fire
            })
            .with(InflictsBurningWhenEncroachedUpon {
                turns: 4, tick_damage: 2
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        map.fire[idx] = true;
    }
}

// Remove a fire entity from the ecs. All despawning of fire MUST use this
// function, since we need to syncronize the map.fire array.
pub fn destroy_fire(ecs: &mut World, entity: &Entity) {
    let idx;
    { // Contain first borrow of ECS.
        let positions = ecs.read_storage::<Position>();
        let map = ecs.fetch::<Map>();
        let pos = positions.get(*entity);
        match pos {
            Some(pos) => {
                idx = map.xy_idx(pos.x, pos.y);
                if !map.fire[idx] {
                    panic!(format!(
                        "Attempted to delete fire but no fire in position {} {}.",
                        pos.x, pos.y
                    ))
                }
            }
            None => panic!("Attempted to delete fire, but fire has no position.")
        }
    }
    { // Contain second borrow of ECS.
        let mut map = ecs.fetch_mut::<Map>();
        map.fire[idx] = false;
    }
    ecs.delete_entity(*entity).expect("Unable to remove fire entity.");
}

//----------------------------------------------------------------------------
// Potions
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
    .with(Untargeted {verb: "drinks".to_string()})
    .with(Throwable {})
    .with(Targeted {verb: "throws".to_string()})
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
    .with(Useable {})
    .with(Untargeted {verb: "drinks".to_string()})
    .with(Throwable {})
    .with(Targeted {verb: "throws".to_string()})
    .with(Consumable {})
    .with(ProvidesFireImmunityWhenUsed {turns: 50})
    .with(AreaOfEffectWhenTargeted {radius: 2})
    .with(InflictsDamageWhenTargeted {
        damage: 10,
        kind: ElementalDamageKind::Fire
    })
    .with(InflictsBurningWhenTargeted {
        turns: 5,
        tick_damage: 2
    })
    .with(SpawnsEntityInAreaWhenTargeted {
        radius: 1,
        kind: EntitySpawnKind::Fire {
            spread_chance: 50,
            dissipate_chance: 50,
        }
    })
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
    .with(Targeted {verb: "throws".to_string()})
    .with(Consumable {})
    .with(InflictsDamageWhenTargeted {damage: 10, kind: ElementalDamageKind::Fire})
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
    .with(Untargeted{ verb: "drinks".to_string()})
    .with(Throwable {})
    .with(Targeted {verb: "throws".to_string()})
    .with(Consumable {})
    .with(MovesToRandomPosition {})
    .marked::<SimpleMarker<SerializeMe>>()
    .build();
}

//----------------------------------------------------------------------------
// Food
//----------------------------------------------------------------------------
fn turnip(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437(';'),
        fg: RGB::named(rltk::WHITE),
        bg: RGB::named(rltk::BLACK),
        order: 2,
    })
    .with(Name {name: "Turnip".to_string()})
    .with(PickUpable {})
    .with(Useable {})
    .with(Consumable {})
    .with(Untargeted {verb: "eats".to_string()})
    // TODO: We probably want a component for triggering the healing animation.
    .with(ProvidesFullFood {})
    .marked::<SimpleMarker<SerializeMe>>()
    .build();
}

fn pomegranate(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437(';'),
        fg: RGB::named(rltk::RED),
        bg: RGB::named(rltk::BLACK),
        order: 2,
    })
    .with(Name {name: "Pomegranate".to_string()})
    .with(PickUpable {})
    .with(Useable {})
    .with(Consumable {})
    .with(Untargeted {verb: "eats".to_string()})
    // TODO: We probably want a component for triggering the healing animation.
    .with(ProvidesFullFood {})
    .with(ProvidesFullHealing {})
    .with(IncreasesMaxHpWhenUsed {amount: 5})
    .marked::<SimpleMarker<SerializeMe>>()
    .build();
}

//----------------------------------------------------------------------------
// Equipment
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
        .with(Targeted {verb: "throws".to_string()})
        .with(Consumable {})
        .with(InflictsDamageWhenTargeted {damage: 15, kind: ElementalDamageKind::Physical})
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
        .with(GrantsMeleeDefenseBonus {bonus: 3})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

//----------------------------------------------------------------------------
// Spells (well, Scrolls for now).
//----------------------------------------------------------------------------
fn fireblast(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
        })
        .with(Name {name: "Scroll of Fireblast".to_string()})
        .with(PickUpable {})
        .with(Castable {})
        .with(SpellCharges {
            max_charges: 3,
            charges: 1,
            regen_time: 200,
            time: 0
        })
        .with(Targeted {verb: "casts".to_string()})
        .with(InflictsDamageWhenTargeted {damage: 10, kind: ElementalDamageKind::Fire})
        .with(InflictsBurningWhenTargeted {turns: 4, tick_damage: 4})
        .with(AreaOfEffectWhenTargeted {radius: 2})
        .with(SpawnsEntityInAreaWhenTargeted {
            radius: 1,
            kind: EntitySpawnKind::Fire {
                spread_chance: 50,
                dissipate_chance: 50,
            }
        })
        .with(AreaOfEffectAnimationWhenTargeted {
            radius: 2,
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::RED),
            glyph: rltk::to_cp437('^')
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn iceblast(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('♪'),
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
        })
        .with(Name {name: "Scroll of Iceblast".to_string()})
        .with(PickUpable {})
        .with(Castable {})
        .with(SpellCharges {
            max_charges: 3,
            charges: 1,
            regen_time: 200,
            time: 0
        })
        .with(Targeted {verb: "casts".to_string()})
        .with(InflictsDamageWhenTargeted {damage: 10, kind: ElementalDamageKind::Fire})
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