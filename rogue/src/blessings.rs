
use specs::prelude::*;
use rltk::{RGB, RandomNumberGenerator};
use super::{
    Map, GameLog, Name, BlessingOrbBag, OfferedBlessing,
    BlessingSelectionTile, Position,
    Point, WantsToDissipate, entity_spawners
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

// Tag any remaining offered blessings as wanting to dissipate, which will cause
// them to be cleaned up the next time the dissipation system runs.
pub fn clean_up_offered_blessings(ecs: &mut World) {
    let entities = ecs.entities();
    let offereds = ecs.read_storage::<OfferedBlessing>();
    let mut wants_dissipate = ecs.write_storage::<WantsToDissipate>();
    for (entity, _offer) in (&entities, &offereds).join() {
        wants_dissipate.insert(entity, WantsToDissipate {})
            .expect("Could not insert WantsToDissipate when cleaning up offered blessings.");
    }
}

// Check if the player is standing on a belssing tile.
pub fn is_player_encroaching_blessing_tile(ecs: &World) -> bool {
    let entities = ecs.entities();
    let blessing_tiles = ecs.read_storage::<BlessingSelectionTile>();
    let positions = ecs.read_storage::<Position>();
    let blessing_locations: Vec<Point> = (&entities, &blessing_tiles, &positions).join()
        .map(|(_t, _tiles, pos)| Point {x: pos.x, y: pos.y})
        .collect();
    let player_point = ecs.fetch::<Point>();
    blessing_locations.contains(&player_point)
}

pub fn does_player_have_sufficient_orbs_for_blessing(ecs: &World) -> bool {
    let player = ecs.fetch::<Entity>();
    let orb_bags = ecs.read_storage::<BlessingOrbBag>();
    let player_orb_bag = orb_bags.get(*player).expect("Player has no BlessingOrbBag.");
    player_orb_bag.enough_orbs_for_blessing()
}

pub fn cash_in_orbs_for_blessing(ecs: &World) {
    let player = ecs.fetch::<Entity>();
    let mut orb_bags = ecs.write_storage::<BlessingOrbBag>();
    let mut player_orb_bag = orb_bags.get_mut(*player).expect("Player has no BlessingOrbBag.");
    player_orb_bag.cash_in_orbs_for_blessing();
}