use super::{Map, AnimationParticleBuffer, AnimationSequenceBuffer};
use specs::prelude::*;


// Transform sequences of AnimationBlocks into AnimationParticles.
pub struct AnimationParticlizationSystem {}

impl<'a> System<'a> for AnimationParticlizationSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteExpect<'a, AnimationSequenceBuffer>,
        WriteExpect<'a, AnimationParticleBuffer>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut sequence_buffer, mut particle_buffer) = data;

        while !sequence_buffer.is_empty() {
            let sequence = sequence_buffer.pop();
            match sequence {
                Some(mut seq) => {
                    let particles = seq.particalize(&map);
                    particle_buffer.request_many(particles);
                }
                None => { continue; }
            }
        }
    }
}
