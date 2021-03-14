use specs::prelude::*;
use rltk::{Point, RandomNumberGenerator};
use super::{ChanceToDissipate, WantsToDissipate, EntitySpawnKind, IsEntityKind, destroy_fire};


pub struct DissipationSystem {}

// Look for entities that may dissipate and determine if they do.
impl<'a> System<'a> for DissipationSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadStorage<'a, ChanceToDissipate>,
        WriteStorage<'a, WantsToDissipate>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut rng, chance_to_dissipate, mut wants_to_dissipate)  = data;
        for (entity, chance) in (&entities, &chance_to_dissipate).join() {
            let does_dissipate = rng.roll_dice(1, 100) <= chance.chance;
            if does_dissipate {
                wants_to_dissipate
                    .insert(entity, WantsToDissipate {})
                    .expect("Unable to queue dissipation for entitiy.");
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
                (_, None) => {
                    ecs.delete_entity(victim)
                        .expect("Unable to dissipate an entitiy that requested it.");
                }
                (victim, Some(is_entity_kind)) => {
                    match is_entity_kind.kind {
                        EntitySpawnKind::Fire => destroy_fire(ecs, &victim),
                        _ => {}
                    }
                }
            }
        }
    }
}