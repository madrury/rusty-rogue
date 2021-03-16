
use specs::prelude::*;
use rltk::{RGB};
use super::{
    GameLog, Name, StatusIsFrozen, StatusIsBurning, StatusIsImmuneToFire,
    WantsToTakeDamage,
    ElementalDamageKind
};


pub struct StatusSystem {}

// A system to update and apply effects of status ailments/buffs on monsters.
impl<'a> System<'a> for StatusSystem {

    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, StatusIsFrozen>,
        WriteStorage<'a, StatusIsBurning>,
        WriteStorage<'a, StatusIsImmuneToFire>,
        WriteStorage<'a, WantsToTakeDamage>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            names,
            mut status_frozen,
            mut status_burning,
            mut status_is_fire_immune,
            mut damages
        ) = data;

        for entity in entities.join() {


            // StatusIsBurning: Tick burning entities, apply the tick damage,
            // and remove the status if expired or if the entity has aquired
            // fire immunity.
            let burning = status_burning.get_mut(entity);
            let is_fire_immune = status_is_fire_immune.get(entity).is_some();
            if let Some(burning) = burning {
                if burning.remaining_turns <= 0 || is_fire_immune {
                    status_burning.remove(entity);
                    let name = names.get(entity);
                    if let Some(name) = name {
                        log.entries.push(
                            format!("{} is np longer frozen.", name.name)
                        )
                    }
                } else {
                    WantsToTakeDamage::new_damage(
                        &mut damages,
                        entity,
                        burning.tick_damage,
                        ElementalDamageKind::Fire
                    );
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
            let is_fire_immune = status_is_fire_immune.get_mut(entity);
            if let Some(immune) = is_fire_immune {
                if immune.remaining_turns <= 0 {
                    status_is_fire_immune.remove(entity);
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
