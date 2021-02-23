pub struct GameLog {
    pub entries: Vec<String>
}

impl GameLog {
    pub fn new() -> GameLog {
        GameLog{ entries: vec!["Welcome to the Rusty Roguelike!".to_string()]}
    }
}