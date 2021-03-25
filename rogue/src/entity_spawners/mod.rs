use super::{
    Map, TileType, EntitySpawnKind, BlocksTile, CombatStats, HungerClock,
    HungerState, Monster, Hazard, IsEntityKind, MonsterBasicAI,
    MonsterAttackSpellcasterAI, MovementRoutingOptions, Name, Player,
    Position, Renderable, Viewshed, PickUpable, Useable, Castable,
    SpellCharges, Equippable, EquipmentSlot, Throwable, Targeted,
    TargetingKind, Untargeted, Consumable, ProvidesFullHealing,
    ProvidesFullFood, IncreasesMaxHpWhenUsed, InflictsDamageWhenTargeted,
    InflictsDamageWhenEncroachedUpon, InflictsFreezingWhenTargeted,
    InflictsBurningWhenTargeted, InflictsBurningWhenEncroachedUpon,
    InflictsFreezingWhenEncroachedUpon, AreaOfEffectAnimationWhenTargeted,
    AlongRayAnimationWhenTargeted, MovesToRandomPosition,
    MoveToPositionWhenTargeted, SpawnsEntityInAreaWhenTargeted,
    ChanceToSpawnAdjacentEntity, ChanceToDissipate, GrantsMeleeAttackBonus,
    GrantsMeleeDefenseBonus, ProvidesFireImmunityWhenUsed,
    ProvidesChillImmunityWhenUsed, SimpleMarker, SerializeMe, MarkedBuilder,
    ElementalDamageKind, InSpellBook,
    MAP_WIDTH, random_table
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
mod potions;
mod equipment;
mod spells;
mod monsters;
mod food;
pub mod hazards;

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
    GoblinBasic,
    GoblinFirecaster,
    GoblinChillcaster,
    Orc
}
fn spawn_random_monster(ecs: &mut World, x: i32, y: i32, depth: i32) {
    let monster: Option<MonsterType>;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        // TODO: Make this table in a less stupid place.
        monster = random_table::RandomTable::new()
            .insert(MonsterType::GoblinBasic, 20)
            .insert(MonsterType::GoblinFirecaster, 1 + depth)
            .insert(MonsterType::GoblinChillcaster, 1 + depth)
            .insert(MonsterType::Orc, 3 + 3 * (depth-1))
            .insert(MonsterType::None, 70 - depth)
            .roll(&mut rng);
    }
    match monster {
        Some(MonsterType::GoblinBasic) => monsters::goblin_basic(ecs, x, y),
        Some(MonsterType::GoblinFirecaster) => monsters::goblin_firecaster(ecs, x, y),
        Some(MonsterType::GoblinChillcaster) => monsters::goblin_chillcaster(ecs, x, y),
        Some(MonsterType::Orc) => monsters::orc_basic(ecs, x, y),
        _ => {None}
    };
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
    MagicMissileScroll,
    BlinkScroll,
    FireblastScroll,
    FireballScroll,
    IceblastScroll,
    IcespikeScroll,
}
fn spawn_random_item(ecs: &mut World, x: i32, y: i32, depth: i32) {
    let item: Option<ItemType>;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        // TODO: Make this table in a less stupid place.
        item = random_table::RandomTable::new()
            .insert(ItemType::Turnip, 4 + depth)
            .insert(ItemType::Pomegranate, depth)
            .insert(ItemType::HealthPotion, 4 + depth)
            .insert(ItemType::TeleportationPotion, 2 + depth)
            .insert(ItemType::FirePotion, 2 + depth)
            .insert(ItemType::FreezingPotion, 2 + depth)
            .insert(ItemType::Dagger, depth)
            .insert(ItemType::LeatherArmor, depth)
            .insert(ItemType::MagicMissileScroll, depth)
            .insert(ItemType::BlinkScroll, depth)
            .insert(ItemType::FireblastScroll, depth)
            .insert(ItemType::FireballScroll, depth)
            .insert(ItemType::IceblastScroll, depth)
            .insert(ItemType::IcespikeScroll, depth)
            .insert(ItemType::None, 100)
            .roll(&mut rng);
    }
    match item {
        Some(ItemType::Turnip) => food::turnip(ecs, x, y),
        Some(ItemType::Pomegranate) => food::pomegranate(ecs, x, y),
        Some(ItemType::HealthPotion) => potions::health(ecs, x, y),
        Some(ItemType::TeleportationPotion) => potions::teleportation(ecs, x, y),
        Some(ItemType::FirePotion) => potions::fire(ecs, x, y),
        Some(ItemType::FreezingPotion) => potions::freezing(ecs, x, y),
        Some(ItemType::Dagger) => equipment::dagger(ecs, x, y),
        Some(ItemType::LeatherArmor) => equipment::leather_armor(ecs, x, y),
        Some(ItemType::MagicMissileScroll) => spells::magic_missile(ecs, x, y, 10, 5),
        Some(ItemType::BlinkScroll) => spells::blink(ecs, x, y),
        Some(ItemType::FireblastScroll) => spells::fireblast(ecs, x, y),
        Some(ItemType::FireballScroll) => spells::fireball(ecs, x, y, 5, 2),
        Some(ItemType::IceblastScroll) => spells::iceblast(ecs, x, y),
        Some(ItemType::IcespikeScroll) => spells::icespike(ecs, x, y, 5, 2),
        Some(ItemType::None) => {None},
        None => {None}
    };
}