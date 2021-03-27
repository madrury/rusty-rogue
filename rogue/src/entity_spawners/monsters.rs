
use super::{
    Map, BlocksTile, CombatStats, Monster, MonsterBasicAI,
    MonsterAttackSpellcasterAI, MonsterClericAI, MovementRoutingOptions,
    Name, Position, Renderable, Viewshed, SimpleMarker, SerializeMe,
    MarkedBuilder, InSpellBook, spells
};
use rltk::{RGB};
use specs::prelude::*;

struct MonsterSpawnData<S: ToString> {
    x: i32,
    y: i32,
    name: S,
    glyph: rltk::FontCharType,
    color: RGB,
    view_range: i32,
    movement_ai: MonsterBasicAI,
    combat_stats: CombatStats,
}

const GOBLIN_VIEW_RANGE: i32 = 8;
const GOBLIN_ROUTING_OPTIONS: MovementRoutingOptions = MovementRoutingOptions {
    avoid_blocked: true,
    avoid_fire: true,
    avoid_chill: true
};
const GOBLIN_MOVEMENT_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    lost_visibility_keep_following_turns_max: 2,
    lost_visibility_keep_following_turns_remaining: 2,
    routing_options: MovementRoutingOptions {..GOBLIN_ROUTING_OPTIONS}
};
const GOBLIN_COMBAT_STATS: CombatStats = CombatStats {
    max_hp: 15,
    hp: 15,
    defense: 0,
    power: 3,
};

// Spawn a generic monster.
fn spawn_monster<S: ToString>(
    ecs: &mut World,
    data: MonsterSpawnData<S>
) -> Option<Entity> {
    let entity = ecs.create_entity()
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
        Some(entity)
}

//----------------------------------------------------------------------------
// Goblins.
//
// The generaic antagonists of the lower dungeon. Once the player is seen, they
// will stupidly attack untill killed, never fleeing. The basic goblin comes in
// other flavors, with either enhanced attacks or the ability to assist their
// friends.
//----------------------------------------------------------------------------
// Individual monster types: Basic Goblin.
pub fn goblin_basic(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let goblin = spawn_monster(
        ecs,
        MonsterSpawnData {
            x: x,
            y: y,
            name: "Goblin",
            glyph: rltk::to_cp437('g'),
            color: RGB::named(rltk::BROWN1),
            view_range: GOBLIN_VIEW_RANGE,
            movement_ai: MonsterBasicAI {
                ..GOBLIN_MOVEMENT_AI
            },
            combat_stats: CombatStats {
                ..GOBLIN_COMBAT_STATS
            },
        },
    );
    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    goblin
}

// Individual monster types: Goblin firecaster.
// A goblin spellcaster wielding basic fire magic.
pub fn goblin_firecaster(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let goblin;
    {
        goblin = ecs.create_entity()
            .with(Position {
                x: x,
                y: y
            })
            .with(Monster {})
            .with(Renderable {
                glyph: rltk::to_cp437('g'),
                fg: RGB::named(rltk::ORANGE),
                bg: RGB::named(rltk::BLACK),
                order: 1
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Name {
                name: "Goblin Firecaster".to_string(),
            })
            .with(MonsterAttackSpellcasterAI {
                distance_to_keep_away: 4,
                routing_options: MovementRoutingOptions {
                    ..GOBLIN_ROUTING_OPTIONS
                }
            })
            .with(CombatStats {..GOBLIN_COMBAT_STATS})
            .with(BlocksTile {})
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        let idx = map.xy_idx(x, y);
        map.blocked[idx] = true;
    }
    {
        // Make a fireball spell and put it in the goblin's spellbook.
        let spell = spells::fireball(ecs, 0, 0, 2, 1)
            .expect("Could not construct fireball spell to put in spellbook.");
        let mut in_spellbooks = ecs.write_storage::<InSpellBook>();
        in_spellbooks.insert(spell, InSpellBook {owner: goblin})
            .expect("Failed to insert fireball spell in goblin's spellbook.");
        let mut positions = ecs.write_storage::<Position>();
        positions.remove(spell);
    }
    Some(goblin)
}

// Individual monster types: Goblin chillcaster.
// A goblin spellcaster wielding basic chill magic.
pub fn goblin_chillcaster(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let goblin;
    {
        goblin = ecs.create_entity()
            .with(Position {
                x: x,
                y: y
            })
            .with(Monster {})
            .with(Renderable {
                glyph: rltk::to_cp437('g'),
                fg: RGB::named(rltk::LIGHT_BLUE),
                bg: RGB::named(rltk::BLACK),
                order: 1
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Name {
                name: "Goblin Firecaster".to_string(),
            })
            .with(MonsterAttackSpellcasterAI {
                distance_to_keep_away: 4,
                routing_options: MovementRoutingOptions {
                    ..GOBLIN_ROUTING_OPTIONS
                }
            })
            .with(CombatStats {..GOBLIN_COMBAT_STATS})
            .with(BlocksTile {})
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        let idx = map.xy_idx(x, y);
        map.blocked[idx] = true;
    }
    {
        // Make a fireball spell and put it in the goblin's spellbook.
        let spell = spells::icespike(ecs, 0, 0, 2, 1)
            .expect("Could not construct fireball spell to put in spellbook.");
        let mut in_spellbooks = ecs.write_storage::<InSpellBook>();
        in_spellbooks.insert(spell, InSpellBook {owner: goblin})
            .expect("Failed to insert fireball spell in goblin's spellbook.");
        let mut positions = ecs.write_storage::<Position>();
        positions.remove(spell);
    }
    Some(goblin)
}

// Individual monster types: Goblin chillcaster.
// A goblin spellcaster wielding basic chill magic.
pub fn goblin_cleric(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let goblin;
    {
        goblin = ecs.create_entity()
            .with(Position {
                x: x,
                y: y
            })
            .with(Monster {})
            .with(Renderable {
                glyph: rltk::to_cp437('g'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
                order: 1
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 4,
                dirty: true,
            })
            .with(Name {
                name: "Goblin Cleric".to_string(),
            })
            .with(MonsterClericAI {
                distance_to_keep_away: 2,
                routing_options: MovementRoutingOptions {
                    ..GOBLIN_ROUTING_OPTIONS
                }
            })
            .with(CombatStats {..GOBLIN_COMBAT_STATS})
            .with(BlocksTile {})
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        let idx = map.xy_idx(x, y);
        map.blocked[idx] = true;
    }
    {
        // Make a fireball spell and put it in the goblin's spellbook.
        let spell = spells::health(ecs, 0, 0, 2, 1)
            .expect("Could not construct healing spell to put in spellbook.");
        let mut in_spellbooks = ecs.write_storage::<InSpellBook>();
        in_spellbooks.insert(spell, InSpellBook {owner: goblin})
            .expect("Failed to insert healing spell in goblin's spellbook.");
        let mut positions = ecs.write_storage::<Position>();
        positions.remove(spell);
    }
    Some(goblin)
}
//----------------------------------------------------------------------------
// Orcs.
//
// Basically a beefier goblin.
//----------------------------------------------------------------------------
const ORC_VIEW_RANGE: i32 = 8;
const ORC_ROUTING_OPTIONS: MovementRoutingOptions = MovementRoutingOptions {
    avoid_blocked: true,
    avoid_fire: true,
    avoid_chill: true
};
const ORC_MOVEMENT_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    lost_visibility_keep_following_turns_max: 5,
    lost_visibility_keep_following_turns_remaining: 5,
    routing_options: MovementRoutingOptions {..ORC_ROUTING_OPTIONS}
};
const ORC_COMBAT_STATS: CombatStats = CombatStats {
    max_hp: 30,
    hp: 30,
    defense: 0,
    power: 5,
};

pub fn orc_basic(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let orc = spawn_monster(
        ecs,
        MonsterSpawnData {
            x: x,
            y: y,
            name: "Orc",
            glyph: rltk::to_cp437('O'),
            color: RGB::named(rltk::YELLOW_GREEN),
            view_range: ORC_VIEW_RANGE,
            movement_ai: MonsterBasicAI {
                ..ORC_MOVEMENT_AI
            },
            combat_stats: CombatStats {
                max_hp: 20,
                hp: 20,
                power: 5,
                ..ORC_COMBAT_STATS
            },
        },
    );
    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    orc
}