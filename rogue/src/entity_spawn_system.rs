use super::{World, Map, EntitySpawnKind, fire};
use rltk::{Point};
use specs::prelude::*;



// A request to spawn a single entity of a specified type.
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
        // spawn_new_entity(ecs, request.kind, request.x, request.y);
        match request.kind {
            EntitySpawnKind::Fire => fire(ecs, request.x, request.y),
            _ => {}
        }
    }
    let mut spawn_buffer = ecs.fetch_mut::<EntitySpawnRequestBuffer>();
    spawn_buffer.requests.clear();
}