//! System state management for the orchestrator.

use std::collections::HashMap;
use common_game::utils::ID;

use super::galaxy_structure::Galaxy;

/// Overall game state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Game is running normally
    Running,
    /// Game is paused (simulation frozen)
    Paused,
    /// Game has ended (all planets destroyed or user quit)
    Ended,
}

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

/// Tracks the complete state of the galaxy simulation.
#[derive(Debug, Clone)]
pub struct SystemState {
    /// Galaxy topology (planets and adjacency)
    galaxy: Galaxy,
    /// Current game state (running/paused/ended)
    game_state: GameState,
    /// Mapping of explorer IDs to their current planet IDs
    explorer_locations: HashMap<ID, ID>,
    /// Planet statistics
    planet_stats: HashMap<ID, PlanetStats>,
    /// Explorer statistics
    explorer_stats: HashMap<ID, ExplorerStats>,
    /// Game-wide statistics
    game_stats: GameStats,
}

impl SystemState {
    /// Creates a new empty system state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            galaxy: Galaxy::new(),
            game_state: GameState::Running,
            explorer_locations: HashMap::new(),
            planet_stats: HashMap::new(),
            explorer_stats: HashMap::new(),
            game_stats: GameStats::default(),
        }
    }

    // ==================== Game State Management ====================

    /// Gets the current game state
    #[must_use]
    pub fn game_state(&self) -> GameState {
        self.game_state
    }

    /// Pauses the game
    pub fn pause(&mut self) {
        if self.game_state == GameState::Running {
            self.game_state = GameState::Paused;
            log::info!("Game paused");
        }
    }

    /// Resumes the game
    pub fn resume(&mut self) {
        if self.game_state == GameState::Paused {
            self.game_state = GameState::Running;
            log::info!("Game resumed");
        }
    }

    /// Ends the game
    pub fn end_game(&mut self) {
        self.game_state = GameState::Ended;
        log::info!("Game ended");
    }

    /// Checks if the game is running
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.game_state == GameState::Running
    }

    /// Checks if the game is paused
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.game_state == GameState::Paused
    }

    /// Checks if the game has ended
    #[must_use]
    pub fn is_ended(&self) -> bool {
        self.game_state == GameState::Ended
    }

    /// Checks if the game should continue (not ended)
    #[must_use]
    pub fn should_continue(&self) -> bool {
        self.game_state != GameState::Ended
    }

    // ==================== Planet Management ====================

    /// Adds a planet to the galaxy.
    pub fn add_planet(&mut self, planet_id: ID) {
        self.galaxy.add_planet(planet_id);
        self.planet_stats.insert(planet_id, PlanetStats::default());
    }

    /// Removes a planet from the galaxy and kills all explorers on it.
    pub fn remove_planet(&mut self, planet_id: ID) {
        // Kill all explorers on this planet
        let explorers_to_remove: Vec<ID> = self.explorer_locations
            .iter()
            .filter_map(|(explorer_id, location)| {
                if *location == planet_id {
                    Some(*explorer_id)
                } else {
                    None
                }
            })
            .collect();

        for explorer_id in explorers_to_remove {
            self.remove_explorer(explorer_id);
        }

        // Remove from galaxy
        self.galaxy.remove_planet(planet_id);

        // Remove planet stats
        self.planet_stats.remove(&planet_id);

        // Update stats
        self.game_stats.planets_destroyed += 1;

        // Check if game should end (no planets left)
        if self.galaxy.alive_planets().is_empty() {
            log::warn!("All planets destroyed - ending game");
            self.end_game();
        }
    }

    /// Checks if a planet exists and is alive.
    #[must_use]
    pub fn is_planet_alive(&self, planet_id: ID) -> bool {
        self.galaxy.is_alive(planet_id)
    }

    /// Gets all alive planet IDs in sorted order for deterministic GUI.
    #[must_use]
    pub fn alive_planets_sorted(&self) -> Vec<ID> {
        let mut planets: Vec<ID> = self.galaxy.alive_planets().iter().copied().collect();
        planets.sort_unstable();
        planets
    }

    /// Gets all alive planet IDs.
    #[must_use]
    pub fn alive_planets(&self) -> &std::collections::HashSet<ID> {
        self.galaxy.alive_planets()
    }

    /// Updates planet statistics.
    pub fn update_planet_stats(&mut self, planet_id: ID, stats: PlanetStats) {
        self.planet_stats.insert(planet_id, stats);
    }

    /// Returns a reference to planet statistics.
    #[must_use]
    pub fn planet_stats(&self, planet_id: ID) -> Option<&PlanetStats> {
        self.planet_stats.get(&planet_id)
    }

    // ==================== Explorer Management ====================

    /// Adds an explorer to the galaxy.
    ///
    /// # Errors
    ///
    /// Returns an error if the initial planet doesn't exist
    pub fn add_explorer(&mut self, explorer_id: ID, planet_id: ID) -> Result<(), String> {
        if !self.galaxy.is_alive(planet_id) {
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

    /// Gets the location of an explorer
    #[must_use]
    pub fn explorer_location(&self, explorer_id: ID) -> Option<ID> {
        self.explorer_locations.get(&explorer_id).copied()
    }

    /// Returns a reference to explorer locations.
    #[must_use]
    pub fn explorer_locations(&self) -> &HashMap<ID, ID> {
        &self.explorer_locations
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
        let current_planet = self.explorer_locations
            .get(&explorer_id)
            .ok_or_else(|| format!("Explorer {explorer_id} doesn't exist"))?;

        if *current_planet != from_planet {
            return Err(format!(
                "Explorer {explorer_id} is not on planet {from_planet}"
            ));
        }

        if !self.galaxy.can_travel(from_planet, to_planet) {
            return Err(format!(
                "Cannot travel from {from_planet} to {to_planet}"
            ));
        }

        self.explorer_locations.insert(explorer_id, to_planet);
        Ok(())
    }

    /// Updates explorer statistics.
    pub fn update_explorer_stats(&mut self, explorer_id: ID, stats: ExplorerStats) {
        self.explorer_stats.insert(explorer_id, stats);
    }

    /// Returns a reference to explorer statistics.
    #[must_use]
    pub fn explorer_stats(&self, explorer_id: ID) -> Option<&ExplorerStats> {
        self.explorer_stats.get(&explorer_id)
    }

    // ==================== Adjacency Management ====================

    /// Adds an adjacency relationship between two planets.
    ///
    /// # Errors
    ///
    /// Returns an error if either planet is dead
    pub fn add_adjacency(&mut self, planet_a: ID, planet_b: ID) -> Result<(), String> {
        self.galaxy.add_connection(planet_a, planet_b)
    }

    /// Checks if two planets are adjacent.
    #[must_use]
    pub fn is_adjacent(&self, planet_a: ID, planet_b: ID) -> bool {
        self.galaxy.can_travel(planet_a, planet_b)
    }

    /// Gets the neighbors of a planet.
    #[must_use]
    pub fn get_neighbors(&self, planet_id: ID) -> Vec<ID> {
        self.galaxy.adjacency()
            .get(&planet_id)
            .map(|neighbors| {
                let mut sorted: Vec<ID> = neighbors.iter().copied().collect();
                sorted.sort_unstable();
                sorted
            })
            .unwrap_or_default()
    }

    // ==================== Statistics ====================

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
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}