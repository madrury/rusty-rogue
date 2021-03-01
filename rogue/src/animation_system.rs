use specs::prelude::*;
use rltk::{RGB};
use super::{
    ParticleBuilder, ParticleRequest
};

//----------------------------------------------------------------------------
// Systems for handing Animation request.
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
        x: i32,
        y: i32,
        bg: RGB,
        glyph: rltk::FontCharType
    },
    Healing {
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType
    }
}

//  A buffer for holding current animation requests. Added to the ECS as a
//  resource, so any entity may access it.
pub struct AnimationBuilder {
    requests: Vec<AnimationRequest>,
}
impl AnimationBuilder {
    pub fn new() -> AnimationBuilder {
        AnimationBuilder {
            requests: Vec::new(),
        }
    }
    pub fn request(&mut self, request: AnimationRequest) {
        self.requests.push(request)
    }
}


// Handles grabbing AnimationREquests from the AnimationBuilder buffer, and
// converting them into the individual constituent ParticleRequests.
pub struct AnimationInitSystem {}

impl<'a> System<'a> for AnimationInitSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, AnimationBuilder>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut animation_builder, mut particle_builder) = data;
        for request in animation_builder.requests.iter() {
            particle_builder.request_many(
                match request {
                    AnimationRequest::MeleeAttack {x, y, bg, glyph}
                        => make_melee_animation(*x, *y, *bg, *glyph),
                    AnimationRequest::Healing {x, y, fg, bg, glyph}
                        => make_healing_animation(*x, *y, *fg, *bg, *glyph)
                }
            )
        }
        animation_builder.requests.clear();
    }
}

// A melee attack animation. The targeted entity flashes briefly.
fn make_melee_animation(x: i32, y: i32, bg: RGB, glyph: rltk::FontCharType) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let color_cycle = [
        rltk::RGB::named(rltk::WHITE),
        rltk::RGB::named(rltk::RED),
        rltk::RGB::named(rltk::WHITE),
    ];
    for (i, color) in color_cycle.iter().enumerate() {
        particles.push(ParticleRequest {
            x: x,
            y: y,
            fg: *color,
            bg: bg,
            glyph: glyph,
            lifetime: 50.0, // ms
            delay: 50.0 * (i as f32)
        })
    }
    particles
}

fn make_healing_animation(x: i32, y: i32, fg: RGB, bg: RGB, glyph: rltk::FontCharType) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let color_cycle = [
        rltk::RGB::named(rltk::RED),
        fg,
        rltk::RGB::named(rltk::RED),
    ];
    let glyph_cycle = [
        rltk::to_cp437('♥'),
        glyph,
        rltk::to_cp437('♥')
    ];
    for (i, (color, glyph)) in color_cycle.iter().zip(glyph_cycle.iter()).enumerate() {
        particles.push(ParticleRequest {
            x: x,
            y: y,
            fg: *color,
            bg: bg,
            glyph: *glyph,
            lifetime: 50.0, // ms
            delay: 50.0 * (i as f32)
        })
    }
    particles
}