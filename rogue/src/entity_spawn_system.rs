use super::{World, Map, EntitySpawnKind, Position, ChanceToSpawnAdjacentEntity, fire};
use rltk::{RandomNumberGenerator};
use specs::prelude::*;


// A request to spawn a single entity of a specified kind.
#[derive(Clone)]
pub struct EntitySpawnRequest {
    pub x: i32,
    pub y: i32,
    pub kind: EntitySpawnKind
}

// A buffer for storing entity spawn requests.
pub struct EntitySpawnRequestBuffer {
    requests: Vec<EntitySpawnRequest>,
}
impl EntitySpawnRequestBuffer {
    pub fn new() -> EntitySpawnRequestBuffer {
        EntitySpawnRequestBuffer {
            requests: Vec::new(),
        }
    }
    pub fn request(&mut self, request: EntitySpawnRequest) {
        self.requests.push(request)
    }
}

// Checks for any components that generate entity spawn requests, determines
// their specific request, and then pushes that request to the
// EntitySpawnRequestBuffer.
pub struct EntitySpawnSystem {}

impl<'a> System<'a> for EntitySpawnSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteExpect<'a, EntitySpawnRequestBuffer>,
        ReadStorage<'a, ChanceToSpawnAdjacentEntity>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {

        let (map, mut rng, mut request_buffer, chance_to_spawn, positions) = data;

        // Component: ChanceToSpawnAdjacentEntity
        for (spawn_chance, pos) in (&chance_to_spawn, &positions).join() {
            let do_spawn = rng.roll_dice(1, 100) <= spawn_chance.chance;
            if do_spawn {
                // spawn_position = map.random_adjacent_point()
                let spawn_position = map.random_adjacent_point(pos.x, pos.y);
                if let Some(spawn_position) = spawn_position {
                    request_buffer.request(EntitySpawnRequest{
                        x: spawn_position.0,
                        y: spawn_position.1,
                        kind: spawn_chance.kind
                    })
                }
            }
        }
    }
}

// Loop through the EntitySpawnRequestBuffer and switch on the kind of entity
// being requested. Delegate the requests to the appropriate functions and then
// clear the buffer.
pub fn process_entity_spawn_request_buffer(ecs: &mut World) {
    // We need to clone the requests vector here to keep the borrow checker
    // happy. Otherwise we're keeping a mutable reference to part of the ecs,
    // but also passing in a mutable reference to the spawning functions.
    let spawn_requests;
    {
        let spawns = ecs.fetch::<EntitySpawnRequestBuffer>();
        spawn_requests = spawns.requests.clone();
    }
    for request in spawn_requests.iter() {
        // TODO: Check that the index is within bounds!
        match request.kind {
            EntitySpawnKind::Fire => fire(ecs, request.x, request.y),
            _ => {}
        }
    }
    let mut spawn_buffer = ecs.fetch_mut::<EntitySpawnRequestBuffer>();
    spawn_buffer.requests.clear();
}