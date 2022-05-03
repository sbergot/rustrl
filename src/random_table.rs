use bracket_lib::prelude::{RandomNumberGenerator};

struct RandomEntry<T> {
    data: T,
    weight: i32,
}

pub struct RandomTable<'a, T> {
    entries: Vec<RandomEntry<T>>,
    total_weight: i32,
    rng: &'a mut RandomNumberGenerator,
}

impl<'a, T> RandomTable<'a, T> {
    pub fn new(rng: &mut RandomNumberGenerator) -> RandomTable<T> {
        RandomTable {
            entries: Vec::new(),
            total_weight: 0,
            rng,
        }
    }

    pub fn add(mut self, data: T, weight: i32) -> RandomTable<'a, T> {
        self.total_weight += weight;
        self.entries.push(RandomEntry { data, weight });
        self
    }

    pub fn roll(&mut self) -> &T {
        let mut roll;
        {
            roll = self.rng.range(0, self.total_weight);
        }

        let mut index: usize = 0;
        while roll > 0 {
            let entry = &self.entries[index];
            if roll < entry.weight {
                return &entry.data;
            }

            roll -= entry.weight;
            index += 1;
        }

        &self.entries[index].data
    }
}
