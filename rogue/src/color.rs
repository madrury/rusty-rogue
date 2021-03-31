
use rltk::{RGB, GREEN, DARKGREEN};

pub fn grass_green_from_noise(f: f32) -> RGB {
    RGB::from_u8(
        ((1.0 - f) * (DARKGREEN.0 as f32)) as u8 + (f * (GREEN.0 as f32)) as u8,
        ((1.0 - f) * (DARKGREEN.1 as f32)) as u8 + (f * (GREEN.1 as f32)) as u8,
        ((1.0 - f) * (DARKGREEN.2 as f32)) as u8 + (f * (GREEN.2 as f32)) as u8
    )
}