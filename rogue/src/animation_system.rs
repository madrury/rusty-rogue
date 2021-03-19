use super::{Map, ParticleBuilder, ParticleRequest};
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
        x: i32,
        y: i32,
        bg: RGB,
        glyph: rltk::FontCharType,
    },
    Healing {
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
    },
    AreaOfEffect {
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
        radius: i32,
    },
    AlongRay {
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
    },
    Teleportation {
        x: i32,
        y: i32,
        bg: RGB,
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

// Handles grabbing AnimationRequests from the AnimationBuilder buffer, and
// converting them into the individual constituent ParticleRequests.
pub struct AnimationInitSystem {}

impl<'a> System<'a> for AnimationInitSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteExpect<'a, AnimationBuilder>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut animation_builder, mut particle_builder) = data;
        for request in animation_builder.requests.iter() {
            particle_builder.request_many(match request {
                AnimationRequest::MeleeAttack {x, y, bg, glyph} => {
                    make_melee_animation(*x, *y, *bg, *glyph)
                }
                AnimationRequest::Healing {
                    x,
                    y,
                    fg,
                    bg,
                    glyph,
                } => make_healing_animation(*x, *y, *fg, *bg, *glyph),
                AnimationRequest::AreaOfEffect {
                    x,
                    y,
                    fg,
                    bg,
                    glyph,
                    radius,
                } => make_aoe_animation(&*map, *x, *y, *fg, *bg, *glyph, *radius),
                AnimationRequest::AlongRay {
                    source_x,
                    source_y,
                    target_x,
                    target_y,
                    fg,
                    bg,
                    glyph,
                } => make_along_ray_animation(
                    &*map,
                    Point {x: *source_x, y: *source_y},
                    Point {x: *target_x, y: *target_y},
                    *fg,
                    *bg,
                    *glyph
                ),
                AnimationRequest::Teleportation {x, y, bg}
                    => make_teleportation_animation(*x, *y, *bg),
            })
        }
        animation_builder.requests.clear();
    }
}

// A melee attack animation. The targeted entity flashes briefly.
fn make_melee_animation(
    x: i32,
    y: i32,
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
            x: x,
            y: y,
            fg: *color,
            bg: bg,
            glyph: glyph,
            lifetime: 50.0, // ms
            delay: 50.0 * (i as f32),
        })
    }
    particles
}

// A healing animation. A heart flashes quickly.
fn make_healing_animation(
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let color_cycle = [rltk::RGB::named(rltk::RED), fg, rltk::RGB::named(rltk::RED)];
    let glyph_cycle = [rltk::to_cp437('♥'), glyph, rltk::to_cp437('♥')];
    for (i, (color, glyph)) in color_cycle.iter().zip(glyph_cycle.iter()).enumerate() {
        particles.push(ParticleRequest {
            x: x,
            y: y,
            fg: *color,
            bg: bg,
            glyph: *glyph,
            lifetime: 100.0, // ms
            delay: 100.0 * (i as f32),
        })
    }
    particles
}

// An aoe animation. Color cycles glyphs in a circle around a point.
fn make_aoe_animation(
    map: &Map,
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    radius: i32,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let fg_color_cycle = [fg, bg, fg];
    let bg_color_cycle = [bg, fg, bg];
    let mut blast_tiles = rltk::field_of_view(Point { x, y }, radius, &*map);
    blast_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
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
    glyph: rltk::FontCharType
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let tiles = map.get_ray_tiles(source, target);
    for (i, tile) in tiles.into_iter().enumerate() {
        particles.push(ParticleRequest {
            x: tile.x,
            y: tile.y,
            fg: fg,
            bg: bg,
            glyph: glyph,
            lifetime: 50.0,
            delay: 50.0 * (i as f32)
        })
    }
    particles
}

// A healing animation. A heart flashes quickly.
fn make_teleportation_animation(
    x: i32,
    y: i32,
    bg: RGB,
) -> Vec<ParticleRequest> {
    let mut particles = Vec::new();
    let glyph_cycle = [rltk::to_cp437('░'), rltk::to_cp437(' '), rltk::to_cp437('░')];
    for (i, glyph) in glyph_cycle.iter().enumerate() {
        particles.push(ParticleRequest {
            x: x,
            y: y,
            fg: rltk::RGB::named(rltk::MEDIUM_PURPLE),
            bg: bg,
            glyph: *glyph,
            lifetime: 100.0, // ms
            delay: 100.0 * (i as f32),
        })
    }
    particles
}