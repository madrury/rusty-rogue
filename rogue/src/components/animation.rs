use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::{RGB, Point};

use crate::Map;

/*
Components and Data Structures for Game Animations
--------------------------------------------------

Data structures, components, and functions for modeling game animations.

We model animations at varying levels of abstraction, so that entities may
specify animations at a high level (i.e. "animate an object moving along a ray
followed by an area of effect"). These are broken down into atomic particles by
game systems, and these particles are what's actually rendered.

AnimationParticles
------------------
At the lowest level, an animation is assembled from atomic AnimationParticles,
which are rendered after some delay (ms) for a finite lifetime (ms).
AnimationParticle is a component encapsulating the information needed to render
the particle, when running the animation it is attached to an uninteractable
entity only for the purposes of rendering, which is removed from teh ECS once
its lifetime has expired.

AnimationBlock
--------------
A higher level abstraction of an animation, intended to be functional pieces
that are composed into single animation events. For example, throwing a fire
potion (which then explodes) is represented as:

    AnimationBlock::AlongRay {..} -> AnimationBlock::AreaOfEffect {..}

AnimationBlocks are transformed into a set of AnimationParticles, a process we
call particalization.

Animation blocks are often created during the running of game systems when some
event is known to occur. For example, a weapon special recharging creates a
AnimationDlock::WeaponSpecialRecharge {..} and inserts it into a buffer ECS
resource for later particalization.

AnimationSequence
-----------------
Encapsulates a sequence of animation blocks, as in teh afomentioned example:

    AnimationSequence:
    [AnimationBlock::AlongRay {..}, AnimationBlock::AreaOfEffect {..}]

This is useful because, when particalizing a *sequence* that is meant to be
played one after the other, we need to track the total lifetime of the first
block. It is useful to have an encapsulating object to attach this sequence
particalization opeartion to.

AnimationComponentData
----------------------
The highest level of specifiation, embedded within a component to indicate that
some action trigers an animation. For example:

    AnimationWhenThrown {
        sequence: vec![
            AnimationComponentData::AlongRay {..}.
            AnimationComponentData::AreaOfEffect {..}.
        ]
    }

An AnimationComponentData obejct is *localized* when the indicated action is
detected, which means we combine the data with the current game state (positions
of the entities at play) to produce a sequence of AnimationBlocks, whcih are
inserted into the ECS.
*/

type Milliseconds = f32;
type Frames = i32;

const FRAME_TIME_MS: Milliseconds = 16.666666666666668;
fn ms(frms: Frames) -> Milliseconds {FRAME_TIME_MS * frms as f32}

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
    pub pt: Point,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType
}


//------------------------------------------------------------------------------
// An ECS resource for holding AnimationParticle components that have not yet
// been attached to new enetities for rendering in the game engine.
//
// This buffer processed in ParticleInitSystem, which empties the buffer and
// inserts renderable entities intot he ECS.
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
                particlize_melee_animation(*pt, *bg, *glyph, delay)
            }
            AnimationBlock::Healing { pt } =>
                particlize_healing_animation(*pt, delay),
            AnimationBlock::WeaponSpecialRecharge { pt, fg, bg, owner_glyph, weapon_glyph } =>
                particlize_weapon_special_recharge_animation(*pt, *fg, *bg, *owner_glyph, *weapon_glyph, delay),
            AnimationBlock::AreaOfEffect { center, fg, bg, glyph, radius } =>
                particlize_aoe_animation(&*map, *center, *fg, *bg, *glyph, *radius, delay),
            AnimationBlock::AlongRay {
                source,
                target,
                fg,
                bg,
                glyph,
                until_blocked
            } => particlize_along_ray_animation(
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
                => particlize_spin_attack_animation(*pt, *fg, *glyph, delay),
            AnimationBlock::Teleportation {pt, bg}
                => particlize_teleportation_animation(*pt, *bg, delay),
        }
    }
}

// Encapsulates a sequence of AnimationBlocks constituting a single event's
// animation.
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

// ECS resource for buffering AnimationSequences. Processed in
// AnimationParticlizationSystem.
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


// Component level data triggering a game animation when the owner is
// Thrown|Cast.
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
    Healing
}
impl AnimationComponentData {
    pub fn localize_targeted_or_cast(&self, source: Option<Point>, target: Option<Point>) -> Option<AnimationBlock> {
        match self {
            AnimationComponentData::AreaOfEffect { radius, fg, bg, glyph } => {
                target.map(|t| AnimationBlock::AreaOfEffect {
                    center: t,
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
            },
            AnimationComponentData::Healing => {
                target.map(|t| AnimationBlock::Healing { pt: t })
            }
        }
    }
    pub fn localize_used(&self, source: Option<Point>) -> Option<AnimationBlock> {
        match self {
            AnimationComponentData::AreaOfEffect { radius, fg, bg, glyph } => {
                source.map(|s| AnimationBlock::AreaOfEffect {
                    center: s,
                    radius: *radius,
                    fg: *fg,
                    bg: *bg,
                    glyph: *glyph
                })
            },
            AnimationComponentData::AlongRay {..} => { None }
            AnimationComponentData::Healing  => {
                source.map(|s| AnimationBlock::Healing { pt: s })
            }
        }
    }
}


#[derive(Component, ConvertSaveload, Clone)]
pub struct AnimationWhenUsed {
    pub sequence: Vec<AnimationComponentData>
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
// Particlization implementations.
//----------------------------------------------------------------------------
fn particlize_melee_animation(
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
            lifetime: ms(2),
            delay: ms(delay) + ms(2 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * 2)
}

fn particlize_weapon_special_recharge_animation(
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
            lifetime: ms(4),
            delay: ms(delay) + ms(4 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * 4)
}

fn particlize_healing_animation(
    pt: Point,
    delay: Frames
) -> (Vec<AnimationParticle>, Frames) {
    let mut particles = Vec::new();
    let color_cycle_fg = [RGB::named(rltk::RED), RGB::named(rltk::WHITE), RGB::named(rltk::RED)];
    let color_cycle_bg = [RGB::named(rltk::WHITE), RGB::named(rltk::RED), RGB::named(rltk::WHITE)];
    for (i, (fg, bg)) in color_cycle_fg.iter().zip(color_cycle_bg.iter()).enumerate() {
        particles.push(AnimationParticle {
            pt: pt,
            fg: *fg,
            bg: *bg,
            glyph: rltk::to_cp437('♥'),
            lifetime: ms(4),
            delay: ms(delay) + ms(4 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * 4)
}

fn particlize_aoe_animation(
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
                lifetime: ms(2),
                delay: ms(delay) + ms(2 * i as i32),
                displayed: false
            })
        }
    }
    (particles, delay + 6 * 2)
}

fn particlize_along_ray_animation(
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
            lifetime: ms(4),
            delay: ms(delay) + ms(3 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * n_tiles as i32)
}

fn particlize_teleportation_animation(
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
            lifetime: ms(4),
            delay: ms(delay) + ms(4 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 3 * 4)
}

fn particlize_spin_attack_animation(
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
            lifetime: ms(4),
            delay: ms(delay) + ms(2 * i as i32),
            displayed: false
        })
    }
    (particles, delay + 2 * drs.len() as i32 + 2)
}