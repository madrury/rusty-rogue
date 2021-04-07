
use specs::prelude::*;
use rltk::{RGB, RandomNumberGenerator};
use super::{
    Map, GameLog, Name, RunState, Monster, Hazard, Position, StatusIsFrozen,
    StatusIsBurning, StatusIsImmuneToFire, StatusIsImmuneToChill,
    StatusIsMeleeAttackBuffed, StatusIsPhysicalDefenseBuffed,
    WantsToTakeDamage, WantsToDissipate, ElementalDamageKind,
    DissipateWhenBurning, ChanceToSpawnEntityWhenBurning,
    ChanceToInflictBurningOnAdjacentEntities, EntitySpawnRequestBuffer,
    EntitySpawnRequest, tick_status, tick_status_with_immunity,
    new_status_with_immunity, BURNING_TICK_DAMAGE
};

//----------------------------------------------------------------------------
// Tick all status effect counteres and removee any that have expired.
//----------------------------------------------------------------------------
pub struct StatusTickSystem {}

#[derive(SystemData)]
pub struct StatusTickSystemData<'a> {
    entities: Entities<'a>,
    log: WriteExpect<'a, GameLog>,
    names: ReadStorage<'a, Name>,
    status_frozen: WriteStorage<'a, StatusIsFrozen>,
    status_burning: WriteStorage<'a, StatusIsBurning>,
    status_immune_fire: WriteStorage<'a, StatusIsImmuneToFire>,
    status_immune_chill: WriteStorage<'a, StatusIsImmuneToChill>,
    status_attack_buffed: WriteStorage<'a, StatusIsMeleeAttackBuffed>,
    status_defense_buffed: WriteStorage<'a, StatusIsPhysicalDefenseBuffed>,
}

impl<'a> System<'a> for StatusTickSystem {

    type SystemData = StatusTickSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let StatusTickSystemData {
            entities,
            mut log,
            names,
            mut status_frozen,
            mut status_burning,
            mut status_immune_fire,
            mut status_immune_chill,
            mut status_attack_buffed,
            mut status_defense_buffed
        } = data;

        for entity in entities.join() {
            // StatusIsBurning: Tick burning entities, and remove the status if
            // expired or if the entity has aquired fire immunity.
            let msg = names.get(entity)
                .map(|nm| format!("{} is no longer burning.", nm.name));
            tick_status_with_immunity::<StatusIsBurning, StatusIsImmuneToFire>(
                &mut status_burning,
                &mut status_immune_fire,
                &mut log,
                entity,
                msg
            );
            // StatusIsFrozen: Tick frozen entities and remove the status if
            // expired.
            let msg = names.get(entity)
                .map(|nm| format!("{} is no longer frozen.", nm.name));
            tick_status_with_immunity::<StatusIsFrozen, StatusIsImmuneToChill>(
                &mut status_frozen,
                &mut status_immune_chill,
                &mut log,
                entity,
                msg
            );
            // StatusIsImmuneToFire: Tick fire immune entities and remove the
            // status if expired.
            let msg = names.get(entity)
                .map(|nm| format!("{} is no longer immune to flames.", nm.name));
            tick_status::<StatusIsImmuneToFire>(
                &mut status_immune_fire,
                &mut log,
                entity,
                msg
            );
            // StatusIsImmuneToChill: Tick chill entities and remove the
            // status if expired.
            let msg = names.get(entity)
                .map(|nm| format!("{} is no longer immune to chill.", nm.name));
            tick_status::<StatusIsImmuneToChill>(
                &mut status_immune_chill,
                &mut log,
                entity,
                msg
            );
            // StatusIsMeleeAttackBuffed
            let msg = names.get(entity)
                .map(|nm| format!("{} is no longer feeling invogroated.", nm.name));
                tick_status::<StatusIsMeleeAttackBuffed>(
                &mut status_attack_buffed,
                &mut log,
                entity,
                msg
            );
            // StatusIsPhysicalDefenseBuffed
            let msg = names.get(entity)
                .map(|nm| format!("{} is no longer feeling protected.", nm.name));
                tick_status::<StatusIsPhysicalDefenseBuffed>(
                &mut status_defense_buffed,
                &mut log,
                entity,
                msg
            );
        }
    }
}


//----------------------------------------------------------------------------
// Apply all status effects that happen once every game loop.
//----------------------------------------------------------------------------
pub struct StatusEffectSystem {}

#[derive(SystemData)]
pub struct StatusEffectSystemData<'a> {
    entities: Entities<'a>,
    runstate: ReadExpect<'a, RunState>,
    map: ReadExpect<'a, Map>,
    rng: WriteExpect<'a, RandomNumberGenerator>,
    spawn_buffer: WriteExpect<'a, EntitySpawnRequestBuffer>,
    player: ReadExpect<'a, Entity>,
    monsters: ReadStorage<'a, Monster>,
    hazards: ReadStorage<'a, Hazard>,
    positions: ReadStorage<'a, Position>,
    names: ReadStorage<'a, Name>,
    status_burning: WriteStorage<'a, StatusIsBurning>,
    status_immune_fire: WriteStorage<'a, StatusIsImmuneToFire>,
    dissipate_when_burning: ReadStorage<'a, DissipateWhenBurning>,
    chance_to_spawn_when_burning: ReadStorage<'a, ChanceToSpawnEntityWhenBurning>,
    chance_to_inflict_burning_on_adjacent:
       ReadStorage<'a, ChanceToInflictBurningOnAdjacentEntities>,
    wants_damages: WriteStorage<'a, WantsToTakeDamage>,
    wants_dissipates: WriteStorage<'a, WantsToDissipate>,

}

impl<'a> System<'a> for StatusEffectSystem {

    type SystemData = StatusEffectSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let StatusEffectSystemData {
            entities,
            runstate,
            map,
            mut rng,
            mut spawn_buffer,
            player,
            monsters,
            hazards,
            positions,
            names,
            mut status_burning,
            status_immune_fire,
            chance_to_spawn_when_burning,
            chance_to_inflict_burning_on_adjacent,
            dissipate_when_burning,
            mut wants_damages,
            mut wants_dissipates
        } = data;

        for entity in entities.join() {

            // Only apply an entity's statuses on the appropriate turn.
            let is_player = entity == *player;
            let is_monster = monsters.get(entity).is_some();
            let is_hazard = hazards.get(entity).is_some();
            let proceed =
                (*runstate == RunState::PlayerTurn && is_player)
                || (*runstate == RunState::MonsterTurn && is_monster)
                || (*runstate == RunState::HazardTurn && is_hazard);
            if !proceed {
                continue
            }

            // StatusIsBurning: Tick burning entities, apply the tick damage,
            // and remove the status if expired or if the entity has aquired
            // fire immunity.
            let burning = status_burning.get_mut(entity);
            let pos = positions.get(entity);
            let chance_to_spawn = chance_to_spawn_when_burning.get(entity);
            let is_fire_immune = status_immune_fire.get(entity).is_some();
            let does_dissipate_when_burning = dissipate_when_burning.get(entity).is_some();
            if let Some(_burning) = burning {
                // Entities with combat stats (in this case, with hp, so
                // succeptable to damage) take tick damage when burning.
                // Note: It's ok to add a WantsToTakeDamage component to
                // entities without any combat stats component, it is ignored in
                // damage_system.
                if !is_fire_immune {
                    WantsToTakeDamage::new_damage(
                        &mut wants_damages,
                        entity,
                        BURNING_TICK_DAMAGE,
                        ElementalDamageKind::Fire
                    );
                }
                // Some entities (i.e. grass) are destroyed when burning.
                if does_dissipate_when_burning {
                    wants_dissipates.insert(entity, WantsToDissipate {})
                        .expect("Unable to insert WantsToDissipate.");
                }
                // Some entities spawn other entities (usually fire) when
                // burning. For example, grass spawn fire in its space when it
                // is burned.
                if let (Some(chance_to_spawn), Some(pos)) = (chance_to_spawn, pos) {
                    let roll = rng.roll_dice(1, 100);
                    if roll <= chance_to_spawn.chance {
                        spawn_buffer.request(EntitySpawnRequest {
                            x: pos.x,
                            y: pos.y,
                            kind: chance_to_spawn.kind
                        })
                    }
                }
            }

            // ChanceToInflictBurningOnAdjacentEntities.
            // Grab all entities in a tile adjaent to this one, roll a die, and
            // maybe burn them.
            let chance_to_spread_burning = chance_to_inflict_burning_on_adjacent.get(entity);
            if let (Some(spread_chance), Some(pos)) = (chance_to_spread_burning, pos) {
                let adjacent = get_adjacent_entities(&*map, pos);
                for entity in adjacent {
                    let roll = rng.roll_dice(1, 100);
                    if roll <= spread_chance.chance {
                        new_status_with_immunity::<StatusIsBurning, StatusIsImmuneToFire>(
                            &mut status_burning,
                            &status_immune_fire,
                            *entity,
                            5
                        );
                    }
                }
            }
        }
    }
}

//----------------------------------------------------------------------------
// Status Glyphs.
// Helper utilities to construct glyphs indicating status effects.
//   - Burning: ♠
//   - Frozen:  ♦
//   - FireImmunity: ♠ (WHITE)
//----------------------------------------------------------------------------
pub struct StatusIndicatorGlyph {
    pub glyph: rltk::FontCharType,
    pub color: RGB
}

// Retrns a vector of StatusIdicatorGlyphs for all status effects currently
// affecting a given entity.
pub fn get_status_indicators(ecs: &World, entity: &Entity) -> Vec<StatusIndicatorGlyph> {
    let mut indicators = Vec::new();

    let frozens = ecs.read_storage::<StatusIsFrozen>();
    if let Some(_) = frozens.get(*entity) {
        indicators.push(
            StatusIndicatorGlyph {glyph: rltk::to_cp437('♦'), color: RGB::named(rltk::LIGHT_BLUE)}
        )
    }
    let burnings = ecs.read_storage::<StatusIsBurning>();
    if let Some(_) = burnings.get(*entity) {
        indicators.push(
            StatusIndicatorGlyph {glyph: rltk::to_cp437('♠'), color: RGB::named(rltk::ORANGE)}
        )
    }
    let chill_immunities = ecs.read_storage::<StatusIsImmuneToChill>();
    if let Some(_) = chill_immunities.get(*entity) {
        indicators.push(
            StatusIndicatorGlyph {glyph: rltk::to_cp437('♦'), color: RGB::named(rltk::WHITE)}
        )
    }
    let fire_immunities = ecs.read_storage::<StatusIsImmuneToFire>();
    if let Some(_) = fire_immunities.get(*entity) {
        indicators.push(
            StatusIndicatorGlyph {glyph: rltk::to_cp437('♠'), color: RGB::named(rltk::WHITE)}
        )
    }
    let attack_buffed = ecs.read_storage::<StatusIsMeleeAttackBuffed>();
    if let Some(_) = attack_buffed.get(*entity) {
        indicators.push(
            StatusIndicatorGlyph {glyph: rltk::to_cp437('▲'), color: RGB::named(rltk::RED)}
        )
    }
    let defense_buffed = ecs.read_storage::<StatusIsPhysicalDefenseBuffed>();
    if let Some(_) = defense_buffed.get(*entity) {
        indicators.push(
            StatusIndicatorGlyph {glyph: rltk::to_cp437('▲'), color: RGB::named(rltk::BLUE)}
        )
    }
    indicators
}


fn get_adjacent_entities<'a>(map: &'a Map, pos: &Position) -> Vec<&'a Entity> {
    let mut entities: Vec<&Entity> = Vec::new();
    let adjacent_tiles = map.get_adjacent_tiles(pos.x, pos.y);
    for (x, y) in adjacent_tiles {
        let idx = map.xy_idx(x, y);
        for e in map.tile_content[idx].iter() {
            entities.push(&e);
        }
    }
    entities
}