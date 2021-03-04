
use specs::prelude::*;
use rltk::{RGB};
use super::{GameLog, Name, Monster, StatusIsFrozen, StatusIsBurning, ApplyDamage};


pub struct MonsterStatusSystem {}

// A system to update and apply effects of status ailments/buffs on monsters.
impl<'a> System<'a> for MonsterStatusSystem {

    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, StatusIsFrozen>,
        WriteStorage<'a, StatusIsBurning>,
        WriteStorage<'a, ApplyDamage>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            monsters,
            names,
            mut status_frozen,
            mut status_burning,
            mut damages
        ) = data;

        for (entity, _monster) in (&entities, &monsters).join() {

            // StatusIsFrozen: Tick frozen entities and remove the status if
            // expired.
            let frozen = status_frozen.get_mut(entity);
            if let Some(frozen) = frozen {
                if frozen.remaining_turns <= 0 {
                    status_frozen.remove(entity);
                    log.entries.push(
                        format!("{} is np longer frozen.", names.get(entity).unwrap().name)
                    )
                } else {
                    frozen.tick();
                }
            }

            // StatusIsBurning: Tick burning entities, apply the tick damage,
            // and remove the status if expired.
            let burning = status_burning.get_mut(entity);
            if let Some(burning) = burning {
                if burning.remaining_turns <= 0 {
                    status_burning.remove(entity);
                    log.entries.push(
                        format!("{}'s blaze is extinguished.", names.get(entity).unwrap().name)
                    )
                } else {
                    ApplyDamage::new_damage(&mut damages, entity, burning.tick_damage);
                    burning.tick();
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
    indicators
}
