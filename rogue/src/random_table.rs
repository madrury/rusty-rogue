use rltk::RandomNumberGenerator;


// An entry in a random spawn table. See RandomTable below.
pub struct RandomEntry<T: Copy> {
    item: T,
    weight: i32
}
impl<T: Copy> RandomEntry<T> {
    pub fn new(item: T, weight: i32) -> RandomEntry<T> {
        RandomEntry {item, weight}
    }
}

// A table of generic objects of a given type with associated weights. This
// table supports a sampling function (roll) which selects a (copy of) a random
// object from the table with the given weights.
#[derive(Default)]
pub struct RandomTable<T: Copy> {
    entries: Vec<RandomEntry<T>>,
    total_weight: i32
}
impl<T: Copy> RandomTable<T> {
    pub fn new() -> RandomTable<T> {
        RandomTable{
            entries: Vec::new(),
            total_weight: 0
        }
    }
    pub fn insert(mut self, item: T, weight: i32) -> RandomTable<T> {
        self.total_weight += weight;
        self.entries.push(RandomEntry {item, weight});
        self
    }
    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> Option<T> {
        if self.total_weight == 0 {
            return None
        }

        let mut roll = rng.roll_dice(1, self.total_weight - 1) - 1;
        for entry in &self.entries {
            if roll < entry.weight {
                return Some(entry.item)
            }
            roll -= entry.weight
        }

        None
    }
}