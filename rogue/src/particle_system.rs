use super::{AnimationParticle, AnimationParticleBuffer, Position, Renderable};
use rltk::{Rltk, RGB};
use specs::prelude::*;



// Empty the AnimationParticleBuffer, create entities and insert into the ECS.
pub struct ParticleInitSystem {}

impl<'a> System<'a> for ParticleInitSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, AnimationParticle>,
        WriteExpect<'a, AnimationParticleBuffer>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut particles, mut particle_buffer) = data;
        while !particle_buffer.is_empty() {
            let particle = particle_buffer.pop();
            match particle {
                Some(particle) => {
                    let p = entities.create();
                    particles.insert(p, particle).expect("Unable to insert new particle.");
                }
                None => { continue; }
            }
        }
    }
}

// Attach Position and Renderable components to particle entities once the
// rendering conditions have been met.
pub struct ParticleRenderSystem {}

impl<'a> System<'a> for ParticleRenderSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, AnimationParticle>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut particles) = data;
        for (entity, p) in (&entities, &mut particles).join() {
            if p.delay <= 0.0 && !p.displayed {
                positions
                    .insert(entity, Position { x: p.pt.x, y: p.pt.y })
                    .expect("Unable to insert particle position.");
                renderables
                    .insert(
                        entity,
                        Renderable {
                            fg: p.fg,
                            bg: p.bg,
                            glyph: p.glyph,
                            order: 0,
                            visible_out_of_fov: false
                        },
                    )
                    .expect("Unable to insert particle renderable.");
                p.displayed = true;
            }
        }
    }
}

// Monitor all particles currently in the ECS, updatie their delay and lifetime
// state according to how much time was spent in the previous game frame.
// Cleans up expired particles when their lifetime is expired..
pub fn update_particle_lifetimes(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles: Vec<Entity> = Vec::new();
    {
        let mut particles = ecs.write_storage::<AnimationParticle>();
        let entities = ecs.entities();
        for (entity, particle) in (&entities, &mut particles).join() {
            if particle.lifetime <= 0.0 {
                dead_particles.push(entity);
            } else if particle.delay <= 0.0 {
                particle.lifetime = f32::max(0.0, particle.lifetime - ctx.frame_time_ms);
            } else {
                particle.delay = f32::max(0.0, particle.delay - ctx.frame_time_ms);
            }
        }
    }
    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Particle will not die.");
    }
}

pub fn is_any_animation_alive(ecs: &World) -> bool {
    let particles = ecs.read_storage::<AnimationParticle>();
    particles.join().next().is_some()
}
