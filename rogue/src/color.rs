
use rltk::{RGB, GREEN, DARKGREEN, BLUE, WHITE, MEDIUMBLUE, SANDY_BROWN};


pub fn grass_green_from_noise(f: f32) -> RGB {
    RGB::from_u8(
        ((1.0 - f) * (DARKGREEN.0 as f32)) as u8 + (f * (GREEN.0 as f32)) as u8,
        ((1.0 - f) * (DARKGREEN.1 as f32)) as u8 + (f * (GREEN.1 as f32)) as u8,
        ((1.0 - f) * (DARKGREEN.2 as f32)) as u8 + (f * (GREEN.2 as f32)) as u8
    )
}

pub fn water_fg_from_noise(f: f32) -> RGB {
    RGB::from_u8(
        (f * (BLUE.0 as f32)) as u8 + ((1.0 - f) * (WHITE.0 as f32)) as u8,
        (f * (BLUE.1 as f32)) as u8 + ((1.0 - f) * (WHITE.1 as f32)) as u8,
        (f * (BLUE.2 as f32)) as u8 + ((1.0 - f) * (WHITE.2 as f32)) as u8
    )
}

pub fn water_bg_from_noise(f: f32) -> RGB {
    RGB::from_u8(
        (f * (MEDIUMBLUE.0 as f32)) as u8 + ((1.0 - f) * (BLUE.0 as f32)) as u8,
        (f * (MEDIUMBLUE.1 as f32)) as u8 + ((1.0 - f) * (BLUE.1 as f32)) as u8,
        (f * (MEDIUMBLUE.2 as f32)) as u8 + ((1.0 - f) * (BLUE.2 as f32)) as u8
    )
}

pub fn shallow_water_bg_from_noise(f: f32) -> RGB {
    RGB::from_u8(
        (f * (BLUE.0 as f32)) as u8 + ((1.0 - f) * (SANDY_BROWN.0 as f32)) as u8,
        (f * (BLUE.1 as f32)) as u8 + ((1.0 - f) * (SANDY_BROWN.1 as f32)) as u8,
        (f * (BLUE.2 as f32)) as u8 + ((1.0 - f) * (SANDY_BROWN.2 as f32)) as u8
    )
}