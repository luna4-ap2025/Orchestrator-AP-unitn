use std::collections::{HashMap, HashSet};

pub struct SystemState {
    pub explorer_location: HashMap<u32, u32>, // explorer_id -> planet_id
    pub alive_planets: HashSet<u32>,
    pub adjacency: HashMap<u32, HashSet<u32>>,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            explorer_location: HashMap::new(),
            alive_planets: HashSet::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn is_adjacent(&self, a: u32, b: u32) -> bool {
        self.adjacency
            .get(&a)
            .map(|n| n.contains(&b))
            .unwrap_or(false)
    }
}
