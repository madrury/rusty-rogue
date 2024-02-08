
use super::{
    Map, BlocksTile, CombatStats, Monster, MovementRoutingAvoids,
    MonsterBasicAI, MonsterAttackSpellcasterAI, MonsterSupportSpellcasterAI,
    SupportSpellcasterKind, SpawnEntityWhenMeleeAttacked,  EntitySpawnKind,
    Name, Position, Renderable, Viewshed, SimpleMarker, SerializeMe,
    MarkedBuilder, StatusIsImmuneToFire, StatusIsImmuneToChill, InSpellBook,
    CanNotAct, BlessingSlot, spells
};
use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;


//----------------------------------------------------------------------------
// Rat.
//
// A dork of the very early dungeon. Low HP, and runs when damaged.
//----------------------------------------------------------------------------
const RAT_VIEW_RANGE: i32 = 4;
const RAT_BASE_HP: i32 = 6;
const RAT_BASE_DEFENSE: i32 = 0;
const RAT_BASE_POWER: i32 = 2;

const RAT_ROUTING_AVOIDS: MovementRoutingAvoids = MovementRoutingAvoids {
    blocked: true,
    fire: true,
    chill: true,
    water: true,
    steam: true,
    smoke: true,
};

const RAT_BASIC_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    chance_to_move_to_random_adjacent_tile: 0,
    escape_when_at_low_health: true,
    lost_visibility_keep_following_turns_max: 8,
    lost_visibility_keep_following_turns_remaining: 8,
};

pub fn rat(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let rat = ecs.create_entity()
        .with(Position {
            x: x,
            y: y
        })
        .with(Monster {})
        .with(Renderable {
            glyph: rltk::to_cp437('r'),
            fg: RGB::named(rltk::CHOCOLATE),
            bg: RGB::named(rltk::BLACK),
            order: 1,
            visible_out_of_fov: false
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: RAT_VIEW_RANGE,
            dirty: true,
        })
        .with(Name {
            name: "Rat".to_string(),
        })
        .with(MonsterBasicAI {
            ..RAT_BASIC_AI
        })
        .with(MovementRoutingAvoids {
            ..RAT_ROUTING_AVOIDS
        })
        .with(CombatStats {
            max_hp: RAT_BASE_HP,
            hp: RAT_BASE_HP,
            power: RAT_BASE_POWER,
            defense: RAT_BASE_DEFENSE
        })
        .with(BlocksTile {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    Some(rat)
}


//----------------------------------------------------------------------------
// Bat.
//
// A slightly more dangerous dork. Moves erratically, and can fly over some
// hazards.
//----------------------------------------------------------------------------
const BAT_VIEW_RANGE: i32 = 12;
const BAT_BASE_HP: i32 = 12;
const BAT_BASE_DEFENSE: i32 = 0;
const BAT_BASE_POWER: i32 = 2;

const BAT_ROUTING_AVOIDS: MovementRoutingAvoids = MovementRoutingAvoids {
    blocked: true,
    fire: true,
    chill: true,
    water: false,
    steam: true,
    smoke: true,
};

const BAT_BASIC_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    chance_to_move_to_random_adjacent_tile: 50,
    escape_when_at_low_health: false,
    lost_visibility_keep_following_turns_max: 2,
    lost_visibility_keep_following_turns_remaining: 2,
};

pub fn bat(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let bat = ecs.create_entity()
        .with(Position {
            x: x,
            y: y
        })
        .with(Monster {})
        .with(Renderable {
            glyph: rltk::to_cp437('v'),
            fg: RGB::named(rltk::GRAY),
            bg: RGB::named(rltk::BLACK),
            order: 1,
            visible_out_of_fov: false
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: BAT_VIEW_RANGE,
            dirty: true,
        })
        .with(Name {
            name: "Bat".to_string(),
        })
        .with(MonsterBasicAI {
            ..BAT_BASIC_AI
        })
        .with(MovementRoutingAvoids {
            ..BAT_ROUTING_AVOIDS
        })
        .with(CombatStats {
            max_hp: BAT_BASE_HP,
            hp: BAT_BASE_HP,
            power: BAT_BASE_POWER,
            defense: BAT_BASE_DEFENSE
        })
        .with(BlocksTile {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    Some(bat)
}


//----------------------------------------------------------------------------
// Goblins.
//
// The generaic antagonists of the lower dungeon. Once the player is seen, they
// will stupidly attack untill killed, never fleeing. The basic goblin comes in
// other flavors, with either enhanced attacks or the ability to assist their
// friends.
//----------------------------------------------------------------------------
const GOBLIN_VIEW_RANGE: i32 = 8;
const GOBLIN_BASE_HP: i32 = 15;
const GOBLIN_SPELLCASTER_HP: i32 = 8;
const GOBLIN_BASE_DEFENSE: i32 = 0;
const GOBLIN_BASE_POWER: i32 = 3;
const GOLBIN_ATTTACK_SPELLCASTER_DISTANCE: i32 = 3;
const GOBLIN_SUPPORT_SPELLCASTER_ALLY_DISTANCE: i32 = 2;
const GOBLIN_SUPPORT_SPELLCASTER_PLAYER_DISTANCE: i32 = 2;

const GOBLIN_ROUTING_AVOIDS: MovementRoutingAvoids = MovementRoutingAvoids {
    blocked: true,
    fire: true,
    chill: true,
    water: true,
    steam: true,
    smoke: true,
};

const GOBLIN_BASIC_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    chance_to_move_to_random_adjacent_tile: 0,
    escape_when_at_low_health: false,
    lost_visibility_keep_following_turns_max: 2,
    lost_visibility_keep_following_turns_remaining: 2,
};

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
        .with(MovementRoutingAvoids {
            ..GOBLIN_ROUTING_AVOIDS
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
                fg: RGB::named(rltk::ORANGERED),
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
            .with(MovementRoutingAvoids {
                ..GOBLIN_ROUTING_AVOIDS
            })
            .with(CombatStats {
                max_hp: GOBLIN_SPELLCASTER_HP,
                hp: GOBLIN_SPELLCASTER_HP,
                power: GOBLIN_BASE_POWER,
                defense: GOBLIN_BASE_DEFENSE
            })
            .with(StatusIsImmuneToFire {
                remaining_turns: i32::MAX,
                render_glyph: true
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
        in_spellbooks.insert(spell, InSpellBook {
            owner: goblin, slot: BlessingSlot::FireAttackLevel1
        }).expect("Failed to insert spell in goblin's spellbook.");
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
                fg: RGB::named(rltk::DODGERBLUE),
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
                name: "Goblin Chillcaster".to_string(),
            })
            .with(MonsterAttackSpellcasterAI {
                distance_to_keep_away: GOLBIN_ATTTACK_SPELLCASTER_DISTANCE,
            })
            .with(MovementRoutingAvoids {
                ..GOBLIN_ROUTING_AVOIDS
            })
            .with(CombatStats {
                max_hp: GOBLIN_SPELLCASTER_HP,
                hp: GOBLIN_SPELLCASTER_HP,
                power: GOBLIN_BASE_POWER,
                defense: GOBLIN_BASE_DEFENSE
            })
            .with(StatusIsImmuneToChill {
                remaining_turns: i32::MAX, render_glyph: true
            })
            .with(BlocksTile {})
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        let mut map = ecs.fetch_mut::<Map>();
        let idx = map.xy_idx(x, y);
        map.blocked[idx] = true;
    }
    {
        // Make an icespike spell and put it in the goblin's spellbook.
        let spell = spells::icespike(ecs, 0, 0, 2, 1)
            .expect("Could not construct icespike spell to put in spellbook.");
        let mut in_spellbooks = ecs.write_storage::<InSpellBook>();
        in_spellbooks.insert(spell, InSpellBook {
            owner: goblin, slot: BlessingSlot::ChillAttackLevel1
        }).expect("Failed to insert spell in goblin's spellbook.");
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
            .with(MovementRoutingAvoids {
                ..GOBLIN_ROUTING_AVOIDS
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
        // Make a healing spell and put it in the goblin's spellbook.
        let spell = spells::health(ecs, 0, 0, 2, 1)
            .expect("Could not construct healing spell to put in spellbook.");
        let mut in_spellbooks = ecs.write_storage::<InSpellBook>();
        in_spellbooks.insert(spell, InSpellBook {
            owner: goblin, slot: BlessingSlot::Assist
        }).expect("Failed to insert spell in goblin's spellbook.");
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
                fg: RGB::named(rltk::MEDIUM_PURPLE),
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
            .with(MovementRoutingAvoids {
                ..GOBLIN_ROUTING_AVOIDS
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
        in_spellbooks.insert(spell, InSpellBook {
            owner: goblin, slot: BlessingSlot::Assist
        }).expect("Failed to insert spell in goblin's spellbook.");
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

const ORC_ROUTING_AVOIDS: MovementRoutingAvoids = MovementRoutingAvoids {
    blocked: true,
    fire: true,
    chill: true,
    water: true,
    steam: true,
    smoke: true,
};

const ORC_BASIC_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    chance_to_move_to_random_adjacent_tile: 0,
    escape_when_at_low_health: false,
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
        .with(MovementRoutingAvoids {
            ..ORC_ROUTING_AVOIDS
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


//----------------------------------------------------------------------------
// Jellies.
//
// Gellatinous foes that spawn other monsters/hazards when melee attacked.
//----------------------------------------------------------------------------
const JELLY_VIEW_RANGE: i32 = 8;
pub const JELLY_BASE_HP: i32 = 20;
const JELLY_BASE_DEFENSE: i32 = 0;
const JELLY_BASE_POWER: i32 = 2;

const JELLY_ROUTING_AVOIDS: MovementRoutingAvoids = MovementRoutingAvoids {
    blocked: true,
    fire: true,
    chill: true,
    water: true,
    steam: true,
    smoke: true,
};

const JELLY_BASIC_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    chance_to_move_to_random_adjacent_tile: 0,
    escape_when_at_low_health: false,
    lost_visibility_keep_following_turns_max: 4,
    lost_visibility_keep_following_turns_remaining: 4,
};

pub fn pink_jelly(ecs: &mut World, x: i32, y: i32, max_hp:i32, hp: i32) -> Option<Entity> {
    let jelly = ecs.create_entity()
        .with(Position {
            x: x,
            y: y
        })
        .with(Monster {})
        .with(Renderable {
            glyph: rltk::to_cp437('J'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            order: 1,
            visible_out_of_fov: false
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: JELLY_VIEW_RANGE,
            dirty: true,
        })
        .with(Name {
            name: "Pink Jelly".to_string(),
        })
        .with(MonsterBasicAI {
            ..JELLY_BASIC_AI
        })
        .with(MovementRoutingAvoids {
            ..JELLY_ROUTING_AVOIDS
        })
        .with(CombatStats {
            max_hp: max_hp,
            hp: hp,
            power: JELLY_BASE_POWER,
            defense: JELLY_BASE_DEFENSE
        })
        .with(SpawnEntityWhenMeleeAttacked {
            kind: EntitySpawnKind::PinkJelly {
                // These are just placeholder values. We will overwrite them
                // with the appropriate hp values when the actual spawn request
                // is generated in melee_combat_system.
                max_hp: 0, hp: 0
            }
        })
        // We prevent pink jelly's from acting on their first turn. If they move
        // immediately, it's hard to tell what actually happened, this improves
        // their gameplay behaviour.
        .with(CanNotAct {})
        .with(BlocksTile {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    Some(jelly)
}

pub fn orange_jelly(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let jelly = ecs.create_entity()
        .with(Position {
            x: x,
            y: y
        })
        .with(Monster {})
        .with(BlocksTile {})
        .with(Renderable {
            glyph: rltk::to_cp437('J'),
            fg: RGB::named(rltk::ORANGERED),
            bg: RGB::named(rltk::BLACK),
            order: 1,
            visible_out_of_fov: false
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: JELLY_VIEW_RANGE,
            dirty: true,
        })
        .with(Name {
            name: "Orange Jelly".to_string(),
        })
        .with(MonsterBasicAI {
            ..JELLY_BASIC_AI
        })
        .with(MovementRoutingAvoids {
            fire: false,
            ..JELLY_ROUTING_AVOIDS
        })
        .with(CombatStats {
            max_hp: JELLY_BASE_HP,
            hp: JELLY_BASE_HP,
            power: JELLY_BASE_POWER,
            defense: JELLY_BASE_DEFENSE
        })
        .with(SpawnEntityWhenMeleeAttacked {
            kind: EntitySpawnKind::Fire {
                spread_chance: 25,
                dissipate_chance: 50,
            }
        })
        .with(StatusIsImmuneToFire {
            remaining_turns: i32::MAX, render_glyph: true
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    Some(jelly)
}

pub fn blue_jelly(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let jelly = ecs.create_entity()
        .with(Position {
            x: x,
            y: y
        })
        .with(Monster {})
        .with(BlocksTile {})
        .with(Renderable {
            glyph: rltk::to_cp437('J'),
            fg: RGB::named(rltk::DODGERBLUE),
            bg: RGB::named(rltk::BLACK),
            order: 1,
            visible_out_of_fov: false
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: JELLY_VIEW_RANGE,
            dirty: true,
        })
        .with(Name {
            name: "Blue Jelly".to_string(),
        })
        .with(MonsterBasicAI {
            ..JELLY_BASIC_AI
        })
        .with(MovementRoutingAvoids {
            chill: false,
            ..JELLY_ROUTING_AVOIDS
        })
        .with(CombatStats {
            max_hp: JELLY_BASE_HP,
            hp: JELLY_BASE_HP,
            power: JELLY_BASE_POWER,
            defense: JELLY_BASE_DEFENSE
        })
        .with(SpawnEntityWhenMeleeAttacked {
            kind: EntitySpawnKind::Chill {
                spread_chance: 25,
                dissipate_chance: 50,
            }
        })
        .with(StatusIsImmuneToChill {
            remaining_turns: i32::MAX,
            render_glyph: true
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    let mut map = ecs.fetch_mut::<Map>();
    let idx = map.xy_idx(x, y);
    map.blocked[idx] = true;
    Some(jelly)
}