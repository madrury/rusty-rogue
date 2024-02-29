use specs::prelude::*;
use specs_derive::*;
use serde::{Serialize, Deserialize};


#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TargetingKind {
    Simple,
    AreaOfEffect {radius: f32},
    AlongRay {until_blocked: bool}
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum TargetingVerb {
    Throw,
    Cast
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct TargetedWhenThrown {
    pub range: f32,
    pub kind: TargetingKind
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct TargetedWhenCast {
    pub range: f32,
    pub kind: TargetingKind
}