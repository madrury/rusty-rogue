use rltk::{
    BLUE, BROWN1, CRIMSON, DARKGRAY, DARKGREEN, GREEN, LIGHTBLUE, LIGHTGRAY, MEDIUMBLUE, ORANGE, RED, RED1, RED2, RED4, RGB, ROSY_BROWN, SANDY_BROWN, SILVER, TOMATO, WHITE, YELLOW
};
use serde::{Serialize, Deserialize};
use super::{NoiseMaps, Map};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum FgColorMap {
    None, ShortGrass, LongGrass, ShallowWater, DeepWater, Fire, Chill, Steam, Blood
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum BgColorMap {
    None, ShallowWater, DeepWater, Fire, Chill, Blood
}

//----------------------------------------------------------------------------
// ColorMaps.
//----------------------------------------------------------------------------
// Encapsulates smoothly varying random colors used during rendering to display
// entities with the UseFgColorMap or UseBgColorMap components during rendering.
// When entities have one of these compoenents, we dynamically look up the value
// in one of the consituent colormaps for the tile that they occupy, and color
// the entity accordingly. This is most useful for immobile entities whose
// display is modified by certain effects.
//----------------------------------------------------------------------------
pub struct ColorMaps {
    short_grass_fg: Vec<RGB>,
    long_grass_fg: Vec<RGB>,
    shallow_water_fg: Vec<RGB>,
    shallow_water_bg: Vec<RGB>,
    deep_water_fg: Vec<RGB>,
    deep_water_bg: Vec<RGB>,
    fire_fg: Vec<RGB>,
    fire_bg: Vec<RGB>,
    chill_fg: Vec<RGB>,
    chill_bg: Vec<RGB>,
    steam_fg: Vec<RGB>,
    blood_fg: Vec<RGB>,
    blood_bg: Vec<RGB>,
}
impl ColorMaps {
    pub fn from_noisemap(nm: &NoiseMaps, map: &Map) -> Self {
        return ColorMaps {
            short_grass_fg: nm.to_short_grass_fg_color_noise(map)
                .into_iter().map(grass_green_from_noise).collect(),
            long_grass_fg: nm.to_long_grass_fg_color_noise(map)
                .into_iter().map(grass_green_from_noise).collect(),
            shallow_water_fg: nm.to_shallow_water_fg_color_noise(map)
                .into_iter().map(water_fg_from_noise).collect(),
            shallow_water_bg: nm.to_shallow_water_bg_color_noise(map)
                .into_iter().map(shallow_water_bg_from_noise).collect(),
            deep_water_fg: nm.to_deep_water_fg_color_noise(map)
                .into_iter().map(water_fg_from_noise).collect(),
            deep_water_bg: nm.to_deep_water_bg_color_noise(map)
                .into_iter().map(water_bg_from_noise).collect(),
            fire_fg: nm.to_fire_fg_color_noise(map)
                .into_iter().map(fire_fg_from_noise).collect(),
            fire_bg: nm.to_fire_bg_color_noise(map)
                .into_iter().map(fire_bg_from_noise).collect(),
            chill_fg: nm.to_chill_fg_color_noise(map)
                .into_iter().map(chill_fg_from_noise).collect(),
            chill_bg: nm.to_chill_bg_color_noise(map)
                .into_iter().map(chill_bg_from_noise).collect(),
            steam_fg: nm.to_steam_fg_color_noise(map)
                .into_iter().map(steam_fg_from_noise).collect(),
            blood_fg: nm.to_blood_fg_color_noise(map)
                .into_iter().map(blood_fg_from_noise).collect(),
            blood_bg: nm.to_blood_bg_color_noise(map)
                .into_iter().map(blood_bg_from_noise).collect(),
        }
    }

    pub fn get_fg_color(&self, idx: usize, cmap: FgColorMap) -> RGB {
        match cmap {
            FgColorMap::ShortGrass => self.short_grass_fg[idx],
            FgColorMap::LongGrass =>  self.long_grass_fg[idx],
            FgColorMap::ShallowWater => self.shallow_water_fg[idx],
            FgColorMap::DeepWater => self.deep_water_fg[idx],
            FgColorMap::Fire => self.fire_fg[idx],
            FgColorMap::Chill => self.chill_fg[idx],
            FgColorMap::Steam => self.steam_fg[idx],
            FgColorMap::Blood => self.blood_fg[idx],
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
            BgColorMap::Chill => self.chill_bg[idx],
            BgColorMap::Blood => self.blood_bg[idx],
            // This should never be reached. We pick a color that likely makes
            // it obvious.
            BgColorMap::None => RGB::named(rltk::HOTPINK)
        }
    }
}

pub fn interpolate_colors(f: f32, lc: (u8, u8, u8), hc: (u8, u8, u8)) -> RGB {
    RGB::from_u8(
        ((1.0 - f) * (lc.0 as f32)) as u8 + (f * (hc.0 as f32)) as u8,
        ((1.0 - f) * (lc.1 as f32)) as u8 + (f * (hc.1 as f32)) as u8,
        ((1.0 - f) * (lc.2 as f32)) as u8 + (f * (hc.2 as f32)) as u8
    )
}

pub fn grass_green_from_noise(f: f32) -> RGB {
    interpolate_colors(f, DARKGREEN, GREEN)
}

pub fn water_fg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, WHITE, BLUE)
}

pub fn water_bg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, BLUE, MEDIUMBLUE)
}

pub fn shallow_water_bg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, SANDY_BROWN, BLUE)
}

pub fn fire_fg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, RED, YELLOW)
}

pub fn fire_bg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, ORANGE, RED)
}

pub fn chill_fg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, SILVER, LIGHTGRAY)
}

pub fn chill_bg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, WHITE, LIGHTBLUE)
}

pub fn steam_fg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, WHITE, DARKGRAY)
}

pub fn blood_fg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, RED, DARKGREEN)
}

pub fn blood_bg_from_noise(f: f32) -> RGB {
    interpolate_colors(f, RED4, BLUE)
}