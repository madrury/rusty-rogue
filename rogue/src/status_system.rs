
use specs::prelude::*;
use rltk::{RGB};
use super::{
    GameLog, Name, RunState, Monster, Hazard, StatusIsFrozen,
    StatusIsBurning, StatusIsImmuneToFire, StatusIsImmuneToChill,
    WantsToTakeDamage, ElementalDamageKind, tick_status,
    tick_status_with_immunity, BURNING_TICK_DAMAGE
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
    player: ReadExpect<'a, Entity>,
    monsters: ReadStorage<'a, Monster>,
    hazards: ReadStorage<'a, Hazard>,
    status_burning: WriteStorage<'a, StatusIsBurning>,
    status_immune_fire: WriteStorage<'a, StatusIsImmuneToFire>,
    wants_damages: WriteStorage<'a, WantsToTakeDamage>
}

impl<'a> System<'a> for StatusEffectSystem {

    type SystemData = StatusEffectSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let StatusEffectSystemData {
            entities,
            runstate,
            player,
            monsters,
            hazards,
            mut status_burning,
            status_immune_fire,
            mut wants_damages
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
            let is_fire_immune = status_immune_fire.get(entity).is_some();
            if let Some(_burning) = burning {
                if !is_fire_immune {
                    WantsToTakeDamage::new_damage(
                        &mut wants_damages,
                        entity,
                        BURNING_TICK_DAMAGE,
                        ElementalDamageKind::Fire
                    );
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
    indicators
}
