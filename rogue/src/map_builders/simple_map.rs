use rltk::RandomNumberGenerator;
use specs::prelude::*;

use super::{MapBuilder, NoiseMaps};
use super::{
    Map, TileType, Point, Rectangle, MAP_WIDTH, MAP_HEIGHT,
    DEBUG_VISUALIZE_MAPGEN, entity_spawners, terrain_spawners, WaterSpawnTable,
    GrassSpawnTable, StatueSpawnData, carve_out_rectangular_room,
    carve_out_horizontal_tunnel, carve_out_vertical_tunnel,
    carve_out_water_spawn_table, get_stairs_position,
    enumerate_connected_components, fill_all_but_largest_component
};

//----------------------------------------------------------------------------
// Simple Map
//
// This is the simplest map genration algorithm, A map with some rectangular
// rooms connected with L shaped corridors.
//----------------------------------------------------------------------------
pub struct SimpleMapBuilder {
    map: Map,
    depth: i32,
    rooms: Vec<Rectangle>,
    history: Vec<Map>
}

impl MapBuilder for SimpleMapBuilder {

    fn map(&self) -> Map {
        self.map.clone()
    }

    fn build_map(&mut self) -> NoiseMaps {
        self.carve_out_rooms_and_corridors();
        self.map.synchronize_blocked();

        let mut rng = RandomNumberGenerator::new();
        // Now we carve out space to spawn water.
        let noisemaps = NoiseMaps::sample(&mut rng, &self.map);
        let water_spawn_table = &noisemaps.to_water_spawn_table(&self.map);
        carve_out_water_spawn_table(&mut self.map, water_spawn_table);
        self.map.synchronize_blocked();
        // Carving out water maybe creates new connected components, so reduce
        // back to a single connected component.
        let components = enumerate_connected_components(&self.map);
        fill_all_but_largest_component(&mut self.map, components);
        self.map.synchronize_blocked();

        self.map.synchronize_opaque();
        self.map.synchronize_ok_to_spawn();

        noisemaps
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        entity_spawners::spawn_monsters(ecs, self.depth);
        for room in self.rooms.iter().skip(1) {
            let mut region: Vec<usize> = Vec::new();
            for y in room.y1 + 1 .. room.y2 {
                for x in room.x1 + 1 .. room.x2 {
                    let idx = self.map.xy_idx(x, y);
                    if self.map.tiles[idx] == TileType::Floor {
                        region.push(idx);
                    }
                }
            }
            entity_spawners::spawn_items_in_region(ecs, &region, self.depth);
        }
    }

    fn spawn_water(&mut self, ecs: &mut World) {
        let table: WaterSpawnTable;
        {
            let noisemaps = &*ecs.read_resource::<NoiseMaps>();
            let map = &*ecs.read_resource::<Map>();
            table = noisemaps.to_water_spawn_table(map);
        }
        terrain_spawners::water::spawn_water_from_table(ecs, &table)
    }

    fn spawn_terrain(&mut self, ecs: &mut World) {
        let grass_spawn_table: GrassSpawnTable;
        let statue_spawn_table: Vec<StatueSpawnData>;
        {
            let noisemaps = &*ecs.read_resource::<NoiseMaps>();
            let map = &*ecs.read_resource::<Map>();
            grass_spawn_table = noisemaps.to_grass_spawn_table(map);
            statue_spawn_table = noisemaps.to_statue_spawn_table(map);
        }
        terrain_spawners::foliage::spawn_grass_from_table(ecs, &grass_spawn_table);
        terrain_spawners::statues::spawn_statues_from_table(ecs, &statue_spawn_table);
    }

    fn starting_position(&self, ecs: &World) -> Point {
        let start = entity_spawners::player::get_spawn_position(ecs);
        match start {
            Some(start) => start,
            None => panic!("Failed to find a starting position.")
        }
    }

    fn stairs_position(&self, ecs: &World) -> Point {
        let start = get_stairs_position(ecs);
        match start {
            Some(start) => start,
            None => panic!("Failed to find a stairs position.")
        }
    }

    fn take_snapshot(&mut self) {
        if DEBUG_VISUALIZE_MAPGEN {
            let mut snapshot = self.map.clone();
            // So the snapshot will render everything.
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }

    fn snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

}

impl SimpleMapBuilder {

    pub fn new(depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder{
            map: Map::new(depth),
            depth: depth,
            rooms: Vec::new(),
            history: Vec::new()
        }
    }

    fn carve_out_rooms_and_corridors(&mut self) {
        const MAX_ROOMS: i32 = 40;
        const MIN_ROOM_SIZE: i32 = 6;
        const MAX_ROOM_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let h = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let x = rng.roll_dice(1, MAP_WIDTH - w - 1) - 1;
            let y = rng.roll_dice(1, MAP_HEIGHT - h - 1) - 1;
            let new_room = Rectangle::new(x, y, w, h);
            // Try to place our new room on the map.
            let ok_to_place = self.rooms
                .iter()
                .all(|other| !new_room.intersect(other));
            if ok_to_place {
                carve_out_rectangular_room(&mut self.map, &new_room);
                self.take_snapshot();
                if !self.rooms.is_empty() {
                    let (cxnew, cynew) = new_room.center();
                    let (cxprev, cyprev) =self.rooms[self.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        carve_out_horizontal_tunnel(&mut self.map, cxprev, cxnew, cyprev);
                        carve_out_vertical_tunnel(&mut self.map, cyprev, cynew, cxnew);
                    } else {
                        carve_out_vertical_tunnel(&mut self.map, cyprev, cynew, cxprev);
                        carve_out_horizontal_tunnel(&mut self.map, cxprev, cxnew, cynew);
                    }
                }
                self.rooms.push(new_room);
                self.take_snapshot();
            }
        }
    }
}