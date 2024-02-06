use super::{Map, TileType};

#[derive(PartialEq, Clone, Debug)]
pub struct ConnectedComponent {
    baseidx: usize,
    component: Vec<usize>,
    size: usize
}

pub fn enumerate_connected_components(map: &Map) -> Vec<ConnectedComponent> {
    let mut components: Vec<ConnectedComponent> = Vec::new();
    let mut floor: Vec<bool> = map.tiles
        .iter()
        .map(|tt| *tt == TileType::Floor)
        .collect();
    loop {
        // println!("{:?}", floor);
        let nextidx = get_next_idx(&map, &floor);
        match nextidx {
            None => {break;}
            Some(idx) => {
                // println!("Looping: {:?}.", idx);
                let component = get_connected_component(&map, idx);
                for cidx in component.component.iter() {
                    // println!("{:?}", cidx);
                    floor[*cidx] = false;
                }
                components.push(component);
            }
        }
    }
    // println!("{:?}", components);
    components
}

pub fn fill_all_but_largest_component(map: &mut Map, components: Vec<ConnectedComponent>) {
    let largest_component = components.iter().max_by(|c1, c2| c1.size.cmp(&c2.size)).unwrap();
    let other_components: Vec<&ConnectedComponent> = components.iter().filter(|c| *c != largest_component).collect();
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

fn get_connected_component(map: &Map, start_idx: usize) -> ConnectedComponent {
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
