
use super::{
    Map, TileType, EntitySpawnKind, BlocksTile, CombatStats, HungerClock,
    HungerState, Monster, Hazard, IsEntityKind, MonsterBasicAI, MonsterAttackSpellcasterAI,
    MovementRoutingOptions, Name, Player, Position, Renderable, Viewshed,
    PickUpable, Useable, Castable, SpellCharges, Equippable, EquipmentSlot,
    Throwable, Targeted, TargetingKind, Untargeted, Consumable,
    ProvidesFullHealing, ProvidesFullFood, IncreasesMaxHpWhenUsed,
    InflictsDamageWhenTargeted, InflictsDamageWhenEncroachedUpon,
    InflictsFreezingWhenTargeted, InflictsBurningWhenTargeted,
    InflictsBurningWhenEncroachedUpon, InflictsFreezingWhenEncroachedUpon,
    AreaOfEffectAnimationWhenTargeted, AlongRayAnimationWhenTargeted,
    MovesToRandomPosition, SpawnsEntityInAreaWhenTargeted,
    ChanceToSpawnAdjacentEntity, ChanceToDissipate, GrantsMeleeAttackBonus,
    GrantsMeleeDefenseBonus, ProvidesFireImmunityWhenUsed,
    ProvidesChillImmunityWhenUsed, SimpleMarker, SerializeMe, MarkedBuilder,
    ElementalDamageKind, InSpellBook,
    MAP_WIDTH, random_table
};
use rltk::{RandomNumberGenerator, RGB};
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

const DEFAULT_VIEW_RANGE: i32 = 8;
const DEAFULT_ROUTING_OPTIONS: MovementRoutingOptions = MovementRoutingOptions {
    avoid_blocked: true,
    avoid_fire: true,
    avoid_chill: true
};
const DEFAULT_MOVEMENT_AI: MonsterBasicAI = MonsterBasicAI {
    only_follow_within_viewshed: true,
    no_visibility_wander: true,
    lost_visibility_keep_following_turns_max: 2,
    lost_visibility_keep_following_turns_remaining: 2,
    routing_options: MovementRoutingOptions {..DEAFULT_ROUTING_OPTIONS}
};
const DEFAULT_COMBAT_STATS: CombatStats = CombatStats {
    max_hp: 15,
    hp: 15,
    defense: 1,
    power: 3,
};

// Spawn a generic monster.
fn spawn_monster<S: ToString>(ecs: &mut World, data: MonsterSpawnData<S>) -> Option<Entity> {
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

// Individual monster types: Basic Goblin.
fn goblin_basic(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let goblin = spawn_monster(
        ecs,
        MonsterSpawnData {
            x: x,
            y: y,
            name: "Goblin",
            glyph: rltk::to_cp437('g'),
            color: RGB::named(rltk::CORAL),
            view_range: DEFAULT_VIEW_RANGE,
            movement_ai: MonsterBasicAI {
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
    goblin
}

// Individual monster types: Goblin firecaster.
// A goblin spellcaster wielding basic fire magic.
fn goblin_firecaster(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
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
                    ..DEAFULT_ROUTING_OPTIONS
                }
            })
            .with(CombatStats {..DEFAULT_COMBAT_STATS})
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
fn goblin_chillcaster(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
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
                    ..DEAFULT_ROUTING_OPTIONS
                }
            })
            .with(CombatStats {..DEFAULT_COMBAT_STATS})
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

// Individual monster types: Orc.
fn orc_basic(ecs: &mut World, x: i32, y: i32) -> Option<Entity> {
    let orc = spawn_monster(
        ecs,
        MonsterSpawnData {
            x: x,
            y: y,
            name: "Orc",
            glyph: rltk::to_cp437('O'),
            color: RGB::named(rltk::YELLOW_GREEN),
            view_range: DEFAULT_VIEW_RANGE,
            movement_ai: MonsterBasicAI {
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
    orc
}