use super::{Map, AnimationParticleBuffer, AnimationSequenceBuffer};
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

// Handles grabbing AnimationRequests from the AnimationBuilder buffer, and
// converting them into the individual constituent ParticleRequests.
pub struct AnimationInitSystem {}

impl<'a> System<'a> for AnimationInitSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteExpect<'a, AnimationSequenceBuffer>,
        WriteExpect<'a, AnimationParticleBuffer>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut sequence_buffer, mut particle_buffer) = data;

        while !sequence_buffer.is_empty() {
            let mut sequence = sequence_buffer.pop();
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
