use super::{AnimationParticle, AnimationParticleBuffer, Position, Renderable};
use rltk::{Rltk, RGB};
use specs::prelude::*;

//----------------------------------------------------------------------------
// Systems for handing particle effects.
//
// The code here handles particle effects, the atomic units of a game animation.
// Each particle is a renderable entity with two additional parameters:
//
//    - delay: How long (in ms) until this particle should be rendered.
//    - lifetime: Once rendered, how long until this particle should be removed.
//
// Particles are requested by adding a ParticleRequest object to the
// ParticleBuilder queue (implemented as a struct wrapping a Vec), which is
// asseccisble for writing to all game enetities. Once in the queue, a
// ParticleInitSystem adds ParticleLifetime entities to the ECS. Once a tick,
// the lifetimes of these particles are updated, and a Particle render system
// checks the delay and lifetime parameters to see if the particle should be
// rendered.
//----------------------------------------------------------------------------


//----------------------------------------------------------------------------
// A system responsible for monitoring the ParticleBuilder contianer and
// actually adding any requested particles (represented as ParticleLifetime
// objects) to the ECS.
//----------------------------------------------------------------------------
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

// A system responsible for actually rendering any particles whose delay and
// lifetime parameters are in the correct state.
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

// Function responsible for monitoring all particles currently in the ECS, and
// updating their delay and lifetime parameters according to how much time was
// spent in the previous game tick. Additionally cleans up expired particles.
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
