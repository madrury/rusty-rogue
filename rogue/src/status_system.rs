
use specs::prelude::*;
use rltk::{RGB};
use super::{
    GameLog, Name, RunState, Monster, StatusIsFrozen, StatusIsBurning,
    StatusIsImmuneToFire, WantsToTakeDamage, ElementalDamageKind
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
        } = data;

        for entity in entities.join() {
            // StatusIsBurning: Tick burning entities, and remove the status if
            // expired or if the entity has aquired fire immunity.
            let burning = status_burning.get_mut(entity);
            let is_fire_immune = status_immune_fire.get(entity).is_some();
            if let Some(burning) = burning {
                if burning.remaining_turns <= 0 || is_fire_immune {
                    status_burning.remove(entity);
                    let name = names.get(entity);
                    if let Some(name) = name {
                        log.entries.push(
                            format!("{} is np longer burning.", name.name)
                        )
                    }
                } else {
                    burning.tick();
                }
            }
            // StatusIsFrozen: Tick frozen entities and remove the status if
            // expired.
            let frozen = status_frozen.get_mut(entity);
            if let Some(frozen) = frozen {
                if frozen.remaining_turns <= 0 {
                    status_frozen.remove(entity);
                    let name = names.get(entity);
                    if let Some(name) = name {
                        log.entries.push(
                            format!("{} is no longer frozen.", name.name)
                        )
                    }
                } else {
                    frozen.tick();
                }
            }
            // StatusIsImmuneToFire: Tick fire immune entities and remove the
            // status if expired.
            let is_fire_immune = status_immune_fire.get_mut(entity);
            if let Some(immune) = is_fire_immune {
                if immune.remaining_turns <= 0 {
                    status_immune_fire.remove(entity);
                    let name = names.get(entity);
                    if let Some(name) = name {
                        log.entries.push(
                            format!("{} is no longer immune to flames.", name.name)
                        )
                    }
                } else {
                    immune.tick();
                }
            }
        }
    }
}


pub struct StatusEffectSystem {}

#[derive(SystemData)]
pub struct StatusEffectSystemData<'a> {
    entities: Entities<'a>,
    runstate: ReadExpect<'a, RunState>,
    player: ReadExpect<'a, Entity>,
    monsters: ReadStorage<'a, Monster>,
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
            mut status_burning,
            status_immune_fire,
            mut wants_damages
        } = data;

        for entity in entities.join() {

            // Only apply an entities statuses on the appropriate turn.
            let is_player = entity == *player;
            let is_monster = monsters.get(entity).is_some();
            let proceed =
                (*runstate == RunState::PlayerTurn && is_player)
                || (*runstate == RunState::MonsterTurn && is_monster);
            if !proceed {
                return
            }

            // StatusIsBurning: Tick burning entities, apply the tick damage,
            // and remove the status if expired or if the entity has aquired
            // fire immunity.
            let burning = status_burning.get_mut(entity);
            let is_fire_immune = status_immune_fire.get(entity).is_some();
            if let Some(burning) = burning {
                if !is_fire_immune {
                    WantsToTakeDamage::new_damage(
                        &mut wants_damages,
                        entity,
                        burning.tick_damage,
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
    let fire_immunities = ecs.read_storage::<StatusIsImmuneToFire>();
    if let Some(_) = fire_immunities.get(*entity) {
        indicators.push(
            StatusIndicatorGlyph {glyph: rltk::to_cp437('♠'), color: RGB::named(rltk::WHITE)}
        )
    }
    indicators
}
