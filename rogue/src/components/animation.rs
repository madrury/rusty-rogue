use log::{info};
use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::{RGB, Point};

use crate::Map;

//------------------------------------------------------------------
// Components and data structures for game animations.
//------------------------------------------------------------------
type Milliseconds = f32;
type Frames = i32;

const FRAME_TIME_MS: Milliseconds = 16.666666666666668;
fn frames(frms: Frames) -> Milliseconds {FRAME_TIME_MS * frms as f32}

//------------------------------------------------------------------------------
// An atomic, irreducible piece of a game animation. Rendered for an interval of
// wall clock time after some delay, after which is is destroyed. Has no game
// effect, only for pretty.
//------------------------------------------------------------------------------
#[derive(Component)]
pub struct AnimationParticle {
    // For how long should this particle be displayed?
    pub lifetime : Milliseconds,
    // How many milliseconds after the animation starts should this particle be
    // displayed?
    pub delay: Milliseconds,
    // Boolean flag, is the particle currently being rendered? Flipped when
    // delay_ms has passed after adding the particle to the ECS.
    pub displayed: bool,
    // Rendering data.
    // TODO: Should we introduce a simpledata structure enacapsulating this
    // data, which is the same as included in the Renderable component.
    pub pt: Point,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType
}


//------------------------------------------------------------------------------
// An ECS resource for holding AnimationParticle components that have not yet
// been attached to new enetities for rendering in the game engine.
//------------------------------------------------------------------------------
pub struct AnimationParticleBuffer {
    requests: Vec<AnimationParticle>,
}
impl AnimationParticleBuffer {
    pub fn new() -> AnimationParticleBuffer {
        AnimationParticleBuffer {
            requests: Vec::new(),
        }
    }
    pub fn pop(&mut self) -> Option<AnimationParticle> {
        self.requests.pop()
    }
    pub fn is_empty(&mut self) -> bool{
        self.requests.is_empty()
    }
    pub fn request(&mut self, particle: AnimationParticle) {
        self.requests.push(particle)
    }
    pub fn request_many(&mut self, particles: Vec<AnimationParticle>) {
        for req in particles {
            self.request(req);
        }
    }
}


//------------------------------------------------------------------------------
// A conceptual block of a game animation. These may be concatenated together to
// form compound animations. For example, an AlongRay animation may be
// concatenated with a AreaOfEffect animation to animate throwning an explosive
// fire potion.
//
// MeleeAttack:
//     The target of a melee attack winces.
// Healing:
//     A player | monster entity is healed.
// WeaponSpecialRecharge:
//     The players equipped weapon recharges its special attack.
// AreaOfEffect:
//     Cycle fg and bg colors of a glyph in an area of effect centered at a
//     source point. For explions and bursts of gas.
// AlongRay:
//     A particle travels along a ray from a source to a target.
// SpinAttack:
//     A weapon's spin attack, as seen in Link to the Past.
// Teloportation:
//     Dissapear from some place, appear in another place.
//------------------------------------------------------------------------------
#[derive(Debug)]
pub enum AnimationBlock {
    MeleeAttack {
        pt: Point,
        bg: RGB,
        glyph: rltk::FontCharType,
    },
    Healing {
        pt: Point,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
    },
    WeaponSpecialRecharge {
        pt: Point,
        fg: RGB,
        bg: RGB,
        owner_glyph: rltk::FontCharType,
        weapon_glyph: rltk::FontCharType
    },
    AreaOfEffect {
        center: Point,
        radius: f32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
    },
    AlongRay {
        source: Point,
        target: Point,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
        until_blocked: bool
    },
    SpinAttack {
        pt: Point,
        fg: RGB,
        glyph: rltk::FontCharType,
    },
    Teleportation {
        pt: Point,
        bg: RGB,
    },
}
impl AnimationBlock {
    // Instantiate the atomic AnimationParticle's that constitute an animation block.
    pub fn particalize(&self, map: &Map, delay: Frames) -> (Vec<AnimationParticle>, Frames) {
        match self {
            AnimationBlock::MeleeAttack {pt, bg, glyph} => {
                make_melee_animation(*pt, *bg, *glyph, delay)
            }
            AnimationBlock::Healing { pt, fg, bg, glyph, } =>
                make_healing_animation(*pt, *fg, *bg, *glyph, delay),
            AnimationBlock::WeaponSpecialRecharge { pt, fg, bg, owner_glyph, weapon_glyph } =>
                make_weapon_special_recharge_animation(*pt, *fg, *bg, *owner_glyph, *weapon_glyph, delay),
            AnimationBlock::AreaOfEffect { center, fg, bg, glyph, radius } =>
                make_aoe_animation(&*map, *center, *fg, *bg, *glyph, *radius, delay),
            AnimationBlock::AlongRay {
                source,
                target,
                fg,
                bg,
                glyph,
                until_blocked
            } => make_along_ray_animation(
                &*map,
                *source,
                *target,
                *fg,
                *bg,
                *glyph,
                *until_blocked,
                delay
            ),
            AnimationBlock::SpinAttack { pt, fg, glyph }
                => make_spin_attack_animation(*pt, *fg, *glyph, delay),
            AnimationBlock::Teleportation {pt, bg}
                => make_teleportation_animation(*pt, *bg, delay),
        }
    }
}

pub struct AnimationSequence {
    pub blocks: Vec<AnimationBlock>
}
impl AnimationSequence {
    pub fn from_blocks(blocks: Vec<AnimationBlock>) -> AnimationSequence {
        AnimationSequence {blocks}
    }
    pub fn particalize(&mut self, map: &Map) -> Vec<AnimationParticle> {
        let mut particles = Vec::<AnimationParticle>::new();
        let mut delay: Frames = 0;
        while !self.blocks.is_empty() {
            let block = self.blocks.pop();
            let newparticles = block.map(|blk| blk.particalize(map, delay));
            match newparticles {
                Some((ps, dl)) => {
                    delay += dl;
                    particles.extend(ps);
                }
                None => { continue }
            }

        }
        particles
    }
}

pub struct AnimationSequenceBuffer {
    requests: Vec<AnimationSequence>,
}
impl AnimationSequenceBuffer {
    pub fn new() -> AnimationSequenceBuffer {
        AnimationSequenceBuffer {
            requests: Vec::<AnimationSequence>::new(),
        }
    }
    pub fn pop(&mut self) -> Option<AnimationSequence> {
        self.requests.pop()
    }
    pub fn is_empty(&mut self) -> bool{
        self.requests.is_empty()
    }
    pub fn request_sequence(&mut self, request: AnimationSequence) {
        self.requests.push(request)
    }
    pub fn request_block(&mut self, request: AnimationBlock) {
        self.requests.push(AnimationSequence {blocks: vec![request]})
    }
}


#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum AnimationComponentData {
    AreaOfEffect {
        radius: f32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
    },
    AlongRay {
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
        until_blocked: bool
    },
}
impl AnimationComponentData {
    pub fn localize(&self, source: Option<Point>, target: Option<Point>) -> Option<AnimationBlock> {
        match self {
            AnimationComponentData::AreaOfEffect { radius, fg, bg, glyph } => {
                target.map(|s| AnimationBlock::AreaOfEffect {
                    center: s,
                    radius: *radius,
                    fg: *fg,
                    bg: *bg,
                    glyph: *glyph
                })
            },
            AnimationComponentData::AlongRay { fg, bg, glyph, until_blocked } => {
                source.zip(target).map(|(s, t)| AnimationBlock::AlongRay {
                    source: s,
                    target: t,
                    fg: *fg,
                    bg:*bg,
                    glyph: *glyph,
                    until_blocked: *until_blocked
                })
            }
        }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct AnimationWhenThrown {
    pub sequence: Vec<AnimationComponentData>
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct AnimationWhenCast {
    pub sequence: Vec<AnimationComponentData>
}

//----------------------------------------------------------------------------
// Animation implementations.
//----------------------------------------------------------------------------

// A melee attack animation. The targeted entity flashes briefly.
fn make_melee_animation(
    pt: Point,
    bg: RGB,
    glyph: rltk::FontCharType,
    delay: Frames
) -> (Vec<AnimationParticle>, Frames) {
    let mut particles = Vec::new();
    let color_cycle = [
        rltk::RGB::named(rltk::WHITE),
        rltk::RGB::named(rltk::RED),
        rltk::RGB::named(rltk::WHITE),
    ];
    for (i, color) in color_cycle.iter().enumerate() {
        particles.push(AnimationParticle {
            pt: pt,
            fg: *color,
            bg: bg,
            glyph: glyph,
            lifetime: frames(2),
            delay: frames(delay) + frames(2 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * 2)
}

// The weapon special has recharged.
fn make_weapon_special_recharge_animation(
    pt: Point,
    fg: RGB,
    bg: RGB,
    owner_glyph: rltk::FontCharType,
    weapon_glyph: rltk::FontCharType,
    delay: Frames
) -> (Vec<AnimationParticle>, Frames) {
    let mut particles = Vec::new();
    let color_cycle = [rltk::RGB::named(rltk::GREEN), fg, rltk::RGB::named(rltk::GREEN)];
    let glyph_cycle = [weapon_glyph, owner_glyph, weapon_glyph];
    for (i, (color, glyph)) in color_cycle.iter().zip(glyph_cycle.iter()).enumerate() {
        particles.push(AnimationParticle {
            pt: pt,
            fg: *color,
            bg: bg,
            glyph: *glyph,
            lifetime: frames(4),
            delay: frames(delay) + frames(4 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * 4)
}

// A healing animation. A heart flashes quickly.
fn make_healing_animation(
    pt: Point,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    delay: Frames
) -> (Vec<AnimationParticle>, Frames) {
    let mut particles = Vec::new();
    let color_cycle = [rltk::RGB::named(rltk::RED), fg, rltk::RGB::named(rltk::RED)];
    let glyph_cycle = [rltk::to_cp437('♥'), glyph, rltk::to_cp437('♥')];
    for (i, (color, glyph)) in color_cycle.iter().zip(glyph_cycle.iter()).enumerate() {
        particles.push(AnimationParticle {
            pt: pt,
            fg: *color,
            bg: bg,
            glyph: *glyph,
            lifetime: frames(4),
            delay: frames(delay) + frames(4 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * 4)
}

// An aoe animation. Color cycles glyphs in a circle around a point.
fn make_aoe_animation(
    map: &Map,
    pt: Point,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    radius: f32,
    delay: Frames
) -> (Vec<AnimationParticle>, Frames) {
    let mut particles = Vec::new();
    let fg_color_cycle = [fg, bg, fg, bg, fg, bg];
    let bg_color_cycle = [bg, fg, bg, fg, bg, fg];
    let blast_tiles = map.get_euclidean_disk_around(pt, radius);
    for (i, (fg, bg)) in fg_color_cycle.iter().zip(bg_color_cycle.iter()).enumerate() {
        for tile in blast_tiles.iter() {
            particles.push(AnimationParticle {
                pt: *tile,
                fg: *fg,
                bg: *bg,
                glyph: glyph,
                lifetime: frames(2),
                delay: frames(delay) + frames(2 * i as i32),
                displayed: false
            })
        }
    }
    (particles, delay + 6 * 2)
}

// An animation that travels along a ray from source to target.
fn make_along_ray_animation(
    map: &Map,
    source: Point,
    target: Point,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    until_blocked: bool,
    delay: Frames
) -> (Vec<AnimationParticle>, Frames) {
    let mut particles = Vec::new();
    let tiles = map.get_ray_tiles(source, target, until_blocked);
    let n_tiles = tiles.len() as i32;
    for (i, tile) in tiles.into_iter().enumerate() {
        particles.push(AnimationParticle {
            pt: tile,
            fg: fg,
            bg: bg,
            glyph: glyph,
            lifetime: frames(4),
            delay: frames(delay) + frames(3 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * n_tiles as i32)
}

// A healing animation. A heart flashes quickly.
fn make_teleportation_animation(
    pt: Point,
    bg: RGB,
    delay: Frames
) -> (Vec<AnimationParticle>, Frames) {
    let mut particles = Vec::new();
    let glyph_cycle = [rltk::to_cp437('░'), rltk::to_cp437(' '), rltk::to_cp437('░')];
    for (i, glyph) in glyph_cycle.iter().enumerate() {
        particles.push(AnimationParticle {
            pt: pt,
            fg: rltk::RGB::named(rltk::MEDIUM_PURPLE),
            bg: bg,
            glyph: *glyph,
            lifetime: frames(4),
            delay: frames(delay) + frames(4 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * 4)
}

// Spin attack, like in Link to the Past.
fn make_spin_attack_animation(
    pt: Point,
    fg: RGB,
    glyph: rltk::FontCharType,
    delay: Frames
) -> (Vec<AnimationParticle>, Frames) {
    let mut particles = Vec::new();
    let drs = vec![
        (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0)
    ];
    for (i, dr) in drs.iter().enumerate() {
        particles.push(AnimationParticle {
            pt: Point {x: pt.x + dr.0, y: pt.y + dr.1},
            fg: fg,
            // TODO: It's difficult at the moment to determine the correct
            // background color for a tile. We default to BLACK here, but in the
            // future, this should likely be stored explicitly on the map, and
            // updated each turn in the MapIndexingSystem.
            // There are likely other places this is relevent, so an audit would
            // be needed to determine where we are already using a worse
            // solution to this problem.
            bg: rltk::RGB::named(rltk::BLACK),
            glyph: glyph,
            lifetime: frames(4),
            delay: frames(delay) + frames(2 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 2 * drs.len() as i32 + 2)
}