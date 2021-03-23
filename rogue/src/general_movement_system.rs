use super::{
    Map, Point, Position, Viewshed, Renderable, BlocksTile,
    WantsToMoveToRandomPosition, AnimationBuilder, AnimationRequest
};
use specs::prelude::*;
use rltk::RandomNumberGenerator;


pub struct TeleportationSystem {}

#[derive(SystemData)]
pub struct TeleportationSystemData<'a> {
    entities: Entities<'a>,
    map: WriteExpect<'a, Map>,
    rng: WriteExpect<'a, RandomNumberGenerator>,
    animation_builder: WriteExpect<'a, AnimationBuilder>,
    player: ReadExpect<'a, Entity>,
    player_pos: WriteExpect<'a, Point>,
    positions: WriteStorage<'a, Position>,
    viewsheds: WriteStorage<'a, Viewshed>,
    renderables: ReadStorage<'a, Renderable>,
    is_blockings: WriteStorage<'a, BlocksTile>,
    wants_to_teleport: WriteStorage<'a, WantsToMoveToRandomPosition>,
}

impl<'a> System<'a> for TeleportationSystem {

    type SystemData = TeleportationSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let TeleportationSystemData {
            entities,
            mut map,
            mut rng,
            mut animation_builder,
            player,
            mut player_pos,
            mut positions,
            mut viewsheds,
            renderables,
            is_blockings,
            mut wants_to_teleport,
        } = data;

        for (entity, pos, _wants_to_teleport) in (&entities, &mut positions, &wants_to_teleport).join() {

            let old_pos = pos.clone();
            let is_player = entity == *player;
            let is_blocking = is_blockings.get(entity).is_some();

            let new_pos = map.random_unblocked_point(10, &mut *rng);
            if let Some(new_pos) = new_pos {
                pos.x = new_pos.0;
                pos.y = new_pos.1;
                if is_player {
                    player_pos.x = new_pos.0;
                    player_pos.y = new_pos.1;
                }
                if is_blocking {
                    let old_idx = map.xy_idx(old_pos.x, old_pos.y);
                    let new_idx = map.xy_idx(new_pos.0, new_pos.1);
                    map.blocked[old_idx] = false;
                    map.blocked[new_idx] = true;
                }
                let viewshed = viewsheds.get_mut(entity);
                if let Some(viewshed) = viewshed {
                    viewshed.dirty = true;
                }
                let render = renderables.get(entity);
                if let Some(render) = render {
                    animation_builder.request(AnimationRequest::Teleportation {
                        x: old_pos.x,
                        y: old_pos.y,
                        bg: render.bg,
                    });
                    animation_builder.request(AnimationRequest::Teleportation {
                        x: new_pos.0,
                        y: new_pos.1,
                        bg: render.bg,
                    })
                }
            }
        }
        wants_to_teleport.clear();
    }
}