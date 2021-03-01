use specs::prelude::*;
use rltk::{RGB, Rltk};
use super::{Position, Renderable, ParticleLifetime};


pub fn update_particle_lifetimes(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles : Vec<Entity> = Vec::new();
    {
        let mut particles = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();
        for (entity, mut particle) in (&entities, &mut particles).join() {
            if particle.lifetime <= 0.0 {
                dead_particles.push(entity);
            } else if particle.delay <= 0.0 {
                particle.lifetime -= ctx.frame_time_ms;
            } else {
                particle.delay = f32::max(0.0, particle.delay - ctx.frame_time_ms);
            }
        }
    }
    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Particle will not die.");
    }
}


pub struct ParticleRenderSystem {}

impl<'a> System<'a> for ParticleRenderSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut particles) = data;
        for (entity, p) in (&entities, &mut particles).join() {
            if p.delay <= 0.0 && !p.displayed {
                positions.insert(entity, Position {x: p.x, y: p.y})
                    .expect("Unable to insert particle position.");
                renderables.insert(entity, Renderable {fg: p.fg, bg: p.bg, glyph: p.glyph, order: 0})
                    .expect("Unable to insert particle renderable.");
                p.displayed = true;
            }
        }
    }
}


pub struct ParticleInitSystem {}

impl<'a> System<'a> for ParticleInitSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, ParticleLifetime>,
        WriteExpect<'a, ParticleBuilder>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut particles, mut particle_builder) = data;
        for request in particle_builder.requests.iter() {
            let p = entities.create();
            particles.insert(p, ParticleLifetime {
                lifetime : request.lifetime,
                delay: request.delay,
                displayed: false,
                x: request.x,
                y: request.y,
                fg: request.fg,
                bg: request.bg,
                glyph: request.glyph
            }).expect("Unable to insert new particle.");
        }
        particle_builder.requests.clear();
    }
}



pub struct ParticleRequest {
    pub x: i32,
    pub y: i32,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType,
    pub lifetime: f32,
    pub delay: f32,
}

pub struct ParticleBuilder {
    requests : Vec<ParticleRequest>
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {requests: Vec::new()}
    }
    pub fn request(&mut self, request: ParticleRequest) {
        self.requests.push(request)
    }
    pub fn request_many(&mut self, requests: Vec<ParticleRequest>) {
        for req in requests {
            self.requests.push(req);
        }
    }
}