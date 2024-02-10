use super::{Map, TileType};

// Module for working with the connected compoents of a map, used during
// generation to count, measure, and fill the randomly generated connected
// components.

#[derive(PartialEq, Clone, Debug)]
pub struct ConnectedComponent {
    // A map index in the connected component.
    baseidx: usize,
    // ALL map indicies in the connected component.
    component: Vec<usize>,
    // The size of the connected component, i.e., count of tiles.
    size: usize
}

pub fn enumerate_connected_components(map: &Map) -> Vec<ConnectedComponent> {
    let mut components: Vec<ConnectedComponent> = Vec::new();
    let mut floor: Vec<bool> = map.tiles
        .iter()
        .map(|tt| *tt == TileType::Floor)
        .collect();
    loop {
        let nextidx = get_next_idx(&map, &floor);
        match nextidx {
            None => {break;}
            Some(idx) => {
                let component = get_connected_component_containing_index(&map, idx);
                for cidx in component.component.iter() {
                    floor[*cidx] = false;
                }
                components.push(component);
            }
        }
    }
    components
}

pub fn fill_all_but_largest_component(map: &mut Map, components: Vec<ConnectedComponent>) {
    let largest_component = components.iter()
        .max_by(|c1, c2| c1.size.cmp(&c2.size))
        .expect("Failed to find the largest component");
    let other_components: Vec<&ConnectedComponent> = components.iter()
        .filter(|c| *c != largest_component)
        .collect();
    for component in other_components {
        for cidx in component.component.iter() {
            map.tiles[*cidx] = TileType::Wall;
        }
    }
}

fn get_next_idx(map: &Map, floor: &Vec<bool>) -> Option<usize> {
    for idx in 0..(map.width * map.height) {
        if floor[idx as usize] {return Some(idx as usize)}
    }
    None
}

fn get_connected_component_containing_index(map: &Map, start_idx: usize) -> ConnectedComponent {
    let map_starts : Vec<usize> = vec![start_idx];
    let dijkstra_map = rltk::DijkstraMap::new(
        map.width, map.height, &map_starts , map, 200.0
    );
    let mut component: Vec<usize> = vec![start_idx];
    for (idx, _) in map.tiles.iter().enumerate() {
        if dijkstra_map.map[idx] != std::f32::MAX {
            component.push(idx)
        }
    }
    let size = component.len();
    ConnectedComponent {baseidx: start_idx, component: component, size: size}
}
