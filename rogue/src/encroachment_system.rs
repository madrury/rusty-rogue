use specs::prelude::*;
use rltk::RandomNumberGenerator;
use super::{
    Map, Name, GameLog, Position, EntitySpawnRequest,
    EntitySpawnRequestBuffer, InflictsDamageWhenEncroachedUpon,
    InflictsBurningWhenEncroachedUpon, InflictsFreezingWhenEncroachedUpon,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    WantsToTakeDamage, StatusIsBurning, StatusIsFrozen, StatusIsImmuneToFire,
    StatusIsImmuneToChill, WantsToDissipate,
    new_status_with_immunity
};


pub struct EncroachmentSystem {}

#[derive(SystemData)]
pub struct EncroachmentSystemData<'a> {
    entities: Entities<'a>,
    map: ReadExpect<'a, Map>,
    log: WriteExpect<'a, GameLog>,
    spawn_buffer: WriteExpect<'a, EntitySpawnRequestBuffer>,
    positions: ReadStorage<'a, Position>,
    names: ReadStorage<'a, Name>,
    damage_when_encroached: ReadStorage<'a, InflictsDamageWhenEncroachedUpon>,
    burning_when_encroached: ReadStorage<'a, InflictsBurningWhenEncroachedUpon>,
    freezing_when_encroached: ReadStorage<'a, InflictsFreezingWhenEncroachedUpon>,
    dissipate_when_encroached: ReadStorage<'a, DissipateWhenEnchroachedUpon>,
    spawn_when_encroached: ReadStorage<'a, SpawnEntityWhenEncroachedUpon>,
    is_fire_immune: WriteStorage<'a, StatusIsImmuneToFire>,
    is_chill_immune: WriteStorage<'a, StatusIsImmuneToChill>,
    wants_damage: WriteStorage<'a, WantsToTakeDamage>,
    wants_dissipate: WriteStorage<'a, WantsToDissipate>,
    is_burning: WriteStorage<'a, StatusIsBurning>,
    is_frozen: WriteStorage<'a, StatusIsFrozen>,
}

impl<'a> System<'a> for EncroachmentSystem {
    type SystemData = EncroachmentSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let EncroachmentSystemData {
            entities,
            map,
            mut log,
            mut spawn_buffer,
            positions,
            names,
            damage_when_encroached,
            burning_when_encroached,
            freezing_when_encroached,
            dissipate_when_encroached,
            spawn_when_encroached,
            is_fire_immune,
            is_chill_immune,
            mut wants_damage,
            mut wants_dissipate,
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
                if let Some(burning) = burning {
                    let play_message = new_status_with_immunity::<StatusIsBurning, StatusIsImmuneToFire>(
                        &mut is_burning,
                        &is_fire_immune,
                        *encroaching,
                        burning.turns,
                    );
                    if play_message {
                        let burner_name = names.get(entity);
                        let target_name = names.get(*encroaching);
                        if let (Some(bnm), Some(tnm)) = (burner_name, target_name) {
                            log.entries.push(format!(
                                "{} encroaches on {}, and it set aflame.",
                                tnm.name,
                                bnm.name,
                            ))
                        }
                    }
                }

                // Component: InflictsFreezingWhenEncroachedUpon.
                let freezing = freezing_when_encroached.get(entity);
                if let Some(freezing) = freezing {
                    let play_message = new_status_with_immunity::<StatusIsFrozen, StatusIsImmuneToChill>(
                        &mut is_frozen,
                        &is_chill_immune,
                        *encroaching,
                        freezing.turns,
                    );
                    if play_message {
                        let freezer_name = names.get(entity);
                        let target_name = names.get(*encroaching);
                        if let (Some(fnm), Some(tnm)) = (freezer_name, target_name) {
                            log.entries.push(format!(
                                "{} encroaches on {}, and is frozen in place.",
                                tnm.name,
                                fnm.name,
                            ))
                        }
                    }
                }

                // Component: DissipatesWhenEncroachedUpon.
                let dissipate = dissipate_when_encroached.get(entity).is_some();
                if dissipate {
                    wants_dissipate.insert(entity, WantsToDissipate {})
                        .expect("Could not insert wants to dissipate upon encroachement.");
                }

                // SpawnEntityWhenEncroachedUpon.
                let spawn = spawn_when_encroached.get(entity);
                if let Some(spawn) = spawn {
                    spawn_buffer.request(EntitySpawnRequest {
                        x: pos.x,
                        y: pos.y,
                        kind: spawn.kind
                    })
                }
            }

        }
    }
}