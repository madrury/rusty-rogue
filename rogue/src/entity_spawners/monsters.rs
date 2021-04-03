
use super::{
    Map, BlocksTile, CombatStats, Monster, MonsterMovementRoutingOptions,
    MonsterBasicAI, MonsterAttackSpellcasterAI, MonsterSupportSpellcasterAI,
    SupportSpellcasterKind, MovementRoutingOptions, Name, Position,
    Renderable, Viewshed, SimpleMarker, SerializeMe, MarkedBuilder,
    InSpellBook, spells
};
use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;


const GOBLIN_VIEW_RANGE: i32 = 8;
const GOBLIN_BASE_HP: i32 = 15;
const GOBLIN_SPELLCASTER_HP: i32 = 8;
const GOBLIN_BASE_DEFENSE: i32 = 0;
const GOBLIN_BASE_POWER: i32 = 3;
const GOLBIN_ATTTACK_SPELLCASTER_DISTANCE: i32 = 3;
const GOBLIN_SUPPORT_SPELLCASTER_ALLY_DISTANCE: i32 = 2;
const GOBLIN_SUPPORT_SPELLCASTER_PLAYER_DISTANCE: i32 = 2;

const GOBLIN_ROUTING_OPTIONS: MovementRoutingOptions = MovementRoutingOptions {
    avoid_blocked: true,
    avoid_fire: true,
    avoid_chill: true,
    avoid_water: true,
    avoid_steam: true,
    avoid_smoke: true,
    avoid_lava: true,
    avoid_brimstone: true,
    avoid_ice: false,
};

const GOBLIN_BASIC_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    lost_visibility_keep_following_turns_max: 2,
    lost_visibility_keep_following_turns_remaining: 2,
};

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
    let goblin = ecs.create_entity()
        .with(Position {
            x: x,
            y: y
        })
        .with(Monster {})
        .with(Renderable {
            glyph: rltk::to_cp437('g'),
            fg: RGB::named(rltk::CHOCOLATE),
            bg: RGB::named(rltk::BLACK),
            order: 1,
            visible_out_of_fov: false
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: GOBLIN_VIEW_RANGE,
            dirty: true,
        })
        .with(Name {
            name: "Goblin".to_string(),
        })
        .with(MonsterBasicAI {
            ..GOBLIN_BASIC_AI
        })
        .with(MonsterMovementRoutingOptions {
            options: GOBLIN_ROUTING_OPTIONS
        })
        .with(CombatStats {
            max_hp: GOBLIN_BASE_HP,
            hp: GOBLIN_BASE_HP,
            power: GOBLIN_BASE_POWER,
            defense: GOBLIN_BASE_DEFENSE
        })
        .with(BlocksTile {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();


    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    Some(goblin)
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
                order: 1,
                visible_out_of_fov: false
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: GOBLIN_VIEW_RANGE,
                dirty: true,
            })
            .with(Name {
                name: "Goblin Firecaster".to_string(),
            })
            .with(MonsterAttackSpellcasterAI {
                distance_to_keep_away: GOLBIN_ATTTACK_SPELLCASTER_DISTANCE,
            })
            .with(MonsterMovementRoutingOptions {
                options: GOBLIN_ROUTING_OPTIONS
            })
            .with(CombatStats {
                max_hp: GOBLIN_SPELLCASTER_HP,
                hp: GOBLIN_SPELLCASTER_HP,
                power: GOBLIN_BASE_POWER,
                defense: GOBLIN_BASE_DEFENSE
            })
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

// Individual monster types: Goblin Chillcaster.
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
                order: 1,
                visible_out_of_fov: false
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
                distance_to_keep_away: GOLBIN_ATTTACK_SPELLCASTER_DISTANCE,
            })
            .with(MonsterMovementRoutingOptions {
                options: GOBLIN_ROUTING_OPTIONS
            })
            .with(CombatStats {
                max_hp: GOBLIN_SPELLCASTER_HP,
                hp: GOBLIN_SPELLCASTER_HP,
                power: GOBLIN_BASE_POWER,
                defense: GOBLIN_BASE_DEFENSE
            })
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

// Individual monster types: Goblin Cleric.
// A goblin spellcaster wielding basic healing magic.
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
                order: 1,
                visible_out_of_fov: false
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Name {
                name: "Goblin Cleric".to_string(),
            })
            .with(MonsterSupportSpellcasterAI {
                support_kind: SupportSpellcasterKind::Cleric,
                distance_to_keep_away_from_monsters: GOBLIN_SUPPORT_SPELLCASTER_ALLY_DISTANCE,
                distance_to_keep_away_from_player: GOBLIN_SUPPORT_SPELLCASTER_PLAYER_DISTANCE,
            })
            .with(MonsterMovementRoutingOptions {
                options: GOBLIN_ROUTING_OPTIONS
            })
            .with(CombatStats {
                max_hp: GOBLIN_SPELLCASTER_HP,
                hp: GOBLIN_SPELLCASTER_HP,
                power: GOBLIN_BASE_POWER,
                defense: GOBLIN_BASE_DEFENSE
            })
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

// Individual monster types: Goblin Enchanter.
// A goblin spellcaster wielding basic attack/defense buff magic.
pub fn goblin_enchanter(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
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
                fg: RGB::named(rltk::GREY),
                bg: RGB::named(rltk::BLACK),
                order: 1,
                visible_out_of_fov: false
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Name {
                name: "Goblin Enchanter".to_string(),
            })
            .with(MonsterMovementRoutingOptions {
                options: GOBLIN_ROUTING_OPTIONS
            })
            .with(CombatStats {
                max_hp: GOBLIN_SPELLCASTER_HP,
                hp: GOBLIN_SPELLCASTER_HP,
                power: GOBLIN_BASE_POWER,
                defense: GOBLIN_BASE_DEFENSE
            })
            .with(BlocksTile {})
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        let idx = map.xy_idx(x, y);
        map.blocked[idx] = true;
    }
    // There are two options for what buff spell we give to the monster here: do
    // they buff attack of defense?
    {
        let mut rng = RandomNumberGenerator::new();
        let roll = rng.roll_dice(1, 2);
        let spell: Entity;
        let spellcaster_kind: SupportSpellcasterKind;
        match roll {
            1 => {
                spellcaster_kind = SupportSpellcasterKind::EnchanterAttack;
                spell = spells::invigorate(ecs, 0, 0, 2, 1)
                    .expect("Could not construct invigorate spell to put in spellbook.");
            }
            2 => {
                spellcaster_kind = SupportSpellcasterKind::EnchanterDefense;
                spell = spells::protect(ecs, 0, 0, 2, 1)
                    .expect("Could not construct protect spell to put in spellbook.");
            }
            _ => {panic!("Got random number out of range, no associated enchantertype.")}
        }
        let mut ais = ecs.write_storage::<MonsterSupportSpellcasterAI>();
        ais.insert(goblin, MonsterSupportSpellcasterAI {
            support_kind: spellcaster_kind,
            distance_to_keep_away_from_monsters: GOBLIN_SUPPORT_SPELLCASTER_ALLY_DISTANCE,
            distance_to_keep_away_from_player: GOBLIN_SUPPORT_SPELLCASTER_PLAYER_DISTANCE,
        }).expect("Could not insert MonsterSupportSpellcasterAI.");
        let mut in_spellbooks = ecs.write_storage::<InSpellBook>();
        in_spellbooks.insert(spell, InSpellBook {owner: goblin})
            .expect("Failed to insert spell in goblin's spellbook.");
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
const ORC_BASE_HP: i32 = 30;
const ORC_BASE_POWER: i32 = 6;
const ORC_BASE_DEFENSE: i32 = 0;

const ORC_ROUTING_OPTIONS: MovementRoutingOptions = MovementRoutingOptions {
    avoid_blocked: true,
    avoid_fire: true,
    avoid_chill: true,
    avoid_water: true,
    avoid_steam: true,
    avoid_smoke: true,
    avoid_lava: true,
    avoid_brimstone: true,
    avoid_ice: false,
};
const ORC_BASIC_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    lost_visibility_keep_following_turns_max: 5,
    lost_visibility_keep_following_turns_remaining: 5,
};

pub fn orc_basic(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {

    let orc = ecs.create_entity()
        .with(Position {
            x: x,
            y: y
        })
        .with(Monster {})
        .with(Renderable {
            glyph: rltk::to_cp437('O'),
            fg: RGB::named(rltk::GREENYELLOW),
            bg: RGB::named(rltk::BLACK),
            order: 1,
            visible_out_of_fov: false
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: ORC_VIEW_RANGE,
            dirty: true,
        })
        .with(Name {
            name: "Orc".to_string(),
        })
        .with(MonsterBasicAI {
            ..ORC_BASIC_AI
        })
        .with(MonsterMovementRoutingOptions {
            options: ORC_ROUTING_OPTIONS
        })
        .with(CombatStats {
            max_hp: ORC_BASE_HP,
            hp: ORC_BASE_HP,
            power: ORC_BASE_POWER,
            defense: ORC_BASE_DEFENSE
        })
        .with(BlocksTile {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    Some(orc)
}