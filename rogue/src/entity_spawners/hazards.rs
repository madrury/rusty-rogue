
use super::{
    Map, TileType, EntitySpawnKind, Hazard, IsEntityKind, Name, Position,
    Renderable, SetsBgColor, InflictsDamageWhenEncroachedUpon,
    InflictsBurningWhenEncroachedUpon, InflictsFreezingWhenEncroachedUpon,
    ChanceToSpawnAdjacentEntity, ChanceToDissipate,
    ChanceToInflictBurningOnAdjacentEntities, SimpleMarker, SerializeMe,
    MarkedBuilder, ElementalDamageKind
};
use rltk::{RGB};
use specs::prelude::*;


// A fire entity. Represents a burning tile that can spread and dissipate. All
// spawning or despawning of fire MUST use thiese functions, since it handles
// syncronizing the map.fire array.
const FIRE_ENCROACHMENT_DAMAGE: i32 = 2;
const FIRE_ADJACENT_BURNING_CHANCE: i32 = 25;
const FIRE_BURNING_TURNS: i32 = 4;
const FIRE_BURNING_TICK_DAMAGE: i32 = 2;
const FIRE_SPAWN_DISSIPATE_CHANCE_CHANGE: i32 = 20;
const FIRE_SPAWN_SPREAD_CHANCE_CHANGE: i32 = 20;

pub fn fire(ecs: &mut World, x: i32, y: i32, spread_chance: i32, dissipate_chance: i32) -> Option<Entity> {
    let can_spawn: bool;
    let idx: usize;
    {
        let map = ecs.fetch::<Map>();
        idx = map.xy_idx(x, y);
        can_spawn = !map.fire[idx]
            && !map.water[idx]
            && map.tiles[idx] != TileType::Wall;
    }
    let entity;
    if can_spawn {
        entity = ecs.create_entity()
            .with(Position {x, y})
            .with(Renderable {
                glyph: rltk::to_cp437('^'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::ORANGE),
                order: 2,
                visible_out_of_fov: false
            })
            .with(SetsBgColor {order: 1})
            .with(Name {name: "Fire".to_string()})
            .with(Hazard {})
            .with(IsEntityKind {
                kind: EntitySpawnKind::Fire {
                    spread_chance, dissipate_chance
                }
            })
            .with(ChanceToSpawnAdjacentEntity {
                chance: spread_chance,
                kind: EntitySpawnKind::Fire {
                    spread_chance: i32::max(
                        0, spread_chance - FIRE_SPAWN_SPREAD_CHANCE_CHANGE),
                    dissipate_chance: i32::max(
                        0, dissipate_chance + FIRE_SPAWN_DISSIPATE_CHANCE_CHANGE),
                }
            })
            .with(ChanceToInflictBurningOnAdjacentEntities {
                chance: FIRE_ADJACENT_BURNING_CHANCE
            })
            .with(ChanceToDissipate {
                chance: dissipate_chance
            })
            .with(InflictsDamageWhenEncroachedUpon {
                damage: FIRE_ENCROACHMENT_DAMAGE,
                kind: ElementalDamageKind::Fire
            })
            .with(InflictsBurningWhenEncroachedUpon {
                turns: FIRE_BURNING_TURNS,
                tick_damage: FIRE_BURNING_TICK_DAMAGE
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        map.fire[idx] = true;
        Some(entity)
    } else {
        None
    }
}

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

// A chill entity. Represents freezing air that can spread and dissipate. All
// spawning of chill MUST use this function, since it handles syncronizing the
// map.chill array.
const CHILL_ENCROACHMENT_DAMAGE: i32 = 2;
const CHILL_FROZEN_TURNS: i32 = 4;
const CHILL_SPAWN_DISSIPATE_CHANCE_CHANGE: i32 = 20;
const CHILL_SPAWN_SPREAD_CHANCE_CHANGE: i32 = 20;

pub fn chill(ecs: &mut World, x: i32, y: i32, spread_chance: i32, dissipate_chance: i32) -> Option<Entity> {
    let can_spawn: bool;
    let idx: usize;
    {
        let map = ecs.fetch::<Map>();
        idx = map.xy_idx(x, y);
        can_spawn = !map.chill[idx] && map.tiles[idx] != TileType::Wall;
    }
    let entity;
    if can_spawn {
        entity = ecs.create_entity()
            .with(Position {x, y})
            .with(Renderable {
                fg: RGB::named(rltk::WHITE),
                bg: RGB::named(rltk::LIGHT_BLUE),
                glyph: rltk::to_cp437('*'),
                order: 2,
                visible_out_of_fov: false
            })
            .with(SetsBgColor {order: 0})
            .with(Name {name: "Chill".to_string()})
            .with(Hazard {})
            .with(IsEntityKind {
                kind: EntitySpawnKind::Chill {
                    spread_chance, dissipate_chance
                }
            })
            .with(ChanceToSpawnAdjacentEntity {
                chance: spread_chance,
                kind: EntitySpawnKind::Chill {
                    spread_chance: i32::max(
                        0, spread_chance - CHILL_SPAWN_SPREAD_CHANCE_CHANGE),
                    dissipate_chance: i32::max(
                        0, dissipate_chance + CHILL_SPAWN_DISSIPATE_CHANCE_CHANGE),
                }
            })
            .with(ChanceToDissipate {
                chance: dissipate_chance
            })
            .with(InflictsDamageWhenEncroachedUpon {
                damage: CHILL_ENCROACHMENT_DAMAGE,
                kind: ElementalDamageKind::Chill
            })
            .with(InflictsFreezingWhenEncroachedUpon {
                turns: CHILL_FROZEN_TURNS
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        map.chill[idx] = true;
        Some(entity)
    } else {
        None
    }
}

pub fn destroy_chill(ecs: &mut World, entity: &Entity) {
    let idx;
    { // Contain first borrow of ECS.
        let positions = ecs.read_storage::<Position>();
        let map = ecs.fetch::<Map>();
        let pos = positions.get(*entity);
        match pos {
            Some(pos) => {
                idx = map.xy_idx(pos.x, pos.y);
                if !map.chill[idx] {
                    panic!(format!(
                        "Attempted to delete chill but no chill in position {} {}.",
                        pos.x, pos.y
                    ))
                }
            }
            None => panic!("Attempted to delete chill, but chill has no position.")
        }
    }
    { // Contain second borrow of ECS.
        let mut map = ecs.fetch_mut::<Map>();
        map.chill[idx] = false;
    }
    ecs.delete_entity(*entity).expect("Unable to remove chill entity.");
}


// A steam entity. Represents hot water vapor that damages any entity
// encroaching, and also spreads and dissipates. All spawning of steam MUST use
// this function, since it handles syncronizing the map.chill array.
const STEAM_ENCROACHMENT_DAMAGE: i32 = 2;

pub fn steam(ecs: &mut World, x: i32, y: i32, spread_chance: i32, dissipate_chance: i32) -> Option<Entity> {
    let can_spawn: bool;
    let idx: usize;
    {
        let map = ecs.fetch::<Map>();
        idx = map.xy_idx(x, y);
        can_spawn = !map.steam[idx] && map.tiles[idx] != TileType::Wall;
    }
    let entity;
    if can_spawn {
        entity = ecs.create_entity()
            .with(Position {x, y})
            .with(Renderable {
                fg: RGB::named(rltk::WHITE),
                bg: RGB::named(rltk::GRAY),
                glyph: rltk::to_cp437('░'),
                order: 2,
                visible_out_of_fov: false
            })
            .with(SetsBgColor {order: 0})
            .with(Name {name: "Steam".to_string()})
            .with(Hazard {})
            .with(IsEntityKind {
                kind: EntitySpawnKind::Steam {
                    spread_chance, dissipate_chance
                }
            })
            .with(ChanceToSpawnAdjacentEntity {
                chance: spread_chance,
                kind: EntitySpawnKind::Steam {
                    spread_chance: i32::max(0, spread_chance - 40),
                    dissipate_chance: i32::max(0, dissipate_chance + 10),
                }
            })
            .with(ChanceToDissipate {
                chance: dissipate_chance
            })
            .with(InflictsDamageWhenEncroachedUpon {
                damage: STEAM_ENCROACHMENT_DAMAGE,
                kind: ElementalDamageKind::Physical
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        map.steam[idx] = true;
        Some(entity)
    } else {
        None
    }
}

pub fn destroy_steam(ecs: &mut World, entity: &Entity) {
    let idx;
    { // Contain first borrow of ECS.
        let positions = ecs.read_storage::<Position>();
        let map = ecs.fetch::<Map>();
        let pos = positions.get(*entity);
        match pos {
            Some(pos) => {
                idx = map.xy_idx(pos.x, pos.y);
                if !map.steam[idx] {
                    panic!(format!(
                        "Attempted to delete steam but no steam in position {} {}.",
                        pos.x, pos.y
                    ))
                }
            }
            None => panic!("Attempted to delete steam, but steam has no position.")
        }
    }
    { // Contain second borrow of ECS.
        let mut map = ecs.fetch_mut::<Map>();
        map.steam[idx] = false;
    }
    ecs.delete_entity(*entity).expect("Unable to remove steam entity.");
}