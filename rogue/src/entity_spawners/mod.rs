use std::collections::HashMap;
use crate::map_builders::{GrassGeometry, WaterGeometry, NoiseMaps};

use super::{
    Point, Map, TileType, EntitySpawnKind, BlocksTile, CombatStats, SwimStamina,
    HungerClock, HungerState, Monster, Hazard, IsEntityKind,
    MovementRoutingAvoids, MovementRoutingBounds, MonsterBasicAI,
    MonsterAttackSpellcasterAI, MonsterSupportSpellcasterAI,
    SupportSpellcasterKind, Name, Player, Position, Renderable, SetsBgColor,
    Viewshed, PickUpable, Useable, Castable, SpellCharges, Equippable,
    EquipmentSlot, Throwable, TargetedWhenThrown, TargetedWhenCast,
    TargetingKind, Untargeted, Consumable, ProvidesFullHealing,
    ProvidesFullFood, IncreasesMaxHpWhenUsed, InflictsDamageWhenThrown, InflictsBurningWhenCast, InflictsBurningWhenThrown,
    InflictsDamageWhenEncroachedUpon, InflictsFreezingWhenCast, InflictsFreezingWhenThrown,
    BuffsMeleeAttackWhenCast, InflictsDamageWhenCast,
    BuffsPhysicalDefenseWhenCast, InflictsBurningWhenEncroachedUpon,
    InflictsFreezingWhenEncroachedUpon, AreaOfEffectAnimationWhenThrown, AreaOfEffectAnimationWhenCast,
    AlongRayAnimationWhenThrown, AlongRayAnimationWhenCast, MovesToRandomPosition,
    MoveToPositionWhenCast, SpawnEntityWhenMeleeAttacked,
    SpawnsEntityInAreaWhenCast, SpawnsEntityInAreaWhenThrown, ChanceToSpawnAdjacentEntity,
    ChanceToDissipate, SkipRandomDissipationForOneTurn,
    ChanceToInflictBurningOnAdjacentEntities, MeeleAttackWepon,
    GrantsMeleeDefenseBonus, ProvidesFireImmunityWhenUsed,
    ProvidesChillImmunityWhenUsed, ProvidesFullSpellRecharge,
    DecreasesSpellRechargeWhenUsed, CanNotAct, SimpleMarker, SerializeMe,
    MarkedBuilder, ElementalDamageKind, InSpellBook, StatusIsImmuneToFire,
    StatusIsImmuneToChill, BlessingOrbBag, BlessingOrb, BlessingSlot,
    SpawnEntityWhenKilled, DissipateWhenBurning,
    InvisibleWhenEncroachingEntityKind, WeaponSpecial, WeaponSpecialKind,
    UseFgColorMap, FgColorMap, UseBgColorMap, BgColorMap, Bloodied,
    UseFgColorMapWhenBloodied, MeeleAttackFormation, MAP_WIDTH, random_table
};
use rltk::RandomNumberGenerator;
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
    Sword,
    Raiper,
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
            .insert(ItemType::Turnip, 8 + depth)
            .insert(ItemType::Pomegranate, depth)
            .insert(ItemType::HealthPotion, 4 + depth)
            .insert(ItemType::RechargingPotion, depth)
            .insert(ItemType::TeleportationPotion, 2 + depth)
            .insert(ItemType::FirePotion, depth)
            .insert(ItemType::FreezingPotion, depth)
            .insert(ItemType::Dagger, 3 + depth)
            .insert(ItemType::Sword, 1 + depth)
            .insert(ItemType::Raiper, 1 + depth)
            .insert(ItemType::LeatherArmor, 1 + depth)
            .insert(ItemType::BlinkScroll, depth)
            .insert(ItemType::FireblastScroll, depth)
            .insert(ItemType::IceblastScroll, depth)
            .insert(ItemType::None, 75)
            // These are avalable at blessing statues.
            // .insert(ItemType::MagicMissileScroll, depth)
            // .insert(ItemType::FireballScroll, depth)
            // .insert(ItemType::IcespikeScroll, depth)
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
        Some(ItemType::Sword) => equipment::sword(ecs, x, y),
        Some(ItemType::Raiper) => equipment::rapier(ecs, x, y),
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
// Determining which monster to spawn:
// To each monster type we associate some data: a diffuiculty score, a minimum
// and maximum floor depth, a (relative) chance to spawn, and constraints on the
// spawn tile. When spawning a monster, we filter this table to only those
// monsters that can spawn on the current floor, then choose a monster at random
// according the data.
//
// Determining how many monsters to spawn.
// As noted, each monster type has a difficuly associated with it. For each
// floor of the dungeon, we associate a total difficulty quota. Then, when
// spawning monsters, we continue until the total difficulty of all monster
// spawned first exceeds the floor's quota.
//
// Determining spawn positions
// The NoiseMaps object in the ECS encapsulates assoreted randomly sampled
// structures useful for making decisions at random that must reqpect geometric
// structure. This exposes two methods we use here to query for monster spawning
// locations:
//
//   general_monster_spawn_position_buffer: Vec<Point>
//   grass_monster_spawn_position_buffer: Vec<Point>
//
// Both of these buffers provide a next spawn position by `pop`ing off the next
// point. For the construction and structure of these buffers, see NoiseMaps.
//----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MonsterType {
    Rat,
    Bat,
    Snake,
    Piranha,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MonsterSpawnBound {
    None,
    Grass,
    Water
}

// Data container for parameters describing a monster's behaviour relative to
// the monster spawning algorithm. See get_monster_spawn_table below for more
// details.
struct MonsterSpawnParameters {
    difficulty: i32,
    min_depth: i32,
    max_depth: i32,
    chance: i32,
    bound: MonsterSpawnBound
}
impl MonsterSpawnParameters {
    fn new(difficulty: i32, min_depth: i32, max_depth: i32, chance: i32, bound: MonsterSpawnBound) -> MonsterSpawnParameters {
        MonsterSpawnParameters {
            difficulty, min_depth, max_depth, chance, bound
        }
    }
}

// Construct the (static) monster spawn table. This maps the monster type to a
// container of parameters describing their behaviour relative to the monster
// spawning algorithm.
//
// Spawn Parameters:
//
// difficulty: i32
//   Each floor has a static difficulty quote that increases as the player
//   descends deeper into the dungeon, and we spawn monsters until this quota is
//   exceeded. Each monster spawned contributes a static amount to this quota.
// min_depth: i32
//   The depth at which the monster is first avalable to spawn. This monster
//   will not spawn on shallower floors.
// max_depth: i32
//   The depth at which the monster is last avalable to spawn. This monster will
//   not spawn on deeper floors.
// chance: i32
//   The *relative* chance this monster spawns in a single spawn event, taken in
//   ratio to the total chance of all monster types available to spawn on the
//   given floor.
// bound: MonsterSpawnBound
//   Constraints on the location of spawning this monster. For example, a Grass
//   constraint is used for monsters that only occupy grass.
fn get_monster_spawn_table() -> HashMap<MonsterType, MonsterSpawnParameters> {
    let mut spawn_table: HashMap<MonsterType, MonsterSpawnParameters> = HashMap::new();
    //                                                 difficulty, min_depth, max_depth, chance, spawn_bounds.
    spawn_table.insert(MonsterType::Rat,               MonsterSpawnParameters::new(1, 1, 4, 25, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::Bat,               MonsterSpawnParameters::new(1, 1, 4, 25, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::Snake,             MonsterSpawnParameters::new(1, 1, 4, 25, MonsterSpawnBound::Grass));
    spawn_table.insert(MonsterType::Piranha,           MonsterSpawnParameters::new(1, 1, 6, 25, MonsterSpawnBound::Water));
    spawn_table.insert(MonsterType::GoblinBasic,       MonsterSpawnParameters::new(2, 1, 6, 25, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::GoblinCleric,      MonsterSpawnParameters::new(2, 2, 8, 20, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::GoblinEnchanter,   MonsterSpawnParameters::new(2, 2, 8, 20, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::GoblinFirecaster,  MonsterSpawnParameters::new(4, 3, 8, 15, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::GoblinChillcaster, MonsterSpawnParameters::new(4, 3, 8, 15, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::PinkJelly,         MonsterSpawnParameters::new(3, 4, 10, 15, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::OrangeJelly,       MonsterSpawnParameters::new(4, 4, 12, 10, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::BlueJelly,         MonsterSpawnParameters::new(4, 4, 12, 10, MonsterSpawnBound::None));
    spawn_table.insert(MonsterType::Orc,               MonsterSpawnParameters::new(4, 5, 12, 25, MonsterSpawnBound::None));
    spawn_table
}

// Main entrypoint for spawning monsters on a floor of the dungeon. The public
// function in the monster spawning API.
pub fn spawn_monsters(ecs: &mut World, depth: i32) {
    let monsters_with_locations = sample_monster_spawn_vector(ecs, depth);
    for (monster, Point {x, y}) in monsters_with_locations.into_iter() {
        insert_monster(ecs, monster, x, y);
    }
}

// TODO: When we get around to implementing a more general drop system, this
// should become part of that. We leave it here for convenience at this point.
const CHANCE_DROP_ORB: i32 = 20;

// Insert a new monster into the ECS.
fn insert_monster(ecs: &mut World, monster: MonsterType, x: i32, y: i32) {
    let entity = match monster {
        MonsterType::Rat => monsters::rat(ecs, x, y),
        MonsterType::Bat => monsters::bat(ecs, x, y),
        MonsterType::Snake => monsters::snake(ecs, x, y),
        MonsterType::Piranha => monsters::piranha(ecs, x, y),
        MonsterType::GoblinBasic => monsters::goblin_basic(ecs, x, y),
        MonsterType::GoblinCleric => monsters::goblin_cleric(ecs, x, y),
        MonsterType::GoblinEnchanter => monsters::goblin_enchanter(ecs, x, y),
        MonsterType::GoblinFirecaster => monsters::goblin_firecaster(ecs, x, y),
        MonsterType::GoblinChillcaster => monsters::goblin_chillcaster(ecs, x, y),
        MonsterType::Orc => monsters::orc_basic(ecs, x, y),
        MonsterType::PinkJelly => monsters::pink_jelly(
            ecs, x, y, monsters::JELLY_BASE_HP, monsters::JELLY_BASE_HP),
        MonsterType::OrangeJelly => monsters::orange_jelly(ecs, x, y),
        MonsterType::BlueJelly => monsters::blue_jelly(ecs, x, y),
    };
    // TODO: Same as above, this should really be part of a more general drop
    // system.
    if let Some(entity) = entity {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        if rng.roll_dice(1, 100) <= CHANCE_DROP_ORB {
            let mut drops = ecs.write_storage::<SpawnEntityWhenKilled>();
            drops
                .insert(entity, SpawnEntityWhenKilled {kind: EntitySpawnKind::MagicOrb})
                .expect("Could not add orb drop to monster.");
        }
    }
}

// Assemble a vector indicating which monsters to spawn and where.
//
// Note:
// This function does not actually modify the ECS, i.e., the monsters are not
// inserted here. We only implement the random spawning patters and collect a
// data structure for the monster spawn requests we want to make.
fn sample_monster_spawn_vector(ecs: &mut World, depth: i32) -> Vec<(MonsterType, Point)> {

    let spawn_table = get_monster_spawn_table();
    let mut rtable: random_table::RandomTable<(MonsterType, MonsterSpawnBound)>
        = random_table::RandomTable::new();
    for (monster_type, spawn_parameters) in spawn_table.iter() {
        let ok_to_spawn_by_depth = (spawn_parameters.min_depth <= depth)
            && (depth <= spawn_parameters.max_depth);
        let ok_to_spawn_by_bound = match spawn_parameters.bound {
            MonsterSpawnBound::None => true,
            MonsterSpawnBound::Grass => {
                let noisemaps = ecs.fetch::<NoiseMaps>();
                match noisemaps.grass_geometry {
                    GrassGeometry::None => false,
                    _ => true,
                }
            }
            MonsterSpawnBound::Water => {
                let noisemaps = ecs.fetch::<NoiseMaps>();
                match noisemaps.water_geometry {
                    WaterGeometry::None => false,
                    _ => true,
                }
            }
        };
        if ok_to_spawn_by_depth && ok_to_spawn_by_bound {
            rtable = rtable.insert((*monster_type, spawn_parameters.bound), spawn_parameters.chance);
        }
    }

    let mut monster_spawns_with_bounds: Vec<(MonsterType, MonsterSpawnBound)> = Vec::new();
    let difficulty_quota: i32 = 10 + (5 * depth) / 2;
    let mut seen_difficulty: i32 = 0;
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    loop {
        let (monster, bound) = rtable.roll(&mut rng).expect("Monster spawn sampling failed.");
        let difficulty = spawn_table[&monster].difficulty;
        monster_spawns_with_bounds.push((monster, bound));
        seen_difficulty += difficulty;
        if seen_difficulty > difficulty_quota {
            break;
        }
    }

    let mut base_monster_spawn_locations: Vec<Point>;
    let mut grass_monster_spawn_locations: Vec<Point>;
    let mut water_monster_spawn_locations: Vec<Point>;
    {
        let noisemaps = ecs.fetch::<NoiseMaps>();
        base_monster_spawn_locations = noisemaps.general_monster_spawn_position_buffer();
        grass_monster_spawn_locations = noisemaps.grassbound_monster_spawn_position_buffer();
        water_monster_spawn_locations = noisemaps.waterbound_monster_spawn_position_buffer();
    }

    let mut monsters_with_locations: Vec<(MonsterType, Point)> = Vec::new();
    let mut map = ecs.fetch_mut::<Map>();
    for (monster, bound) in monster_spawns_with_bounds.iter() {
        let (mut loc, mut idx): (Point, usize);
        match bound {
            MonsterSpawnBound::None => {
                loop {
                    loc = base_monster_spawn_locations.pop()
                        .expect("Failed to pop spawn location.");
                    idx = map.xy_idx(loc.x, loc.y);
                    if map.ok_to_spawn[idx] && !map.deep_water[idx] { break; }
                }
            }
            MonsterSpawnBound::Grass => {
                loop {
                    loc = grass_monster_spawn_locations.pop()
                        .expect("Failed to pop spawn location.");
                    idx = map.xy_idx(loc.x, loc.y);
                    let ok_to_spawn =
                        map.ok_to_spawn[idx]
                        && map.grass[idx]
                        && !map.deep_water[idx];
                    if ok_to_spawn { break; }
                }
            }
            MonsterSpawnBound::Water => {
                loop {
                    loc = water_monster_spawn_locations.pop()
                        .expect("Failed to pop spawn location.");
                    idx = map.xy_idx(loc.x, loc.y);
                    let ok_to_spawn =
                        map.ok_to_spawn[idx]
                        && (map.shallow_water[idx] || map.deep_water[idx]);
                    if ok_to_spawn { break; }
                }
            }
        }
        if !map.ok_to_spawn[idx] { continue; }
        map.ok_to_spawn[idx] = false;
        monsters_with_locations.push((*monster, loc));
    }
    monsters_with_locations
}

