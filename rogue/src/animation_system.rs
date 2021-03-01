use specs::prelude::*;
use rltk::{RGB};
use super::{
    ParticleBuilder, ParticleRequest
};


pub enum AnimationRequest {
    MeleeAttack {
        x: i32,
        y: i32,
        bg: RGB,
        glyph: rltk::FontCharType
    }
}

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
                        => make_melee_animation(*x, *y, *bg, *glyph)
                }
            )
        }
        animation_builder.requests.clear();
    }
}

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