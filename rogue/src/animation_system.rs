use super::{Map, ParticleRequestBuffer, ParticleRequest};
use rltk::{Point, RGB};
use specs::prelude::*;

//----------------------------------------------------------------------------
// Systems for handing Animation requests.
//
// This system handles requests from entities to play an animation. It is meant
// to allow these requests at a higher level of abstraction than each entity
// needing to request the individual particles that make up the animation.
//
// To request an animation, the entity contructs the appropriate
// AnimationRequest and pushes it to the AnimationBuilder buffer.
//----------------------------------------------------------------------------

// An individual request fon an animation. This enumerates over all possible
// game animations.
pub enum AnimationRequest {
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

//  A buffer for holding current animation requests. Added to the ECS as a
//  resource, so any entity may access it.
pub struct AnimationRequestBuffer {
    requests: Vec<AnimationRequest>,
}
impl AnimationRequestBuffer {
    pub fn new() -> AnimationRequestBuffer {
        AnimationRequestBuffer {
            requests: Vec::new(),
        }
    }
    pub fn request(&mut self, request: AnimationRequest) {
        self.requests.push(request)
    }
}

// Handles grabbing AnimationRequests from the AnimationBuilder buffer, and
// converting them into the individual constituent ParticleRequests.
pub struct AnimationInitSystem {}

impl<'a> System<'a> for AnimationInitSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteExpect<'a, AnimationRequestBuffer>,
        WriteExpect<'a, ParticleRequestBuffer>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut animation_builder, mut particle_request_buffer) = data;
        for request in animation_builder.requests.iter() {
            particle_request_buffer.request_many(match request {
                AnimationRequest::MeleeAttack {pt, bg, glyph} => {
                    make_melee_animation(*pt, *bg, *glyph)
                }
                AnimationRequest::Healing { pt, fg, bg, glyph, } =>
                    make_healing_animation(*pt, *fg, *bg, *glyph),
                AnimationRequest::WeaponSpecialRecharge { pt, fg, bg, owner_glyph, weapon_glyph } =>
                    make_weapon_special_recharge_animation(*pt, *fg, *bg, *owner_glyph, *weapon_glyph),
                AnimationRequest::AreaOfEffect { center, fg, bg, glyph, radius } =>
                    make_aoe_animation(&*map, *center, *fg, *bg, *glyph, *radius),
                AnimationRequest::AlongRay {
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
                    *until_blocked
                ),
                AnimationRequest::SpinAttack { pt, fg, glyph }
                    => make_spin_attack_animation(*pt, *fg, *glyph),
                AnimationRequest::Teleportation {pt, bg}
                    => make_teleportation_animation(*pt, *bg),
            })
        }
        animation_builder.requests.clear();
    }
}

// A melee attack animation. The targeted entity flashes briefly.
fn make_melee_animation(
    pt: Point,
    bg: RGB,
    glyph: rltk::FontCharType,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let color_cycle = [
        rltk::RGB::named(rltk::WHITE),
        rltk::RGB::named(rltk::RED),
        rltk::RGB::named(rltk::WHITE),
    ];
    for (i, color) in color_cycle.iter().enumerate() {
        particles.push(ParticleRequest {
            x: pt.x,
            y: pt.y,
            fg: *color,
            bg: bg,
            glyph: glyph,
            // Slightly shorter than the other animations so combat feels
            // snappyer.
            lifetime: 35.0, // ms
            delay: 35.0 * (i as f32),
        })
    }
    particles
}

// The weapon special has recharged.
fn make_weapon_special_recharge_animation(
    pt: Point,
    fg: RGB,
    bg: RGB,
    owner_glyph: rltk::FontCharType,
    weapon_glyph: rltk::FontCharType,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let color_cycle = [rltk::RGB::named(rltk::GREEN), fg, rltk::RGB::named(rltk::GREEN)];
    let glyph_cycle = [weapon_glyph, owner_glyph, weapon_glyph];
    for (i, (color, glyph)) in color_cycle.iter().zip(glyph_cycle.iter()).enumerate() {
        particles.push(ParticleRequest {
            x: pt.x,
            y: pt.y,
            fg: *color,
            bg: bg,
            glyph: *glyph,
            lifetime: 50.0, // ms
            delay: 50.0 * (i as f32),
        })
    }
    particles
}

// A healing animation. A heart flashes quickly.
fn make_healing_animation(
    pt: Point,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let color_cycle = [rltk::RGB::named(rltk::RED), fg, rltk::RGB::named(rltk::RED)];
    let glyph_cycle = [rltk::to_cp437('♥'), glyph, rltk::to_cp437('♥')];
    for (i, (color, glyph)) in color_cycle.iter().zip(glyph_cycle.iter()).enumerate() {
        particles.push(ParticleRequest {
            x: pt.x,
            y: pt.y,
            fg: *color,
            bg: bg,
            glyph: *glyph,
            lifetime: 50.0, // ms
            delay: 50.0 * (i as f32),
        })
    }
    particles
}

// An aoe animation. Color cycles glyphs in a circle around a point.
fn make_aoe_animation(
    map: &Map,
    pt: Point,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    radius: f32,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let fg_color_cycle = [fg, bg, fg];
    let bg_color_cycle = [bg, fg, bg];
    let blast_tiles = map.get_euclidean_disk_around(pt, radius);
    for (i, (fg, bg)) in fg_color_cycle.iter().zip(bg_color_cycle.iter()).enumerate() {
        for tile in blast_tiles.iter() {
            particles.push(ParticleRequest {
                x: tile.x,
                y: tile.y,
                fg: *fg,
                bg: *bg,
                glyph: glyph,
                lifetime: 100.0,
                delay: 100.0 * (i as f32),
            })
        }
    }
    particles
}

// An animation that travels along a ray from source to target.
fn make_along_ray_animation(
    map: &Map,
    source: Point,
    target: Point,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    until_blocked: bool
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let tiles = map.get_ray_tiles(source, target, until_blocked);
    for (i, tile) in tiles.into_iter().enumerate() {
        particles.push(ParticleRequest {
            x: tile.x,
            y: tile.y,
            fg: fg,
            bg: bg,
            glyph: glyph,
            lifetime: 40.0,
            delay: 20.0 * (i as f32)
        })
    }
    particles
}

// A healing animation. A heart flashes quickly.
fn make_teleportation_animation(
    pt: Point,
    bg: RGB,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let glyph_cycle = [rltk::to_cp437('░'), rltk::to_cp437(' '), rltk::to_cp437('░')];
    for (i, glyph) in glyph_cycle.iter().enumerate() {
        particles.push(ParticleRequest {
            x: pt.x,
            y: pt.y,
            fg: rltk::RGB::named(rltk::MEDIUM_PURPLE),
            bg: bg,
            glyph: *glyph,
            lifetime: 100.0, // ms
            delay: 100.0 * (i as f32),
        })
    }
    particles
}

// Spin attack, like in Link to the Past.
fn make_spin_attack_animation(
    pt: Point,
    fg: RGB,
    glyph: rltk::FontCharType,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let drs = vec![
        (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0)
    ];
    for (i, dr) in drs.iter().enumerate() {
        particles.push(ParticleRequest {
            x: pt.x + dr.0,
            y: pt.y + dr.1,
            fg: fg,
            // TODO: It's difficult at the moment to determine the correct
            // background color for a tile. We default to BLACK here, but in the
            // future, this should likely be stored explicitly on the map, and
            // updated each turn in the MapIndexingSystem.
            // There are likely other places this is relevent, so an audit would
            // be needed to determine where we are already using a worse
            // solution to this.
            bg: rltk::RGB::named(rltk::BLACK),
            glyph: glyph,
            lifetime: 40.0, // ms
            delay: 20.0 * (i as f32),
        })
    }
    particles
}