
use super::{
    Map, TileType, EntitySpawnKind, Hazard, IsEntityKind, Name, Position,
    Renderable, InflictsDamageWhenEncroachedUpon,
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
pub fn fire(ecs: &mut World, x: i32, y: i32, spread_chance: i32, dissipate_chance: i32) -> Option<Entity> {
    let can_spawn: bool;
    let idx: usize;
    {
        let map = ecs.fetch::<Map>();
        idx = map.xy_idx(x, y);
        can_spawn = !map.fire[idx] && map.tiles[idx] != TileType::Wall;
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
                    spread_chance: i32::max(0, spread_chance - 20),
                    dissipate_chance: i32::max(0, dissipate_chance + 20),
                }
            })
            .with(ChanceToInflictBurningOnAdjacentEntities {
                chance: 50
            })
            .with(ChanceToDissipate {
                chance: dissipate_chance
            })
            .with(InflictsDamageWhenEncroachedUpon {
                damage: 2,
                kind: ElementalDamageKind::Fire
            })
            .with(InflictsBurningWhenEncroachedUpon {
                turns: 4, tick_damage: 2
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
                    spread_chance: i32::max(0, spread_chance - 10),
                    dissipate_chance: i32::max(0, dissipate_chance + 40),
                }
            })
            .with(ChanceToDissipate {
                chance: dissipate_chance
            })
            .with(InflictsDamageWhenEncroachedUpon {
                damage: 2,
                kind: ElementalDamageKind::Chill
            })
            .with(InflictsFreezingWhenEncroachedUpon {
                turns: 2
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