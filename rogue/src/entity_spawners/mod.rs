use std::collections::HashMap;
use super::{
    Point, Map, TileType, EntitySpawnKind, BlocksTile, CombatStats,
    SwimStamina, HungerClock, HungerState, Monster, Hazard, IsEntityKind,
    MonsterMovementRoutingOptions, MonsterBasicAI,
    MonsterAttackSpellcasterAI, MonsterSupportSpellcasterAI,
    SupportSpellcasterKind, MovementRoutingOptions, Name, Player, Position,
    Renderable, SetsBgColor, Viewshed, PickUpable, Useable, Castable,
    SpellCharges, Equippable, EquipmentSlot, Throwable, Targeted,
    TargetingKind, Untargeted, Consumable, ProvidesFullHealing,
    ProvidesFullFood, IncreasesMaxHpWhenUsed, InflictsDamageWhenTargeted,
    InflictsDamageWhenEncroachedUpon, InflictsFreezingWhenTargeted,
    InflictsBurningWhenTargeted, BuffsMeleeAttackWhenTargeted,
    BuffsPhysicalDefenseWhenTargeted, InflictsBurningWhenEncroachedUpon,
    InflictsFreezingWhenEncroachedUpon, AreaOfEffectAnimationWhenTargeted,
    AlongRayAnimationWhenTargeted, MovesToRandomPosition,
    MoveToPositionWhenTargeted, SpawnEntityWhenMeleeAttacked,
    SpawnsEntityInAreaWhenTargeted, ChanceToSpawnAdjacentEntity,
    ChanceToDissipate, SkipRandomDissipationForOneTurn,
    ChanceToInflictBurningOnAdjacentEntities, GrantsMeleeAttackBonus,
    GrantsMeleeDefenseBonus, ProvidesFireImmunityWhenUsed,
    ProvidesChillImmunityWhenUsed, ProvidesFullSpellRecharge,
    DecreasesSpellRechargeWhenUsed, CanNotAct, SimpleMarker, SerializeMe,
    MarkedBuilder, ElementalDamageKind, InSpellBook, StatusIsImmuneToFire,
    StatusIsImmuneToChill, BlessingOrbBag, BlessingOrb, SpawnEntityWhenKilled,
    MAP_WIDTH, random_table, noise
};
use rltk::{RandomNumberGenerator};
use specs::prelude::*;

mod potions;
mod equipment;
pub mod spells;
pub mod monsters;
mod food;
pub mod hazards;
pub mod player;
pub mod magic;


//----------------------------------------------------------------------------
// Item Spawning.
//
// Item spawning requires chopping a map up into reigons, and then flipping a
// weighted coin to see if an item spawns in a given reigon.
//----------------------------------------------------------------------------
const CHANCE_SPAWN_ITEM: i32 = 25;

pub fn spawn_items_in_region(ecs: &mut World, region: &[usize], depth: i32) {
    let mut areas : Vec<usize> = Vec::from(region);
    if areas.is_empty() {return;}

    let mut item_spawn_points: Vec<usize> = Vec::new();
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let do_spawn_item = rng.roll_dice(1, 100) <= CHANCE_SPAWN_ITEM;
        if do_spawn_item && !areas.is_empty() {
            let array_index = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };
            let map_idx = areas[array_index];
            item_spawn_points.push(map_idx);
            areas.remove(array_index);
        }
    }

    for idx in item_spawn_points.iter() {
        let (x, y) = (*idx as i32 % MAP_WIDTH, *idx as i32 / MAP_WIDTH);
        {
            let mut map = ecs.write_resource::<Map>();
            if !map.ok_to_spawn[*idx] {continue;}
            map.ok_to_spawn[*idx] = false;
        }
        spawn_random_item(ecs, x, y, depth);
    }
}
// Spawns a randomly chosen item at a specified location.
#[derive(Clone, Copy)]
enum ItemType {
    None,
    Turnip,
    Pomegranate,
    HealthPotion,
    RechargingPotion,
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
            .insert(ItemType::RechargingPotion, depth)
            .insert(ItemType::TeleportationPotion, 2 + depth)
            .insert(ItemType::FirePotion, 200)
            .insert(ItemType::FreezingPotion, 200)
            .insert(ItemType::Dagger, depth)
            .insert(ItemType::LeatherArmor, depth)
            .insert(ItemType::MagicMissileScroll, 1 + depth)
            .insert(ItemType::BlinkScroll, depth)
            .insert(ItemType::FireblastScroll, depth)
            .insert(ItemType::FireballScroll, depth)
            .insert(ItemType::IceblastScroll, depth)
            .insert(ItemType::IcespikeScroll, depth)
            .insert(ItemType::None, 75)
            .roll(&mut rng);
    }
    match item {
        Some(ItemType::Turnip) => food::turnip(ecs, x, y),
        Some(ItemType::Pomegranate) => food::pomegranate(ecs, x, y),
        Some(ItemType::HealthPotion) => potions::health(ecs, x, y),
        Some(ItemType::RechargingPotion) => potions::recharging(ecs, x, y),
        Some(ItemType::TeleportationPotion) => potions::teleportation(ecs, x, y),
        Some(ItemType::FirePotion) => potions::fire(ecs, x, y),
        Some(ItemType::FreezingPotion) => potions::freezing(ecs, x, y),
        Some(ItemType::Dagger) => equipment::dagger(ecs, x, y),
        Some(ItemType::LeatherArmor) => equipment::leather_armor(ecs, x, y),
        Some(ItemType::MagicMissileScroll) => spells::magic_missile(ecs, x, y, 10, 5),
        Some(ItemType::BlinkScroll) => spells::blink(ecs, x, y),
        Some(ItemType::FireblastScroll) => spells::fireblast(ecs, x, y),
        Some(ItemType::FireballScroll) => spells::fireball(ecs, x, y, 5, 1),
        Some(ItemType::IceblastScroll) => spells::iceblast(ecs, x, y),
        Some(ItemType::IcespikeScroll) => spells::icespike(ecs, x, y, 5, 1),
        Some(ItemType::None) => {None},
        None => {None}
    };
}


//----------------------------------------------------------------------------
// Monster Spawning.
//
// Spawn the monster antagonists in a dungeon floor. There a few concepts used
// in this spawning algorithm:
//
// Determining spawn positions:
// We build a vector of (x, y) coordinates, and spawn monsters in the order of
// points in this vector. The vector of spawn positions is created by summing
// smooth and white random nose, then sorting all the map coordinates according
// to the value of this nose. The large values of noise thus created tend to
// cluster due to the smooth component, so we end up generating monsters in lose
// clusters.
//
// Determining which monster to spawn:
// To each monster type we associate some data: a diffuiculty score, a minimum
// and maximum floor depth, and a chance to spawn. When spawning a monster, we
// filter this table to only those monsters that can spawn on the current floor,
// then choose a monster at random using the change data.
//
// Determining how many monsters to spawn.
// As noted above, each monster type has a difficuly associated with it. For
// each floor of the dungeon, we associate a total difficulty quota. Then, when
// spawning monsters, we continue until the total difficulty of all monster
// spawned first exceeds the floor's quota.
//----------------------------------------------------------------------------
pub fn spawn_monsters(ecs: &mut World, depth: i32) {
    let monster_difficulty_quota: i32 = 10 + (5 * depth) / 2;
    let monster_spawn_table = get_monster_spawn_table();
    let mut monster_seen_difficulty: i32 = 0;
    let mut monster_spawn_locations: Vec<(i32, i32)>;
    {
        let map = ecs.fetch::<Map>();
        monster_spawn_locations = noise::monster_spawn_locations(&map);
    }
    loop {
        let point = monster_spawn_locations.pop()
            .expect("Ran out of Monster spawn locations.");
        { // Holy scopes!
            let mut map = ecs.write_resource::<Map>();
            let idx = map.xy_idx(point.0, point.1);
            if !map.ok_to_spawn[idx] {continue;}
            map.ok_to_spawn[idx] = false;
        }
        let monster_type = spawn_random_monster(ecs, point.0, point.1, depth);
        if let Some(monster_type) = monster_type {
            let monster_difficulty = monster_spawn_table[&monster_type].difficulty;
            monster_seen_difficulty += monster_difficulty;
        }
        if monster_seen_difficulty > monster_difficulty_quota {
            break;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum MonsterType {
    Rat,
    Bat,
    GoblinBasic,
    GoblinCleric,
    GoblinEnchanter,
    GoblinFirecaster,
    GoblinChillcaster,
    Orc,
    PinkJelly,
    OrangeJelly,
    BlueJelly
}

// Entries in the monster spawn table.
struct MonsterSpawnParameters {
    // A positive interger representing the difficulty of an encounter with a
    // given monster. We spawn monsters on a floor by fufilling a difficulty
    // quota.
    difficulty: i32,
    // The minimum floor depth at which a monster can spawn.
    min_depth: i32,
    // The maximum floor depth at which a monster can spawn.
    max_depth: i32,
    // A positive integer representing the chance that a monster spawns in a
    // singular spawn event.
    chance: i32
}
impl MonsterSpawnParameters {
    fn new(difficulty: i32, min_depth: i32, max_depth: i32, chance: i32) -> MonsterSpawnParameters {
        MonsterSpawnParameters {
            difficulty, min_depth, max_depth, chance
        }
    }
}

// Construct the (static) monster spawn table.
fn get_monster_spawn_table() -> HashMap<MonsterType, MonsterSpawnParameters> {
    let mut spawn_table: HashMap<MonsterType, MonsterSpawnParameters> = HashMap::new();
    //                                                 difficulty, min-depth, max-depth, chance.
    spawn_table.insert(MonsterType::Rat,               MonsterSpawnParameters::new(1, 1, 4, 25));
    spawn_table.insert(MonsterType::Bat,               MonsterSpawnParameters::new(1, 1, 4, 25));
    spawn_table.insert(MonsterType::GoblinBasic,       MonsterSpawnParameters::new(2, 1, 6, 25));
    spawn_table.insert(MonsterType::GoblinCleric,      MonsterSpawnParameters::new(2, 2, 8, 20));
    spawn_table.insert(MonsterType::GoblinEnchanter,   MonsterSpawnParameters::new(2, 2, 8, 20));
    spawn_table.insert(MonsterType::GoblinFirecaster,  MonsterSpawnParameters::new(4, 3, 8, 15));
    spawn_table.insert(MonsterType::GoblinChillcaster, MonsterSpawnParameters::new(4, 3, 8, 15));
    spawn_table.insert(MonsterType::PinkJelly,         MonsterSpawnParameters::new(3, 4, 10, 15));
    spawn_table.insert(MonsterType::OrangeJelly,       MonsterSpawnParameters::new(4, 4, 12, 10));
    spawn_table.insert(MonsterType::BlueJelly,         MonsterSpawnParameters::new(4, 4, 12, 10));
    spawn_table.insert(MonsterType::Orc,               MonsterSpawnParameters::new(4, 5, 12, 25));
    spawn_table
}

// Spawn a single random monster at a given depth, according to the static spawn
// table given above.
const CHANCE_DROP_ORB: i32 = 100;

fn spawn_random_monster(ecs: &mut World, x: i32, y: i32, depth: i32) -> Option<MonsterType> {
    let monster: Option<MonsterType>;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let spawn_table = get_monster_spawn_table();
        let mut rtable: random_table::RandomTable<MonsterType> = random_table::RandomTable::new();
        for (monster_type, spawn_parameters) in spawn_table.into_iter() {
            if spawn_parameters.min_depth <= depth && depth <= spawn_parameters.max_depth {
                rtable = rtable.insert(monster_type, spawn_parameters.chance);
            }
        }
        monster = rtable.roll(&mut rng);
    }
    let entity = match monster {
        Some(MonsterType::Rat) => monsters::rat(ecs, x, y),
        Some(MonsterType::Bat) => monsters::bat(ecs, x, y),
        Some(MonsterType::GoblinBasic) => monsters::goblin_basic(ecs, x, y),
        Some(MonsterType::GoblinCleric) => monsters::goblin_cleric(ecs, x, y),
        Some(MonsterType::GoblinEnchanter) => monsters::goblin_enchanter(ecs, x, y),
        Some(MonsterType::GoblinFirecaster) => monsters::goblin_firecaster(ecs, x, y),
        Some(MonsterType::GoblinChillcaster) => monsters::goblin_chillcaster(ecs, x, y),
        Some(MonsterType::Orc) => monsters::orc_basic(ecs, x, y),
        Some(MonsterType::PinkJelly) => monsters::pink_jelly(
            ecs, x, y, monsters::JELLY_BASE_HP, monsters::JELLY_BASE_HP),
        Some(MonsterType::OrangeJelly) => monsters::orange_jelly(ecs, x, y),
        Some(MonsterType::BlueJelly) => monsters::blue_jelly(ecs, x, y),
        _ => {None}
    };
    if let Some(entity) = entity {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        if rng.roll_dice(1, 100) <= CHANCE_DROP_ORB {
            let mut drops = ecs.write_storage::<SpawnEntityWhenKilled>();
            drops
                .insert(entity, SpawnEntityWhenKilled {kind: EntitySpawnKind::MagicOrb})
                .expect("Could not add orb drop to monster.");
        }
    }
    monster
}
