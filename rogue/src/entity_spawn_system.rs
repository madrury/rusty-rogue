use super::{
    World, RunState, Map, Monster, Hazard, EntitySpawnKind, Position,
    ChanceToSpawnAdjacentEntity, entity_spawners, terrain_spawners
};
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
        Entities<'a>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Hazard>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteExpect<'a, EntitySpawnRequestBuffer>,
        ReadStorage<'a, ChanceToSpawnAdjacentEntity>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {

        let (
            entities,
            map,
            runstate,
            player,
            monsters,
            hazards,
            mut rng,
            mut request_buffer,
            chance_to_spawn,
            positions
        ) = data;


        // Component: ChanceToSpawnAdjacentEntity
        for (entity, spawn_chance, pos) in (&entities, &chance_to_spawn, &positions).join() {

            // Turn guard.
            let is_player = entity == *player;
            let is_monster = monsters.get(entity).is_some();
            let is_hazard = hazards.get(entity).is_some();
            let proceed =
                (*runstate == RunState::PlayerTurn && is_player)
                || (*runstate == RunState::MonsterTurn && is_monster)
                || (*runstate == RunState::HazardTurn && is_hazard);
            if !proceed {
                continue
            }

            let do_spawn = rng.roll_dice(1, 100) <= spawn_chance.chance;
            if do_spawn {
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
            EntitySpawnKind::MagicOrb =>
                entity_spawners::magic::blessing_orb(ecs, request.x, request.y),
            EntitySpawnKind::Fire {spread_chance, dissipate_chance} =>
                entity_spawners::hazards::fire(ecs, request.x, request.y, spread_chance, dissipate_chance),
            EntitySpawnKind::Chill {spread_chance, dissipate_chance} =>
                entity_spawners::hazards::chill(ecs, request.x, request.y, spread_chance, dissipate_chance),
            EntitySpawnKind::Steam {spread_chance, dissipate_chance} =>
                entity_spawners::hazards::steam(ecs, request.x, request.y, spread_chance, dissipate_chance),
            EntitySpawnKind::Grass {fg} =>
                terrain_spawners::foliage::grass(ecs, request.x, request.y, fg),
            EntitySpawnKind::PinkJelly {max_hp, hp} =>
                entity_spawners::monsters::pink_jelly(ecs, request.x, request.y, max_hp, hp),
            // TODO: We'll eventually have ways to spawn water.
            _ => {None}
        };
    }
    let mut spawn_buffer = ecs.fetch_mut::<EntitySpawnRequestBuffer>();
    spawn_buffer.requests.clear();
}