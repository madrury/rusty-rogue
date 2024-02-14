
use specs::{prelude::*};
use super::{
    GameLog, Name, Position, WeaponSpecial, Equipped, Renderable,
    WeaponSpecialKind, AnimationRequestBuffer, AnimationRequest
};

//----------------------------------------------------------------------------
// Tick all status effect counteres and removee any that have expired.
//----------------------------------------------------------------------------
pub struct WeaponSpecialTickSystem {}

#[derive(SystemData)]
pub struct WeaponSpecialTickSystemData<'a> {
    entities: Entities<'a>,
    animation_buffer: WriteExpect<'a, AnimationRequestBuffer>,
    log: WriteExpect<'a, GameLog>,
    names: ReadStorage<'a, Name>,
    positions: WriteStorage<'a, Position>,
    renderables: ReadStorage<'a, Renderable>,
    equipped: ReadStorage<'a, Equipped>,
    specials: WriteStorage<'a, WeaponSpecial>
}

impl<'a> System<'a> for WeaponSpecialTickSystem {

    type SystemData = WeaponSpecialTickSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let WeaponSpecialTickSystemData {
            entities,
            mut animation_buffer,
            mut log,
            names,
            positions,
            renderables,
            equipped,
            mut specials,
        } = data;

        for (weapon, special, equipped) in (&entities, &mut specials, &equipped).join() {
            let owner = equipped.owner;
            let ownerposition = positions.get(owner);
            let ownerrender = renderables.get(owner);
            let recharged = special.tick();

            if !recharged { continue; }

            if let(Some(pos), Some(render)) = (ownerposition, ownerrender) {
                animation_buffer.request(AnimationRequest::WeaponSpecialRecharge {
                    x: pos.x,
                    y: pos.y,
                    fg: render.fg,
                    bg: render.bg,
                    glyph: render.glyph,
                })
            }
            // if recharged {
            //     if let (Some(nm), Some(pos)) = (ownername, ownerposition) {
            //         println!("{:?}'s weapon recharged.", nm.name);
            //     }
            // }
        }
    }
}