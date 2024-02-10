use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::RGB;

//------------------------------------------------------------------
// Components and data structures for ggame animations.
//
// Game animations are constructed using GameAnimationParticle structs, which
// are renderable (though not in the literal sense that they have a Renderable
// component, they are not eneitys) objects that live for a specified amount of
// time. Game animations are build from these objects by displaying them for a
// given amount of time, and then removing them. See animation_system.rs and
// particle_system.rs for the business logic of handline game animation requests
// and rendering.
//------------------------------------------------------------------

// Represents an atomic piece of game animation.
#[derive(Component)]
pub struct GameAnimationParticle {
    pub lifetime : f32,
    // How many milliseconds after the animation starts should this particle be
    // displayed?
    pub delay: f32,
    // For how long should this particle be displayed?
    pub displayed: bool,
    pub x: i32,
    pub y: i32,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType
}

// Component for effects that create an area of effect animation when
// used as a targeted effect.
#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffectAnimationWhenTargeted {
    pub radius: i32,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType
}

// Component for effects that create an animation along a ray (so, say a
// fireball) when used as a targeted effect.
#[derive(Component, ConvertSaveload, Clone)]
pub struct AlongRayAnimationWhenTargeted {
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType,
    pub until_blocked: bool
}