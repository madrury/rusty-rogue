use specs::prelude::*;

use crate::StatusEffect;

use super::{
    Map, Name, GameLog, Position, Tramples, EntitySpawnRequest,
    EntitySpawnRequestBuffer, InflictsDamageWhenEncroachedUpon,
    InflictsBurningWhenEncroachedUpon, InflictsFreezingWhenEncroachedUpon,
    DissipateWhenEnchroachedUpon, SpawnEntityWhenEncroachedUpon,
    RemoveBurningWhenEncroachedUpon, DissipateFireWhenEncroachedUpon,
    DissipateWhenTrampledUpon, WantsToTakeDamage, StatusIsBurning,
    StatusIsFrozen, StatusIsImmuneToFire, StatusIsImmuneToChill,
    WantsToDissipate, IsEntityKind, EntitySpawnKind,
    InvisibleWhenEncroachingEntityKind, StatusInvisibleToPlayer,
    SpawnEntityWhenTrampledUpon, new_status, new_status_with_immunity,
    remove_status
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
    tramples: ReadStorage<'a, Tramples>,
    damage_when_encroached: ReadStorage<'a, InflictsDamageWhenEncroachedUpon>,
    burning_when_encroached: ReadStorage<'a, InflictsBurningWhenEncroachedUpon>,
    freezing_when_encroached: ReadStorage<'a, InflictsFreezingWhenEncroachedUpon>,
    dissipate_when_encroached: ReadStorage<'a, DissipateWhenEnchroachedUpon>,
    dissipate_when_trampled: ReadStorage<'a, DissipateWhenTrampledUpon>,
    spawn_when_encroached: ReadStorage<'a, SpawnEntityWhenEncroachedUpon>,
    spawn_when_trampled: ReadStorage<'a, SpawnEntityWhenTrampledUpon>,
    remove_burning_when_encroached: ReadStorage<'a, RemoveBurningWhenEncroachedUpon>,
    dissipate_fire_when_encroached: ReadStorage<'a, DissipateFireWhenEncroachedUpon>,
    invisible_when_encroaching: ReadStorage<'a, InvisibleWhenEncroachingEntityKind>,
    status_invisible: WriteStorage<'a, StatusInvisibleToPlayer>,
    is_fire_immune: WriteStorage<'a, StatusIsImmuneToFire>,
    is_chill_immune: WriteStorage<'a, StatusIsImmuneToChill>,
    wants_damage: WriteStorage<'a, WantsToTakeDamage>,
    wants_dissipate: WriteStorage<'a, WantsToDissipate>,
    is_burning: WriteStorage<'a, StatusIsBurning>,
    is_frozen: WriteStorage<'a, StatusIsFrozen>,
    entity_kind: ReadStorage<'a, IsEntityKind>,
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
            tramples,
            damage_when_encroached,
            burning_when_encroached,
            freezing_when_encroached,
            dissipate_when_encroached,
            dissipate_when_trampled,
            spawn_when_encroached,
            spawn_when_trampled,
            remove_burning_when_encroached,
            dissipate_fire_when_encroached,
            invisible_when_encroaching,
            mut status_invisible,
            is_fire_immune,
            is_chill_immune,
            mut wants_damage,
            mut wants_dissipate,
            mut is_burning,
            mut is_frozen,
            entity_kind
        } = data;

        for (entity, pos) in (&entities, &positions).join() {

            let idx = map.xy_idx(pos.x, pos.y);

            for encroaching in map.tile_content[idx].iter().filter(|e| **e != entity) {

                let encroaching_does_trample = tramples.get(*encroaching).is_some();

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
                        true
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
                        true
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

                // Component: DissipatesWhenTrampledUpon.
                let dissipate = dissipate_when_trampled.get(entity).is_some();
                if dissipate && encroaching_does_trample {
                    wants_dissipate.insert(entity, WantsToDissipate {})
                        .expect("Could not insert wants to dissipate upon encroachement.");
                }

                // Component: SpawnEntityWhenEncroachedUpon.
                let spawn = spawn_when_encroached.get(entity);
                if let Some(spawn) = spawn {
                    spawn_buffer.request(EntitySpawnRequest {
                        x: pos.x,
                        y: pos.y,
                        kind: spawn.kind
                    })
                }

                // Component: SpawnEntityWhenTrampledUpon.
                let spawn = spawn_when_trampled.get(entity);
                if let Some(spawn) = spawn {
                    if encroaching_does_trample {
                        spawn_buffer.request(EntitySpawnRequest {
                            x: pos.x,
                            y: pos.y,
                            kind: spawn.kind
                        })
                    }
                }

                // RemoveBurningWhenEncroachedUpon.
                let removes_burning = remove_burning_when_encroached.get(entity);
                if let Some(_) = removes_burning {
                    remove_status::<StatusIsBurning>(
                        &mut is_burning,
                        *encroaching
                    )
                }

                // DissipateFireWhenEncroachedUpon.
                let dissipates_fire = dissipate_fire_when_encroached.get(entity).is_some();
                let encroaching_is_fire = entity_kind.get(*encroaching)
                    .map_or(false, |k| matches!(k.kind, EntitySpawnKind::Fire {..}));
                if encroaching_is_fire && dissipates_fire {
                    wants_dissipate.insert(*encroaching, WantsToDissipate {})
                        .expect("Could not insert wants to dissipate on fire entity.");
                }

                // InvisibleWhenEncroachingEntityKind
                let is_tall_grass = entity_kind.get(entity)
                    .map_or(false, |k| matches!(k.kind, EntitySpawnKind::TallGrass {..}));
                let encroaching_is_hidden = invisible_when_encroaching.get(*encroaching).is_some();
                if is_tall_grass && encroaching_is_hidden {
                    new_status::<StatusInvisibleToPlayer>(&mut status_invisible, *encroaching, 1, false);
                }

            } // for encroaching
        } // for (entity, pos)
    }
}