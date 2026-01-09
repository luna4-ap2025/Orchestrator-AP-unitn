//! System state management for the orchestrator.
//!
//! This module defines the `SystemState` struct which tracks the current
//! state of the galaxy, including planet locations, explorer positions,
//! adjacency relationships, and game statistics.

use std::collections::{HashMap, HashSet};

use common_game::utils::ID;

/// Planet types as defined in the project specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanetType {
    /// Type A: 5 energy cells, limited generation rules, can have rockets
    A,
    /// Type B: 1 energy cell, unlimited generation rules, no rockets
    B,
    /// Type C: 1 energy cell, limited generation rules, can have rockets
    C,
    /// Type D: 5 energy cells, unlimited generation rules, no rockets
    D,
    /// Unknown type (used when planet type hasn't been determined)
    Unknown,
}

/// Explorer operating modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorerMode {
    /// Autonomous mode - explorer makes its own decisions
    Auto,
    /// Manual mode - controlled by user commands
    Manual,
}

impl Default for ExplorerMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl std::fmt::Display for ExplorerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::Manual => write!(f, "Manual"),
        }
    }
}

impl std::fmt::Display for PlanetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Tracks the complete state of the galaxy simulation.
#[derive(Debug, Clone)]
pub struct SystemState {
    /// Mapping of explorer IDs to their current planet IDs
    pub(crate) explorer_locations: HashMap<ID, ID>,

    /// Set of currently alive planets
    alive_planets: HashSet<ID>,

    /// Adjacency list representing galaxy topology
    adjacency: HashMap<ID, HashSet<ID>>,

    /// Planet statistics
    planet_stats: HashMap<ID, PlanetStats>,

    /// Explorer statistics
    explorer_stats: HashMap<ID, ExplorerStats>,

    /// Game-wide statistics
    game_stats: GameStats,
}


/// Statistics for a single planet
#[derive(Debug, Clone)]
pub struct PlanetStats {
    /// Current energy level (0.0 to 1.0)
    pub energy_level: f32,
    /// Number of charged energy cells
    pub charged_cells: usize,
    /// Whether the planet has a rocket
    pub has_rocket: bool,
    /// Planet type
    pub planet_type: PlanetType,
    /// Resources available for generation (as display strings)
    pub available_resources: Vec<String>,
}

impl Default for PlanetStats {
    fn default() -> Self {
        Self {
            energy_level: 0.0,
            charged_cells: 0,
            has_rocket: false,
            planet_type: PlanetType::Unknown,
            available_resources: Vec::new(),
        }
    }
}

/// Statistics for a single explorer
#[derive(Debug, Clone)]
pub struct ExplorerStats {
    /// Current health (0.0 to 1.0)
    pub health: f32,
    /// Number of resources in bag
    pub bag_count: usize,
    /// Whether AI is active
    pub ai_active: bool,
    /// Current mode
    pub mode: ExplorerMode,
}

impl Default for ExplorerStats {
    fn default() -> Self {
        Self {
            health: 1.0,
            bag_count: 0,
            ai_active: true,
            mode: ExplorerMode::default(),
        }
    }
}

/// Game-wide statistics
#[derive(Debug, Clone, Default)]
pub struct GameStats {
    /// Total asteroids sent
    pub asteroids_sent: u32,
    /// Total sunrays sent
    pub sunrays_sent: u32,
    /// Total planets destroyed
    pub planets_destroyed: u32,
    /// Total explorers killed
    pub explorers_killed: u32,
    /// Total resources generated (basic + complex)
    pub resources_generated: u32,
}

impl SystemState {
    /// Creates a new empty system state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            explorer_locations: HashMap::new(),
            alive_planets: HashSet::new(),
            adjacency: HashMap::new(),
            planet_stats: HashMap::new(),
            explorer_stats: HashMap::new(),
            game_stats: GameStats::default(),
        }
    }

    /// Adds a planet to the galaxy.
    pub fn add_planet(&mut self, planet_id: ID) {
        self.alive_planets.insert(planet_id);
        self.adjacency.entry(planet_id).or_default();
        self.planet_stats.insert(planet_id, PlanetStats::default());
    }

    /// Removes a planet from the galaxy.
    pub fn remove_planet(&mut self, planet_id: ID) {
        self.alive_planets.remove(&planet_id);
        self.adjacency.remove(&planet_id);
        self.planet_stats.remove(&planet_id);

        // Remove adjacency references to this planet
        for neighbors in self.adjacency.values_mut() {
            neighbors.remove(&planet_id);
        }

        self.game_stats.planets_destroyed += 1;
    }
    /// Gets the location of an explorer
    #[must_use]
    pub fn explorer_location(&self, explorer_id: ID) -> Option<ID> {
        self.explorer_locations.get(&explorer_id).copied()
    }

    /// Gets the neighbors of a planet.
    #[must_use]
    pub fn get_neighbors(&self, planet_id: ID) -> Vec<ID> {
        self.adjacency
            .get(&planet_id)
            .map(|neighbors| {
                let mut sorted: Vec<ID> = neighbors.iter().copied().collect();
                sorted.sort_unstable();
                sorted
            })
            .unwrap_or_default()
    }

    /// Gets all alive planet IDs in sorted order for deterministic GUI.
    #[must_use]
    pub fn alive_planets_sorted(&self) -> Vec<ID> {
        let mut planets: Vec<ID> = self.alive_planets.iter().copied().collect();
        planets.sort_unstable();
        planets
    }

    /// Gets explorers on a planet in sorted order.
    #[must_use]
    pub fn get_explorers_on_planet(&self, planet_id: ID) -> Vec<ID> {
        let mut explorers: Vec<ID> = self.explorer_locations
            .iter()
            .filter_map(|(&explorer_id, &location)| {
                if location == planet_id {
                    Some(explorer_id)
                } else {
                    None
                }
            })
            .collect();
        explorers.sort_unstable();
        explorers
    }

    /// Adds an explorer to the galaxy.
    ///
    /// # Errors
    ///
    /// Returns an error if the initial planet doesn't exist
    pub fn add_explorer(&mut self, explorer_id: ID, planet_id: ID) -> Result<(), String> {
        if !self.alive_planets.contains(&planet_id) {
            return Err(format!("Planet {planet_id} doesn't exist"));
        }

        self.explorer_locations.insert(explorer_id, planet_id);
        self.explorer_stats.insert(explorer_id, ExplorerStats::default());

        Ok(())
    }

    /// Removes an explorer from the galaxy.
    pub fn remove_explorer(&mut self, explorer_id: ID) {
        self.explorer_locations.remove(&explorer_id);
        self.explorer_stats.remove(&explorer_id);
        self.game_stats.explorers_killed += 1;
    }

    /// Moves an explorer from one planet to another.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The explorer doesn't exist
    /// - The explorer isn't on the specified current planet
    /// - The destination planet doesn't exist
    /// - The planets aren't adjacent
    pub fn move_explorer(
        &mut self,
        explorer_id: ID,
        from_planet: ID,
        to_planet: ID,
    ) -> Result<(), String> {
        let current_planet = self.explorer_locations.get(&explorer_id)
            .ok_or_else(|| format!("Explorer {explorer_id} doesn't exist"))?;

        if *current_planet != from_planet {
            return Err(format!("Explorer {explorer_id} is not on planet {from_planet}"));
        }

        if !self.alive_planets.contains(&to_planet) {
            return Err(format!("Destination planet {to_planet} doesn't exist"));
        }

        if !self.is_adjacent(from_planet, to_planet) {
            return Err(format!("Planets {from_planet} and {to_planet} are not adjacent"));
        }

        self.explorer_locations.insert(explorer_id, to_planet);

        Ok(())
    }

    /// Adds an adjacency relationship between two planets.
    pub fn add_adjacency(&mut self, planet_a: ID, planet_b: ID) {
        self.adjacency.entry(planet_a).or_default().insert(planet_b);
        self.adjacency.entry(planet_b).or_default().insert(planet_a);
    }

    /// Removes an adjacency relationship between two planets.
    pub fn remove_adjacency(&mut self, planet_a: ID, planet_b: ID) {
        if let Some(neighbors) = self.adjacency.get_mut(&planet_a) {
            neighbors.remove(&planet_b);
        }

        if let Some(neighbors) = self.adjacency.get_mut(&planet_b) {
            neighbors.remove(&planet_a);
        }
    }

    /// Checks if two planets are adjacent.
    #[must_use]
    pub fn is_adjacent(&self, planet_a: ID, planet_b: ID) -> bool {
        self.adjacency
            .get(&planet_a)
            .map(|neighbors| neighbors.contains(&planet_b))
            .unwrap_or(false)
    }

    /// Checks if a planet exists.
    #[must_use]
    pub fn has_planet(&self, planet_id: ID) -> bool {
        self.alive_planets.contains(&planet_id)
    }

    /// Updates planet statistics.
    pub fn update_planet_stats(&mut self, planet_id: ID, stats: PlanetStats) {
        self.planet_stats.insert(planet_id, stats);
    }

    /// Updates explorer statistics.
    pub fn update_explorer_stats(&mut self, explorer_id: ID, stats: ExplorerStats) {
        self.explorer_stats.insert(explorer_id, stats);
    }

    /// Increments the asteroids sent counter.
    pub fn increment_asteroids_sent(&mut self) {
        self.game_stats.asteroids_sent += 1;
    }

    /// Increments the sunrays sent counter.
    pub fn increment_sunrays_sent(&mut self) {
        self.game_stats.sunrays_sent += 1;
    }

    /// Increments the resources generated counter.
    pub fn increment_resources_generated(&mut self) {
        self.game_stats.resources_generated += 1;
    }

    /// Returns a reference to the game statistics.
    #[must_use]
    pub fn game_stats(&self) -> &GameStats {
        &self.game_stats
    }

    /// Returns a reference to planet statistics.
    #[must_use]
    pub fn planet_stats(&self, planet_id: ID) -> Option<&PlanetStats> {
        self.planet_stats.get(&planet_id)
    }

    /// Returns a reference to explorer statistics.
    #[must_use]
    pub fn explorer_stats(&self, explorer_id: ID) -> Option<&ExplorerStats> {
        self.explorer_stats.get(&explorer_id)
    }

    /// Returns all alive planet IDs.
    #[must_use]
    pub fn alive_planets(&self) -> &HashSet<ID> {
        &self.alive_planets
    }

    /// Returns a reference to explorer locations.
    #[must_use]
    pub fn explorer_locations(&self) -> &HashMap<ID, ID> {
        &self.explorer_locations
    }
}