//! Galaxy topology management.

use std::collections::{HashMap, HashSet};
use common_game::utils::ID;

/// Represents the galaxy topology (planets and their connections)
#[derive(Debug, Clone)]
pub struct Galaxy_Structure {
    adjacency: HashMap<ID,HashSet<ID>>, //uses tuples of IDs to indicate a 2-way connection between planets
    alive_planets: HashSet<ID>,
}

impl Galaxy_Structure {
    /// Creates a new empty galaxy
    #[must_use]
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            alive_planets: HashSet::new(),
        }
    }

    /// Creates a new galaxy from an array of strings
    pub fn new_from_file(structure_file: &[String]) -> Self {
        let mut result = Self::new();

        for line in structure_file {

            let parsed = Self::parse_u32_with_errors(line);

            if parsed.len() > 0 {
                let planet = parsed[0].clone() as ID;
                let mut adjacents: HashSet<ID> = HashSet::new();

                let adj = &parsed[1..];

                if !adj.is_empty() {
                    adjacents = adj.iter().map(|id| id.clone() as ID).collect();
                }

                result.add_planet_unchecked(planet, adjacents)
            }else {
                log::error!("File is either empty or invalid");
            }
        }

        result
    }

    ///parse string into u32 to use as ID
    fn parse_u32_with_errors(input: &str) -> Vec<u32> {
        let res: Result<Vec<u32>, String> = input
            .split_whitespace()
            .map(|s| s.parse::<u32>().map_err(|e| format!("'{}': {}", s, e)))
            .collect();

        if res.is_err() {
            log::error!("Failed to parse input: {}", input);
            vec![]
        }else {
            res.unwrap()
        }
    }

    fn add_planet_unchecked(&mut self, planet: ID, adjacents: HashSet<ID>) {
        self.alive_planets.insert(planet.clone());
        self.adjacency.insert(planet, adjacents);
    }

    /// Adds a planet to the galaxy and connects it accordingly
    pub fn add_planet(&mut self, planet: ID, adjacents: &[ID]) {
        self.alive_planets.insert(planet.clone());
        let set:HashSet<ID> = HashSet::from_iter(adjacents.iter().cloned());
        self.adjacency.insert(planet, set);

        if !adjacents.is_empty() {
            for a in adjacents {
                self.adjacency.get_mut(a).unwrap().insert(planet);
            }
        }
    }

    /// Removes a planet from the galaxy
    pub fn remove_planet(&mut self, planet: ID) {
        self.alive_planets.remove(&planet);

        if !self.adjacency.get(&planet).unwrap().is_empty() {
            for a in self.adjacency.values_mut() {
                a.remove(&planet);
            }
        }

        self.adjacency.remove(&planet);
    }

    /// Adds a connection between two planets
    ///
    /// # Errors
    ///
    /// Returns an error if either planet is dead
    pub fn add_connection(&mut self, a: ID, b: ID) {
        if !self.is_alive(a) || !self.is_alive(b) {
            log::error!("cannot connect dead planets: planet {a} is {}, planet {b} is {}",
                if self.is_alive(a) {"alive"} else {"dead"},
                if self.is_alive(b) {"alive"} else {"dead"});
        } else {
            self.adjacency.entry(a).or_default().insert(b);
            self.adjacency.entry(b).or_default().insert(a);
        }
    }

    /// Checks if travel is possible from one planet to another
    #[must_use]
    pub fn can_travel(&self, from: ID, to: ID) -> bool {
        self.is_alive(to)
            && self.adjacency
            .get(&from)
            .map_or(false, |n| n.contains(&to))
    }

    /// Checks if a planet is alive
    #[must_use]
    pub fn is_alive(&self, planet: ID) -> bool {
        self.alive_planets.contains(&planet)
    }

    /// Returns a reference to alive planets
    #[must_use]
    pub fn get_alive_planets(&self) -> &HashSet<ID> {
        &self.alive_planets
    }

    /// Returns a reference to the adjacency map
    #[must_use]
    pub fn get_adjacency(&self) -> &HashMap<ID, HashSet<ID>> {
        &self.adjacency
    }

    /// Returns the adjacents to a specific planet
    #[must_use]
    pub fn get_adjacents(&self, select: ID) -> &HashSet<ID> {
        self.adjacency.get(&select).unwrap()
    }
}

impl Default for Galaxy_Structure {
    fn default() -> Self {
        Self::new()
    }
}