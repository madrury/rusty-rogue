use super::{
    Map, Point, Position, Viewshed, Renderable, BlocksTile,
    MovementRoutingAvoids, MovementRoutingBounds, WantsToMoveToRandomPosition,
    WantsToMoveToPosition, AnimationRequestBuffer, AnimationRequest
};
use specs::prelude::*;
use rltk::RandomNumberGenerator;


pub struct TeleportationSystem {}

#[derive(SystemData)]
pub struct TeleportationSystemData<'a> {
    entities: Entities<'a>,
    map: ReadExpect<'a, Map>,
    rng: WriteExpect<'a, RandomNumberGenerator>,
    animation_builder: WriteExpect<'a, AnimationRequestBuffer>,
    positions: WriteStorage<'a, Position>,
    renderables: ReadStorage<'a, Renderable>,
    wants_to_move: WriteStorage<'a, WantsToMoveToPosition>,
    wants_to_teleport: WriteStorage<'a, WantsToMoveToRandomPosition>,
}

impl<'a> System<'a> for TeleportationSystem {

    type SystemData = TeleportationSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let TeleportationSystemData {
            entities,
            map,
            mut rng,
            mut animation_builder,
            mut positions,
            renderables,
            mut wants_to_move,
            mut wants_to_teleport,
        } = data;

        for (entity, pos, _wants_to_teleport) in (&entities, &mut positions, &wants_to_teleport).join() {

            let old_pos = pos.clone();

            let new_pt = map.random_unblocked_point(10, &mut *rng);
            if let Some(new_pt) = new_pt {
                wants_to_move.insert(entity, WantsToMoveToPosition {
                    // TODO: We should be finding a safe position to teleport to
                    // so this force is not needed.
                    pt: new_pt, force: true
                })
                    .expect("Could not insert WantsToMoveToPosition.");
                let render = renderables.get(entity);
                if let Some(render) = render {
                    animation_builder.request(AnimationRequest::Teleportation {
                        x: old_pos.x,
                        y: old_pos.y,
                        bg: render.bg,
                    });
                    animation_builder.request(AnimationRequest::Teleportation {
                        x: new_pt.x,
                        y: new_pt.y,
                        bg: render.bg,
                    })
                }
            }
        }
        wants_to_teleport.clear();
    }
}


//----------------------------------------------------------------------------
// Process Requests to Change Position on the Map.
//
// This system is used by monsters and hazzards to change position, instead of
// mutating their state inside the system that determines a position change is
// needed. This works around some subtle issues of timing, where monsters
// targeting other monsters could have unpredictable effects depending on what
// orded movement is processes. Instead, we buffer requests to move and them
// process them inbatch here.
//
// This has one drawback: a monster will not know a tile they request movement
// into will become blocked at the time they are buffering their move request.
//----------------------------------------------------------------------------
pub struct PositionMovementSystem {}

#[derive(SystemData)]
pub struct PositionMovementSystemData<'a> {
    entities: Entities<'a>,
    map: WriteExpect<'a, Map>,
    player: ReadExpect<'a, Entity>,
    player_pt: WriteExpect<'a, Point>,
    avoids: ReadStorage<'a, MovementRoutingAvoids>,
    bounds: ReadStorage<'a, MovementRoutingBounds>,
    positions: WriteStorage<'a, Position>,
    viewsheds: WriteStorage<'a, Viewshed>,
    is_blockings: WriteStorage<'a, BlocksTile>,
    wants_to_move: WriteStorage<'a, WantsToMoveToPosition>,
}

impl<'a> System<'a> for PositionMovementSystem {

    type SystemData = PositionMovementSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let PositionMovementSystemData {
            entities,
            mut map,
            player,
            mut player_pt,
            mut positions,
            avoids,
            bounds,
            mut viewsheds,
            is_blockings,
            mut wants_to_move,
        } = data;

        for (entity, pos, wants_to_move) in (&entities, &mut positions, &wants_to_move).join() {

            let old_pos = pos.clone();
            let new_pos = wants_to_move.pt;
            let old_idx = map.xy_idx(old_pos.x, old_pos.y);
            let new_idx = map.xy_idx(new_pos.x, new_pos.y);

            let is_player = entity == *player;
            let is_blocking = is_blockings.get(entity).is_some();
            let eavoids = avoids.get(entity);
            let ebounds = bounds.get(entity);

            let new_pos_is_player_pos =
                new_pos.x == player_pt.x && new_pos.y == player_pt.y;
            let ok_to_move =
                wants_to_move.force || ok_to_move_to_position(&map, eavoids, ebounds, new_idx);

            if (!is_player && new_pos_is_player_pos) || !ok_to_move {
                continue
            }

            pos.x = new_pos.x;
            pos.y = new_pos.y;
            if is_player {
                player_pt.x = new_pos.x;
                player_pt.y = new_pos.y;
            }
            if is_blocking {
                map.blocked[old_idx] = false;
                map.blocked[new_idx] = true;
            }
            let viewshed = viewsheds.get_mut(entity);
            if let Some(viewshed) = viewshed {
                viewshed.dirty = true;
            }
        }
        wants_to_move.clear();
    }
}

// Check the constraints imposed on movement by the MovementRoutingAvoids and
// MovementRoutingBounds arrays are satisfied.
fn ok_to_move_to_position(
    map: &Map,
    avoids: Option<&MovementRoutingAvoids>,
    bounds: Option<&MovementRoutingBounds>,
    idx: usize
) -> bool {
    let ok_according_to_avoids = match avoids {
        None => !map.blocked[idx],
        Some(routing) => {
            !map.blocked[idx]
                && !(routing.fire && map.fire[idx])
                && !(routing.chill && map.chill[idx])
                && !(routing.deep_water && map.deep_water[idx])
        }
    };
    let ok_according_to_bounds = match bounds {
        None => true,
        Some(routing) => {
            (!routing.water || map.shallow_water[idx] || map.deep_water[idx])
            && (!routing.grass || map.grass[idx])
        }
    };
    ok_according_to_avoids && ok_according_to_bounds
}