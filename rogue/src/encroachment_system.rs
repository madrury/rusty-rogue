use specs::prelude::*;
use super::{
    Map, Position, InflictsDamageWhenEncroachedUpon,
    InflictsBurningWhenEncroachedUpon, InflictsFreezingWhenEncroachedUpon,
    WantsToTakeDamage, StatusIsBurning, StatusIsFrozen, StatusIsImmuneToFire
};


pub struct EncroachmentSystem {}

#[derive(SystemData)]
pub struct EncroachmentSystemData<'a> {
    entities: Entities<'a>,
    map: ReadExpect<'a, Map>,
    positions: ReadStorage<'a, Position>,
    damage_when_encroached: ReadStorage<'a, InflictsDamageWhenEncroachedUpon>,
    burning_when_encroached: ReadStorage<'a, InflictsBurningWhenEncroachedUpon>,
    freezing_when_encroached: ReadStorage<'a, InflictsFreezingWhenEncroachedUpon>,
    is_fire_immune: ReadStorage<'a, StatusIsImmuneToFire>,
    // is_chill_immune: ReadStorage<'a, StatusIsImmuneToChill>,
    wants_damage: WriteStorage<'a, WantsToTakeDamage>,
    is_burning: WriteStorage<'a, StatusIsBurning>,
    is_frozen: WriteStorage<'a, StatusIsFrozen>,
}

impl<'a> System<'a> for EncroachmentSystem {
    type SystemData = EncroachmentSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let EncroachmentSystemData {
            entities,
            map,
            positions,
            damage_when_encroached,
            burning_when_encroached,
            freezing_when_encroached,
            is_fire_immune,
            mut wants_damage,
            mut is_burning,
            mut is_frozen
        } = data;

        for (entity, pos) in (&entities, &positions).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for encroaching in map.tile_content[idx].iter().filter(|e| **e != entity) {

                // Component: InflictsDamageWhenEncroachedUpon.
                let dmg = damage_when_encroached.get(entity);
                if let Some(dmg) = dmg {
                    WantsToTakeDamage::new_damage(
                        &mut wants_damage,
                        *encroaching,
                        dmg.damage,
                        dmg.kind
                    );
                }
                // Component: InflictsBurningWhenEncroachedUpon.
                let burning = burning_when_encroached.get(entity);
                let encroaching_is_immune = is_fire_immune.get(*encroaching).is_some();
                if let Some(burning) = burning {
                    if !encroaching_is_immune {
                        StatusIsBurning::new_status(
                            &mut is_burning,
                            *encroaching,
                            burning.turns,
                            burning.tick_damage
                        )
                    }
                }
                // Component: InflictsFreezingWhenEncroachedUpon.
                let freezing = freezing_when_encroached.get(entity);
                // let encroaching_is_immune = is_fire_immune.get(*encroaching).is_some();
                let encroaching_is_immune = false;
                if let Some(freezing) = freezing {
                    if !encroaching_is_immune {
                        StatusIsFrozen::new_status(
                            &mut is_frozen,
                            *encroaching,
                            freezing.turns,
                        )
                    }
                }
            }
        }
    }
}