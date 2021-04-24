
use specs::prelude::*;
use rltk::{RGB, RandomNumberGenerator};
use super::{
    Map, GameLog, Name, OfferedBlessing, entity_spawners
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

pub fn get_offered_blessings(ecs: &mut World) -> Vec<Entity> {
    let mut offers: Vec<Entity> = Vec::new();
    offers
}