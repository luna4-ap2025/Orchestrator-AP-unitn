//! System state management for the orchestrator.

use std::collections::HashMap;
use common_game::utils::ID;

use super::galaxy::Galaxy;

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
    A,
    B,
    C,
    D,
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
    Auto,
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
    pub energy_level: f32,
    pub charged_cells: usize,
    pub has_rocket: bool,
    pub planet_type: PlanetType,
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
    pub health: f32,
    pub bag_count: usize,
    pub ai_active: bool,
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
    pub asteroids_sent: u32,
    pub sunrays_sent: u32,
    pub planets_destroyed: u32,
    pub explorers_killed: u32,
    pub resources_generated: u32,
}

/// Tracks the complete state of the galaxy simulation.
#[derive(Debug, Clone)]
pub struct SystemState {
    galaxy: Galaxy,
    game_state: GameState,
    explorer_locations: HashMap<ID, ID>,
    planet_stats: HashMap<ID, PlanetStats>,
    explorer_stats: HashMap<ID, ExplorerStats>,
    game_stats: GameStats,
}

impl SystemState {
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

    #[must_use]
    pub fn game_state(&self) -> GameState {
        self.game_state
    }

    pub fn pause(&mut self) {
        if self.game_state == GameState::Running {
            self.game_state = GameState::Paused;
            log::info!("Game paused");
        }
    }

    pub fn resume(&mut self) {
        if self.game_state == GameState::Paused {
            self.game_state = GameState::Running;
            log::info!("Game resumed");
        }
    }

    pub fn end_game(&mut self) {
        self.game_state = GameState::Ended;
        log::info!("Game ended");
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        self.game_state == GameState::Running
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.game_state == GameState::Paused
    }

    #[must_use]
    pub fn is_ended(&self) -> bool {
        self.game_state == GameState::Ended
    }

    #[must_use]
    pub fn should_continue(&self) -> bool {
        self.game_state != GameState::Ended
    }

    // ==================== Planet Management ====================

    pub fn add_planet(&mut self, planet_id: ID) {
        self.galaxy.add_planet(planet_id);
        self.planet_stats.insert(planet_id, PlanetStats::default());
    }

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

    #[must_use]
    pub fn is_planet_alive(&self, planet_id: ID) -> bool {
        self.galaxy.is_alive(planet_id)
    }

    #[must_use]
    pub fn alive_planets_sorted(&self) -> Vec<ID> {
        let mut planets: Vec<ID> = self.galaxy.alive_planets().iter().copied().collect();
        planets.sort_unstable();
        planets
    }

    #[must_use]
    pub fn alive_planets(&self) -> &std::collections::HashSet<ID> {
        self.galaxy.alive_planets()
    }

    pub fn update_planet_stats(&mut self, planet_id: ID, stats: PlanetStats) {
        self.planet_stats.insert(planet_id, stats);
    }

    #[must_use]
    pub fn planet_stats(&self, planet_id: ID) -> Option<&PlanetStats> {
        self.planet_stats.get(&planet_id)
    }

    // ==================== Explorer Management ====================

    pub fn add_explorer(&mut self, explorer_id: ID, planet_id: ID) -> Result<(), String> {
        if !self.galaxy.is_alive(planet_id) {
            return Err(format!("Planet {planet_id} doesn't exist"));
        }
        self.explorer_locations.insert(explorer_id, planet_id);
        self.explorer_stats.insert(explorer_id, ExplorerStats::default());
        Ok(())
    }

    pub fn remove_explorer(&mut self, explorer_id: ID) {
        self.explorer_locations.remove(&explorer_id);
        self.explorer_stats.remove(&explorer_id);
        self.game_stats.explorers_killed += 1;
    }

    #[must_use]
    pub fn explorer_location(&self, explorer_id: ID) -> Option<ID> {
        self.explorer_locations.get(&explorer_id).copied()
    }

    #[must_use]
    pub fn explorer_locations(&self) -> &HashMap<ID, ID> {
        &self.explorer_locations
    }

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

    pub fn update_explorer_stats(&mut self, explorer_id: ID, stats: ExplorerStats) {
        self.explorer_stats.insert(explorer_id, stats);
    }

    #[must_use]
    pub fn explorer_stats(&self, explorer_id: ID) -> Option<&ExplorerStats> {
        self.explorer_stats.get(&explorer_id)
    }

    // ==================== Adjacency Management ====================

    pub fn add_adjacency(&mut self, planet_a: ID, planet_b: ID) -> Result<(), String> {
        self.galaxy.add_connection(planet_a, planet_b)
    }

    #[must_use]
    pub fn is_adjacent(&self, planet_a: ID, planet_b: ID) -> bool {
        self.galaxy.can_travel(planet_a, planet_b)
    }

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

    pub fn increment_asteroids_sent(&mut self) {
        self.game_stats.asteroids_sent += 1;
    }

    pub fn increment_sunrays_sent(&mut self) {
        self.game_stats.sunrays_sent += 1;
    }

    pub fn increment_resources_generated(&mut self) {
        self.game_stats.resources_generated += 1;
    }

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