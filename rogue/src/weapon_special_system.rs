
use specs::prelude::*;
use super::{
    GameLog, Name, Position, WeaponSpecial, Equipped, Renderable,
    AnimationRequestBuffer, AnimationRequest
};

//----------------------------------------------------------------------------
// Tick equipped weapon specials, and notify the player when fully charged.
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
            let recharged = special.tick();
            if !recharged { continue; }

            let owner = equipped.owner;
            let ownername = names.get(owner);
            let weaponname = names.get(weapon);
            let ownerrender = renderables.get(owner);
            let weaponrender = renderables.get(weapon);
            let ownerposition = positions.get(owner);

            if let(Some(pos), Some(orender), Some(wrender)) = (ownerposition, ownerrender, weaponrender) {
                animation_buffer.request(AnimationRequest::WeaponSpecialRecharge {
                    x: pos.x,
                    y: pos.y,
                    fg: orender.fg,
                    bg: orender.bg,
                    owner_glyph: orender.glyph,
                    weapon_glyph: wrender.glyph
                })
            }
            if let (Some(on), Some(wn)) = (ownername, weaponname) {
                log.entries.push(format!(
                    "{}'s {} glints menacingly.",
                    on.name, wn.name
                ));
            }
        }
    }
}