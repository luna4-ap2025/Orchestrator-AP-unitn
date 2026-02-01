//! Galaxy topology management.

use std::collections::{HashMap, HashSet};
use common_game::utils::ID;

/// Represents the galaxy topology (planets and their connections)
#[derive(Debug, Clone)]
pub struct Galaxy {
    adjacency: HashMap<ID, HashSet<ID>>,
    alive_planets: HashSet<ID>,
}

impl Galaxy {
    /// Creates a new empty galaxy
    #[must_use]
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            alive_planets: HashSet::new(),
        }
    }

    /// Adds a planet to the galaxy
    pub fn add_planet(&mut self, planet: ID) {
        self.alive_planets.insert(planet);
        self.adjacency.entry(planet).or_default();
    }

    /// Removes a planet from the galaxy
    pub fn remove_planet(&mut self, planet: ID) {
        self.alive_planets.remove(&planet);
        if let Some(neighbors) = self.adjacency.remove(&planet) {
            for n in neighbors {
                if let Some(set) = self.adjacency.get_mut(&n) {
                    set.remove(&planet);
                }
            }
        }
    }

    /// Adds a connection between two planets
    ///
    /// # Errors
    ///
    /// Returns an error if either planet is dead
    pub fn add_connection(&mut self, a: ID, b: ID) -> Result<(), String> {
        if !self.is_alive(a) || !self.is_alive(b) {
            return Err("Cannot connect dead planets".into());
        }
        self.adjacency.entry(a).or_default().insert(b);
        self.adjacency.entry(b).or_default().insert(a);
        Ok(())
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
    pub fn alive_planets(&self) -> &HashSet<ID> {
        &self.alive_planets
    }

    /// Returns a reference to the adjacency map
    #[must_use]
    pub fn adjacency(&self) -> &HashMap<ID, HashSet<ID>> {
        &self.adjacency
    }
}

impl Default for Galaxy {
    fn default() -> Self {
        Self::new()
    }
}