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

// Monitor all particles currently in the ECS, update their delay and lifetime
// state according to how much time was spent in the previous game frame.
// When a particle's delay first drops to zero, set the displayed flag.
// When a particle's lifetime first drops to zero, remove it from teh ECS.
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
                if particle.delay <= 0.0 {
                    particle.displayed = true
                }
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
