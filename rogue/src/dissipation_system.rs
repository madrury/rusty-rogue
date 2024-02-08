use specs::prelude::*;
use rltk::{RandomNumberGenerator};

use super::{
    ChanceToDissipate, WantsToDissipate, EntitySpawnKind, IsEntityKind,
    SkipRandomDissipationForOneTurn, entity_spawners, terrain_spawners
};


pub struct DissipationSystem {}

// Look for entities that may dissipate and determine if they do.
impl<'a> System<'a> for DissipationSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadStorage<'a, ChanceToDissipate>,
        WriteStorage<'a, SkipRandomDissipationForOneTurn>,
        WriteStorage<'a, WantsToDissipate>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut rng, chance_to_dissipate, mut skip_dissipation, mut wants_to_dissipate)  = data;
        for (entity, chance) in (&entities, &chance_to_dissipate).join() {
            let do_skip = skip_dissipation.get(entity);
            match do_skip {
                Some(_) => {
                    skip_dissipation.remove(entity);
                }
                None => {
                    let does_dissipate = rng.roll_dice(1, 100) <= chance.chance;
                    if does_dissipate {
                        wants_to_dissipate
                            .insert(entity, WantsToDissipate {})
                            .expect("Unable to queue dissipation for entitiy.");
                    }
                }
            }
        }
    }
}

// Search for entities with a WantsToDissipate component, then delegate the
// dissipation to the appropriate destructor for that entity.
impl DissipationSystem {
    pub fn clean_up_dissipated_entities(ecs: &mut World) {
        // Holds the entities that want to dissipate, along with type
        // information that allows us to delegate to the appropriate destructor.
        let mut dissipated: Vec<(Entity, Option<IsEntityKind>)> = Vec::new();
        { // Scope to contain the lifetime of the immutable borrow of ecs in
            //the line below.
            let does_dissipate = ecs.read_storage::<WantsToDissipate>();
            let spawn_kind = ecs.read_storage::<IsEntityKind>();
            let entities = ecs.entities();
            for (entity, _does_dissipate) in (&entities, &does_dissipate).join() {
                let kind = spawn_kind.get(entity);
                dissipated.push((entity, kind.map(|k| *k)));
            }
        }
        // Actually call the apppropriate destructors.
        for (victim, kind) in dissipated {
            match (victim, kind) {
                // If the victim does not have a registered kind (type), then we
                // assume there is no custom destructor.
                (_, None) => {
                    ecs.delete_entity(victim)
                        .expect("Unable to dissipate an entitiy that requested it.");
                }
                // If the victim does have a registered kind, then we lookup the
                // approporiate destructor.
                (victim, Some(is_entity_kind)) => {
                    match is_entity_kind.kind {
                        EntitySpawnKind::Fire {..} => entity_spawners::hazards::destroy_fire(ecs, &victim),
                        EntitySpawnKind::Chill {..} => entity_spawners::hazards::destroy_chill(ecs, &victim),
                        EntitySpawnKind::Steam {..} => entity_spawners::hazards::destroy_steam(ecs, &victim),
                        EntitySpawnKind::ShortGrass {..} => terrain_spawners::foliage::destroy_grass(ecs, &victim),
                        EntitySpawnKind::TallGrass {..} => terrain_spawners::foliage::destroy_tall_grass(ecs, &victim),
                        // TODO: Implement destroy water.
                        _ => {}
                    }
                }
            }
        }
    }
}