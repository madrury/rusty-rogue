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
    ChanceToDissipate, ChanceToInflictBurningOnAdjacentEntities,
    GrantsMeleeAttackBonus, GrantsMeleeDefenseBonus,
    ProvidesFireImmunityWhenUsed, ProvidesChillImmunityWhenUsed,
    ProvidesFullSpellRecharge, DecreasesSpellRechargeWhenUsed, CanNotAct,
    SimpleMarker, SerializeMe, MarkedBuilder, ElementalDamageKind,
    InSpellBook, StatusIsImmuneToFire, StatusIsImmuneToChill,
    MAP_WIDTH, random_table
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
mod potions;
mod equipment;
mod spells;
pub mod monsters;
mod food;
pub mod hazards;
pub mod player;


const MAX_MONSTERS_IN_ROOM: i32 = 4;
const MAX_ITEMS_IN_ROOM: i32 = 2;

// Populate a reigon (defined by a container of map indexes) with monsters and
// items.
// TODO: This should jsut get the map so we don't need to import the MAP_WIDTH
// and MAP_HEIGHT constants.
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
        { // Holy scopes!
            let mut map = ecs.write_resource::<Map>();
            if !map.ok_to_spawn[*idx] {continue;}
            map.ok_to_spawn[*idx] = false;
        }
        spawn_random_monster(ecs, x, y, depth);
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

// Spawns a randomly chosen monster at a specified location.
#[derive(Clone, Copy)]
enum MonsterType {
    None,
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
fn spawn_random_monster(ecs: &mut World, x: i32, y: i32, depth: i32) {
    let monster: Option<MonsterType>;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        // TODO: Make this table in a less stupid place.
        monster = random_table::RandomTable::new()
            .insert(MonsterType::Rat, 25)
            .insert(MonsterType::Bat, 25)
            .insert(MonsterType::GoblinBasic, 10)
            .insert(MonsterType::GoblinCleric, 2 + depth)
            .insert(MonsterType::GoblinEnchanter, 2 + depth)
            .insert(MonsterType::GoblinFirecaster, depth - 1)
            .insert(MonsterType::GoblinChillcaster, depth - 1)
            .insert(MonsterType::Orc, depth - 2)
            .insert(MonsterType::PinkJelly, depth)
            .insert(MonsterType::OrangeJelly, depth - 1)
            .insert(MonsterType::BlueJelly, depth - 1)
            .insert(MonsterType::None, 70)
            .roll(&mut rng);
    }
    match monster {
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
            .insert(ItemType::FirePotion, depth)
            .insert(ItemType::FreezingPotion, depth)
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