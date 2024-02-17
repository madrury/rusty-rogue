use rltk::{BLUE, DARKGREEN, GREEN, MEDIUMBLUE, ORANGE, RED, RGB, SANDY_BROWN, WHITE, YELLOW};
use serde::{Serialize, Deserialize};

use super::{NoiseMaps, Map};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum FgColorMap {
    None, ShortGrass, LongGrass, ShallowWater, DeepWater, Fire
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum BgColorMap {
    None, ShallowWater, DeepWater, Fire
}

pub struct ColorMaps {
    short_grass_fg: Vec<RGB>,
    long_grass_fg: Vec<RGB>,
    shallow_water_fg: Vec<RGB>,
    shallow_water_bg: Vec<RGB>,
    deep_water_fg: Vec<RGB>,
    deep_water_bg: Vec<RGB>,
    fire_fg: Vec<RGB>,
    fire_bg: Vec<RGB>,
}
impl ColorMaps {
    pub fn from_noisemap(nm: &NoiseMaps, map: &Map) -> Self {
        return ColorMaps {
            short_grass_fg: nm.to_short_grass_fg_colormap(map),
            long_grass_fg: nm.to_long_grass_fg_colormap(map),
            shallow_water_fg: nm.to_shallow_water_fg_colormap(map),
            shallow_water_bg: nm.to_shallow_water_bg_colormap(map),
            deep_water_fg: nm.to_deep_water_fg_colormap(map),
            deep_water_bg: nm.to_deep_water_bg_colormap(map),
            fire_fg: nm.to_fire_fg_colormap(map),
            fire_bg: nm.to_fire_bg_colormap(map),
        }
    }

    pub fn get_fg_color(&self, idx: usize, cmap: FgColorMap) -> RGB {
        match cmap {
            FgColorMap::ShortGrass => self.short_grass_fg[idx],
            FgColorMap::LongGrass =>  self.long_grass_fg[idx],
            FgColorMap::ShallowWater => self.shallow_water_fg[idx],
            FgColorMap::DeepWater => self.deep_water_fg[idx],
            FgColorMap::Fire => self.fire_fg[idx],
            // This should never be reached. We pick a color that likely makes
            // it obvious.
            FgColorMap::None => RGB::named(rltk::HOTPINK)
        }
    }

    pub fn get_bg_color(&self, idx: usize, cmap: BgColorMap) -> RGB {
        match cmap {
            BgColorMap::ShallowWater => self.shallow_water_bg[idx],
            BgColorMap::DeepWater => self.deep_water_bg[idx],
            BgColorMap::Fire => self.fire_bg[idx],
            // This should never be reached. We pick a color that likely makes
            // it obvious.
            BgColorMap::None => RGB::named(rltk::HOTPINK)
        }
    }
}

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

pub fn fire_fg_from_noise(f: f32) -> RGB {
    RGB::from_u8(
        (f * (YELLOW.0 as f32)) as u8 + ((1.0 - f) * (RED.0 as f32)) as u8,
        (f * (YELLOW.1 as f32)) as u8 + ((1.0 - f) * (RED.1 as f32)) as u8,
        (f * (YELLOW.2 as f32)) as u8 + ((1.0 - f) * (RED.2 as f32)) as u8
    )
}

pub fn fire_bg_from_noise(f: f32) -> RGB {
    RGB::from_u8(
        (f * (ORANGE.0 as f32)) as u8 + ((1.0 - f) * (RED.0 as f32)) as u8,
        (f * (ORANGE.1 as f32)) as u8 + ((1.0 - f) * (RED.1 as f32)) as u8,
        (f * (ORANGE.2 as f32)) as u8 + ((1.0 - f) * (RED.2 as f32)) as u8
    )
}