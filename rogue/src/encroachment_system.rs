use specs::prelude::*;
use super::{Map, Position, InflictsDamageWhenEncroachedUpon, WantsToTakeDamage};


pub struct EncroachmentSystem {}

#[derive(SystemData)]
pub struct EncroachmentSystemData<'a> {
    entities: Entities<'a>,
    map: ReadExpect<'a, Map>,
    positions: ReadStorage<'a, Position>,
    damage_when_encroached: ReadStorage<'a, InflictsDamageWhenEncroachedUpon>,
    wants_damage: WriteStorage<'a, WantsToTakeDamage>
}

impl<'a> System<'a> for EncroachmentSystem {
    type SystemData = EncroachmentSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let EncroachmentSystemData {
            entities,
            map,
            positions,
            damage_when_encroached,
            mut wants_damage
        } = data;

        for (entity, dmg, pos) in (&entities, &damage_when_encroached, &positions).join() {
            // Search for entities in the same tile.
            let idx = map.xy_idx(pos.x, pos.y);
            // let encroaching = &map.tile_content[idx];
            for encroaching in map.tile_content[idx].iter() {
                if *encroaching == entity {continue;}
                WantsToTakeDamage::new_damage(&mut wants_damage, *encroaching, dmg.damage);
            }
        }
    }
}