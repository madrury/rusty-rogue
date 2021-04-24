
use specs::prelude::*;
use rltk::{RGB, RandomNumberGenerator};
use super::{
    Map, GameLog, Name, OfferedBlessing, WantsToDissipate, entity_spawners
};

pub fn create_offered_blessings(ecs: &mut World) {
    let mut offers: Vec<Entity> = Vec::new();
    let offer = entity_spawners::spells::magic_missile(ecs, 0, 0, 5, 2);
    match offer {
        Some(offer) => {
            let mut offered_blessings = ecs.write_storage::<OfferedBlessing>();
            offered_blessings.insert(offer, OfferedBlessing {})
                .expect("Could not insert offered blessing component.");
            offers.push(offer);
        }
        None => {}
    };
    let offer = entity_spawners::spells::fireball(ecs, 0, 0, 5, 2);
    match offer {
        Some(offer) => {
            let mut offered_blessings = ecs.write_storage::<OfferedBlessing>();
            offered_blessings.insert(offer, OfferedBlessing {})
                .expect("Could not insert offered blessing component.");
            offers.push(offer);
        }
        None => {}
    };
    let offer = entity_spawners::spells::icespike(ecs, 0, 0, 5, 2);
    match offer {
        Some(offer) => {
            let mut offered_blessings = ecs.write_storage::<OfferedBlessing>();
            offered_blessings.insert(offer, OfferedBlessing {})
                .expect("Could not insert offered blessing component.");
            offers.push(offer);
        }
        None => {}
    };
    ecs.maintain();
}

pub fn clean_up_offered_blessings(ecs: &mut World) {
    let entities = ecs.entities();
    let offereds = ecs.read_storage::<OfferedBlessing>();
    let mut wants_dissipate = ecs.write_storage::<WantsToDissipate>();
    for (entity, _offer) in (&entities, &offereds).join() {
        wants_dissipate.insert(entity, WantsToDissipate {})
            .expect("Could not insert WantsToDissipate when cleaning up offered blessings.");
    }
}