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

pub fn statue_noisemap(map: &Map) -> Vec<f32> {
    let mut rng = RandomNumberGenerator::new();

    let mut whitenoise = FastNoise::seeded(rng.next_u64());
    whitenoise.set_noise_type(NoiseType::WhiteNoise);
    whitenoise.set_frequency(0.5);

    let mut noisemap: Vec<f32> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let wn = whitenoise.get_noise(x as f32, y as f32);
            noisemap.push(wn);
        }
    }
    noisemap
}

pub fn monster_spawn_locations(map: &Map) -> Vec::<(i32, i32)> {
    let mut rng = RandomNumberGenerator::new();

    let mut valuenoise = FastNoise::seeded(rng.next_u64());
    valuenoise.set_noise_type(NoiseType::Value);
    valuenoise.set_frequency(0.25);

    let mut whitenoise = FastNoise::seeded(rng.next_u64());
    whitenoise.set_noise_type(NoiseType::WhiteNoise);
    whitenoise.set_frequency(0.5);

    let mut points: Vec::<((i32, i32), f32)> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let wn = whitenoise.get_noise(x as f32, y as f32);
            let vn = valuenoise.get_noise(x as f32, y as f32);
            points.push(((x, y), wn + vn));
        }
    }
    points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    points.iter().map(|t| t.0).collect()
}