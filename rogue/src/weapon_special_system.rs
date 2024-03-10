use specs::prelude::*;
use crate::components::melee::*;
use crate::components::magic::*;

use super::{
    GameLog, Name, Position, WeaponSpecial, Equipped, Renderable,
    AnimationSequenceBuffer, AnimationBlock
};

//----------------------------------------------------------------------------
// Tick equipped weapon specials, and notify the player when fully charged.
//----------------------------------------------------------------------------
pub struct WeaponSpecialTickSystem {}

#[derive(SystemData)]
pub struct WeaponSpecialTickSystemData<'a> {
    entities: Entities<'a>,
    animation_buffer: WriteExpect<'a, AnimationSequenceBuffer>,
    log: WriteExpect<'a, GameLog>,
    names: ReadStorage<'a, Name>,
    positions: WriteStorage<'a, Position>,
    renderables: ReadStorage<'a, Renderable>,
    equipped: ReadStorage<'a, Equipped>,
    specials: WriteStorage<'a, WeaponSpecial>,
    spellbooks: WriteStorage<'a, InSpellBook>,
    singlecast: WriteStorage<'a, RemovedFromSpellBookWhenCast>
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
            mut spellbooks,
            mut singlecast,
        } = data;

        for (weapon, special, equipped) in (&entities, &mut specials, &equipped).join() {
            let recharged = special.tick();
            if !recharged {
                continue;
            }

            let owner = equipped.owner;
            let ownername = names.get(owner);
            let weaponname = names.get(weapon);
            let ownerrender = renderables.get(owner);
            let weaponrender = renderables.get(weapon);
            let ownerposition = positions.get(owner);

            match special.kind {
                WeaponSpecialKind::AddToSpellBook => {
                    spellbooks.insert(weapon, InSpellBook {
                        owner: owner,
                        slot: BlessingSlot::None
                    }).expect("Failed to insert InSpellBook component on special charge.");
                    singlecast.insert(weapon, RemovedFromSpellBookWhenCast {})
                        .expect("Failed to insert SingleCast component upon special charge.");
                }
                _ => {}
            }

            if let(Some(pos), Some(orender), Some(wrender)) = (ownerposition, ownerrender, weaponrender) {
                animation_buffer.request_block(AnimationBlock::WeaponSpecialRecharge {
                    pt: pos.to_point(),
                    fg: orender.fg,
                    owner_glyph: orender.glyph,
                    weapon_glyph: wrender.glyph
                })
            }

            if let (Some(on), Some(wn)) = (ownername, weaponname) {
                let logstr = match special.kind {
                    WeaponSpecialKind::ThrowWithoutExpending =>
                        format!("{}'s {} glints menacingly.", on.name, wn.name),
                    WeaponSpecialKind::Dash =>
                        format!("{}'s {} glints menacingly.", on.name, wn.name),
                    WeaponSpecialKind::SpinAttack =>
                        format!("{}'s {} glints menacingly.", on.name, wn.name),
                    WeaponSpecialKind::AddToSpellBook =>
                        format!("{}'s {} sparks with magical energy.", on.name, wn.name)
                };
                log.entries.push(logstr);
            }
        }
    }
}