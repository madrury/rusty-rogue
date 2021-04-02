use rltk::{RandomNumberGenerator, FastNoise, NoiseType};
use super::{Map};


pub fn grass_noisemap(map: &Map) -> Vec<(f32, f32)> {
    let mut rng = RandomNumberGenerator::new();

    let mut valuenoise = FastNoise::seeded(rng.next_u64());
    valuenoise.set_noise_type(NoiseType::ValueFractal);
    valuenoise.set_frequency(0.1);

    let mut whitenoise = FastNoise::seeded(rng.next_u64());
    whitenoise.set_noise_type(NoiseType::WhiteNoise);
    whitenoise.set_frequency(0.1);

    let mut noisemap: Vec<(f32, f32)> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let vn = valuenoise.get_noise(x as f32, y as f32);
            let wn = whitenoise.get_noise(x as f32, y as f32);
            noisemap.push((vn, wn))
        }
    }
    noisemap
}

pub fn water_noisemap(map: &Map, vfreq: f32) -> Vec<(f32, f32)> {
    let mut rng = RandomNumberGenerator::new();

    let mut valuenoise = FastNoise::seeded(rng.next_u64());
    valuenoise.set_noise_type(NoiseType::Value);
    valuenoise.set_frequency(vfreq);

    let mut whitenoise = FastNoise::seeded(rng.next_u64());
    whitenoise.set_noise_type(NoiseType::WhiteNoise);
    whitenoise.set_frequency(0.7);

    let mut noisemap: Vec<(f32, f32)> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let vn = valuenoise.get_noise(x as f32, y as f32);
            let wn = whitenoise.get_noise(x as f32, y as f32);
            noisemap.push((vn, wn))
        }
    }
    noisemap
}