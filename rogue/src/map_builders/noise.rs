use rltk::{RandomNumberGenerator, FastNoise, NoiseType, Point, RGB};
use super::{Map, TileType, color};


const LARGE_LAKES_SHALLOW_WATER_THRESHOLD: f32 = 0.4;
const LARGE_LAKES_DEEP_WATER_THRESHOLD: f32 = 0.6;
const LARGE_LAKES_NOISE_FREQUENCY: f32 = 0.1;

const SMALL_LAKES_SHALLOW_WATER_THRESHOLD: f32 = 0.6;
const SMALL_LAKES_DEEP_WATER_THRESHOLD: f32 = 0.7;
const SMALL_LAKES_NOISE_FREQUENCY: f32 = 0.25;

const SHORT_GRASS_NOISE_THRESHOLD: f32 = 0.0;
const SPORADIC_TALL_GRASS_NOISE_THRESHOLD: f32 = 0.8;
const GROVE_TALL_GRASS_NOISE_THRESHOLD: f32 = 0.4;

const STATUE_NOISE_THRESHOLD: f32 = 0.98;

pub enum WaterGeometry { None, LargeLakes, SmallLakes }
impl WaterGeometry {
    pub fn random(rng: &mut RandomNumberGenerator) -> Self {
        match rng.roll_dice(1, 3) {
            1 => WaterGeometry::None,
            2 => WaterGeometry::LargeLakes,
            3 => WaterGeometry::SmallLakes,
            _ => panic!("Rolled too high on water spawning.")
        }
    }
}
pub struct WaterSpawnData {
    pub x: i32,
    pub y: i32,
    pub fgcolor: RGB,
    pub bgcolor: RGB
}
pub struct WaterSpawnTable {
    pub shallow: Vec<WaterSpawnData>,
    pub deep: Vec<WaterSpawnData>
}

impl WaterSpawnTable {
    pub fn new() -> Self {
        WaterSpawnTable {
            shallow: Vec::<WaterSpawnData>::new(),
            deep: Vec::<WaterSpawnData>::new(),
        }
    }
}

pub enum GrassGeometry { None, OnlyShort, SporadicTall, GroveTall }
impl GrassGeometry {
    pub fn random(rng: &mut RandomNumberGenerator) -> Self {
        match rng.roll_dice(1, 5) {
            // Not including GrassGeometry::None for now, None is boring with
            // our limited terrain types.
            1 => GrassGeometry::OnlyShort,
            2 | 3 => GrassGeometry::SporadicTall,
            4 | 5 => GrassGeometry::GroveTall,
            _ => panic!("Rolled to high on grass spawning.")
        }
    }
}
pub struct GrassSpawnData {
    pub x: i32,
    pub y: i32,
    pub fgcolor: RGB,
}
pub struct GrassSpawnTable {
    pub short: Vec<GrassSpawnData>,
    pub long: Vec<GrassSpawnData>
}
impl GrassSpawnTable {
    pub fn new() -> Self {
        GrassSpawnTable {
            short: Vec::<GrassSpawnData>::new(),
            long: Vec::<GrassSpawnData>::new(),
        }
    }
}

pub enum StatueGeometry { None, Some }
impl StatueGeometry {
    pub fn random(rng: &mut RandomNumberGenerator) -> Self {
        match rng.roll_dice(1, 2) {
            1 => StatueGeometry::None,
            2 => StatueGeometry::Some,
            _ => panic!("Rolled to high on water spawning.")
        }
    }
}
pub struct StatueSpawnData {
    pub x: i32,
    pub y: i32,
}


//----------------------------------------------------------------------------
// NoiseMaps
//
// Encapsulating objects for random geometric structures, which we refer to
// collectively as noise-maps. These have various applications:
//
// - Various dungeon terrain like grass and water should be spawned in smooth
// contiguous reigons,
// - Colorization on dungeon terrain should vary smoothly with some variation,
// it looks nice!
// - Spawning of monsters should cluster together in groups to provide engaging
// combat experiences.
//----------------------------------------------------------------------------
// Public API:
// There are two types of functions eposed in the public NoiseMaps AIP.
//
// - to_<terraintype>_spawn_table:
// Returns a SpawnTable type obejct for terraintype (GrassSpawnTable,
// WaterSpawnTable, ... for example). These encaspulate a sequence of
// instructions for spawning the required entities into the ECS to construct
// terrain of the given type.
//
// - <monstertype>_monster_spawn_buffer
// Returns a Vec<Point>, sorted in order that we should attempt to spawn a
// monster. There are multiple of these methods and the client is reposible for
// depulicating the streams (yo! But they probably shouldnt be, you could do
// this with some internal state on this thing).
//----------------------------------------------------------------------------
pub struct NoiseMaps {
    // Noisemap for sampling monster spawn positions.
    // Spawn locations are determined by sorting this vector by noise value, and
    // poping off the points with the largest noise values. The noise value is a
    // sum of smooth and white noise, the smooth allows them to cluster and the
    // white noise breaks up the clusters a little.
    spawning: Vec<(Point, f32)>,
    // The geometric structure of grass spawned on the map.
    pub grass_geometry: GrassGeometry,
    // Noisemap for sampling grass spawn positions and colorization.
    // The noise value is a pair (smooth, white) noise. Grass spawn positions
    // are determined by thresholding either the smooth or white component
    // (depending on desired geometry), above the threshold produces tall grass,
    // below produces short grass. Colorization is determined as a linear
    // combination of the two components.
    grass: Vec<(Point, (f32, f32))>,
    // The geometric structure of pools/lakes of water spawned on the map.
    pub water_geometry: WaterGeometry,
    // Noisemap for sampling water spawn positions and colorization.
    // The noise value is a pair (smooth, white) noise. Water spawn positions
    // are determined by thresholding the smooth component, a larger threshold
    // produces deep water, and slighly lesser threshold produces shallower;
    // this tends to produce pools of deep water ringed by shallow. Colorization
    // is determined as a linear combination of the two components.
    water: Vec<(Point, (f32, f32))>,
    // The geometric structure of statues spawned on the map.
    pub statue_geometry: StatueGeometry,
    // Noisemap for sampling statue spawn locations.
    // The noise value is a pair (smooth, white) noise. Water spawn positions
    // are determined by thresholding the smooth component, a larger threshold
    // produces deep water, and slighly lesser threshold produces shallower;
    // this tends to produce pools of deep water ringed by shallow. Colorization
    // is determined as a linear combination of the two components.
    statue: Vec<(Point, f32)>,
    // The possition of the blessing tile.
    pub blessing: Option<Point>,
}

impl NoiseMaps {

    pub fn random(rng: &mut RandomNumberGenerator, map: &Map) -> Self {
        // Roll for the type of geometry for each of our terrain features.
        let watergeom = WaterGeometry::random(rng);
        let watervfreq = match watergeom {
            WaterGeometry::None => 1.0,
            WaterGeometry::LargeLakes => LARGE_LAKES_NOISE_FREQUENCY,
            WaterGeometry::SmallLakes => SMALL_LAKES_NOISE_FREQUENCY,
        };
        let grassgeom = GrassGeometry::random(rng);
        let statuegeom = StatueGeometry::random(rng);
        // Determine the blessing location. Nothing fancy.
        let blessingloc = map.random_unblocked_point(100, rng);
        let blessingpt = match blessingloc {
            None => None,
            Some(loc) => Some(Point {x: loc.0, y: loc.1}),
        };
        NoiseMaps {
            water_geometry: watergeom,
            grass_geometry: grassgeom,
            statue_geometry: statuegeom,
            blessing: blessingpt,
            // Sample the noisemaps.
            spawning: monster_spawn_noisemap(rng, map),
            grass: grass_noisemap(rng, map),
            water: water_noisemap(rng, map, watervfreq),
            statue: statue_noisemap(rng, map),
        }
    }

    pub fn general_monster_spawn_position_buffer(&self) -> Vec<Point> {
        let mut buffer = self.spawning.clone();
        buffer.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        buffer.iter().map(|(pt, _)| *pt).collect()
    }

    pub fn grassbound_monster_spawn_position_buffer(&self) -> Vec<Point> {
        let mut buffer = self.grass.clone();
        buffer.sort_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap());
        buffer.iter().map(|(pt, _)| *pt).collect()
    }

    pub fn waterbound_monster_spawn_position_buffer(&self) -> Vec<Point> {
        let mut buffer = self.water.clone();
        buffer.sort_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap());
        buffer.iter().map(|(pt, _)| *pt).collect()
    }

    fn deep_water_noise_threshold(&self) -> f32 {
        match self.water_geometry {
            WaterGeometry::None => f32::MAX,
            WaterGeometry::LargeLakes => LARGE_LAKES_DEEP_WATER_THRESHOLD,
            WaterGeometry::SmallLakes => SMALL_LAKES_DEEP_WATER_THRESHOLD
        }
    }

    fn shallow_water_noise_threshold(&self) -> f32 {
        match self.water_geometry {
            WaterGeometry::None => f32::MAX,
            WaterGeometry::LargeLakes => LARGE_LAKES_SHALLOW_WATER_THRESHOLD,
            WaterGeometry::SmallLakes => SMALL_LAKES_SHALLOW_WATER_THRESHOLD
        }
    }

    pub fn to_water_spawn_table(&self, map: &Map) -> WaterSpawnTable {
        let mut water_spawn_table = WaterSpawnTable::new();
        let mut blessing_adjacent_tiles: Vec<Point> = Vec::new();
        if let Some(blessing) = self.blessing {
            blessing_adjacent_tiles = map.get_adjacent_tiles(blessing.x, blessing.y)
                .iter()
                .map(|t| Point::from_tuple(*t))
                .collect();
        }
        for (pt, (vnoise, wnoise)) in self.water.iter() {
            let idx = map.xy_idx(pt.x, pt.y);
            if map.is_edge_tile(pt.x, pt.y) {continue;}
            if map.tiles[idx] == TileType::DownStairs {continue;}
            if *vnoise > self.deep_water_noise_threshold() || blessing_adjacent_tiles.contains(pt) {
                let colorseeds = (vnoise + 0.6, 0.7 * vnoise + 0.2 * wnoise + 0.4);
                let fgcolor = color::water_fg_from_noise(colorseeds.0);
                let bgcolor = color::water_bg_from_noise(colorseeds.1);
                water_spawn_table.deep.push(WaterSpawnData {
                    x: pt.x, y: pt.y, fgcolor: fgcolor, bgcolor: bgcolor
                })
            } else if *vnoise > self.shallow_water_noise_threshold() {
                let colorseeds = (vnoise + 0.4, 0.5 * vnoise + 0.1 * wnoise + 0.4);
                let fgcolor = color::water_fg_from_noise(colorseeds.0);
                let bgcolor = color::shallow_water_bg_from_noise(colorseeds.1);
                water_spawn_table.shallow.push(WaterSpawnData {
                    x: pt.x, y: pt.y, fgcolor: fgcolor, bgcolor: bgcolor
                })
            }
        }
        water_spawn_table
    }

    fn short_grass_noise_threshold(&self) -> f32 {
        SHORT_GRASS_NOISE_THRESHOLD
    }

    fn long_grass_noise_threshold(&self) -> f32 {
        match self.grass_geometry {
            GrassGeometry::None => f32::MAX,
            GrassGeometry::OnlyShort => f32::MAX,
            GrassGeometry::SporadicTall => SPORADIC_TALL_GRASS_NOISE_THRESHOLD,
            GrassGeometry::GroveTall => GROVE_TALL_GRASS_NOISE_THRESHOLD,
        }
    }

    pub fn to_grass_spawn_table(&self, map: &Map) -> GrassSpawnTable {
        let mut grass_spawn_table = GrassSpawnTable::new();
        for (pt, (vnoise, wnoise)) in self.grass.iter() {
            let idx = map.xy_idx(pt.x, pt.y);
            if map.is_edge_tile(pt.x, pt.y) {continue;}
            if map.tiles[idx] == TileType::DownStairs {continue;}
            if *vnoise > self.short_grass_noise_threshold() && map.ok_to_spawn[idx] {
                let long_grass_noise = match self.grass_geometry {
                    GrassGeometry::SporadicTall => wnoise,
                    _ => vnoise
                };
                if *long_grass_noise > self.long_grass_noise_threshold() {
                    let colorseed = vnoise + 0.3 * wnoise;
                    let gcolor = color::grass_green_from_noise(colorseed);
                    grass_spawn_table.long.push(GrassSpawnData {x: pt.x, y: pt.y, fgcolor: gcolor})
                } else {
                    let colorseed = vnoise + 0.3 * wnoise + 0.6;
                    let gcolor = color::grass_green_from_noise(colorseed);
                    grass_spawn_table.short.push(GrassSpawnData {x: pt.x, y: pt.y, fgcolor: gcolor})
                }
            }
        }
        grass_spawn_table
    }

    pub fn to_statue_spawn_table(&self, map: &Map) -> Vec<StatueSpawnData> {
        let mut statue_spawn_table = Vec::<StatueSpawnData>::new();
        for (pt, wnoise) in self.statue.iter() {
            let idx = map.xy_idx(pt.x, pt.y);
            if map.is_edge_tile(pt.x, pt.y) {continue;}
            if map.tiles[idx] == TileType::DownStairs {continue;}
            if *wnoise > STATUE_NOISE_THRESHOLD && map.ok_to_spawn[idx] {
                statue_spawn_table.push(StatueSpawnData {x: pt.x, y: pt.y})
            }
        }
        statue_spawn_table
    }
}


fn grass_noisemap(rng: &mut RandomNumberGenerator, map: &Map) -> Vec<(Point, (f32, f32))> {
    let mut valuenoise = FastNoise::seeded(rng.next_u64());
    valuenoise.set_noise_type(NoiseType::ValueFractal);
    valuenoise.set_frequency(0.1);

    let mut whitenoise = FastNoise::seeded(rng.next_u64());
    whitenoise.set_noise_type(NoiseType::WhiteNoise);
    whitenoise.set_frequency(0.1);

    let mut noisemap: Vec<(Point, (f32, f32))> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let vn = valuenoise.get_noise(x as f32, y as f32);
            let wn = whitenoise.get_noise(x as f32, y as f32);
            noisemap.push((Point {x, y}, (vn, wn)))
        }
    }
    noisemap
}

fn water_noisemap(rng: &mut RandomNumberGenerator, map: &Map, vfreq: f32) -> Vec<(Point, (f32, f32))> {
    let mut valuenoise = FastNoise::seeded(rng.next_u64());
    valuenoise.set_noise_type(NoiseType::Value);
    valuenoise.set_frequency(vfreq);

    let mut whitenoise = FastNoise::seeded(rng.next_u64());
    whitenoise.set_noise_type(NoiseType::WhiteNoise);
    whitenoise.set_frequency(0.7);

    let mut noisemap: Vec<(Point, (f32, f32))> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let vn = valuenoise.get_noise(x as f32, y as f32);
            let wn = whitenoise.get_noise(x as f32, y as f32);
            noisemap.push((Point {x, y}, (vn, wn)))
        }
    }
    noisemap
}

fn statue_noisemap(rng: &mut RandomNumberGenerator, map: &Map) -> Vec<(Point, f32)> {
    let mut whitenoise = FastNoise::seeded(rng.next_u64());
    whitenoise.set_noise_type(NoiseType::WhiteNoise);
    whitenoise.set_frequency(0.5);

    let mut noisemap: Vec<(Point, f32)> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let wn = whitenoise.get_noise(x as f32, y as f32);
            noisemap.push((Point {x, y}, wn));
        }
    }
    noisemap
}

fn monster_spawn_noisemap(rng: &mut RandomNumberGenerator, map: &Map) -> Vec::<(Point, f32)> {
    let mut valuenoise = FastNoise::seeded(rng.next_u64());
    valuenoise.set_noise_type(NoiseType::Value);
    valuenoise.set_frequency(0.25);

    let mut whitenoise = FastNoise::seeded(rng.next_u64());
    whitenoise.set_noise_type(NoiseType::WhiteNoise);
    whitenoise.set_frequency(0.5);

    let mut noisemap: Vec::<(Point, f32)> = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let wn = whitenoise.get_noise(x as f32, y as f32);
            let vn = valuenoise.get_noise(x as f32, y as f32);
            noisemap.push((Point {x, y}, wn + vn));
        }
    }
    noisemap//.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}