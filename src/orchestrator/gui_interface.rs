//! GUI-friendly interface for the orchestrator.
//!
//! This module provides types and functions to make orchestrator state
//! easily accessible to GUI applications like Bevy. It separates GUI
//! concerns from core orchestrator logic.

use serde::{Deserialize, Serialize};
use common_game::utils::ID;
use crate::orchestrator::galaxy_ai::AIPhase;
use crate::orchestrator::state::SystemState;

/// Events that flow FROM the orchestrator TO the GUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuiEvent {
    /// A planet was added to the system
    PlanetAdded(ID),
    /// A planet was removed from the system
    PlanetRemoved(ID),
    /// A planet's state was updated
    PlanetStateUpdated(ID),
    /// An explorer was added to the system
    ExplorerAdded(ID, ID), // explorer_id, planet_id
    /// An explorer was removed from the system
    ExplorerRemoved(ID),
    /// An explorer moved between planets
    ExplorerMoved(ID, ID, ID), // explorer_id, from_planet, to_planet
    /// An explorer arrived at a planet
    ExplorerArrived(ID, ID), // explorer_id, planet_id
    /// An explorer's move was rejected
    ExplorerMoveRejected(ID, ID, String), // explorer_id, planet_id, reason
    /// An explorer's location was confirmed
    ExplorerLocationConfirmed(ID, ID), // explorer_id, planet_id
    /// An explorer's bag was updated
    ExplorerBagUpdated(ID),
    /// Resources were discovered by an explorer
    ResourcesDiscovered(ID),
    /// Combinations were discovered by an explorer
    CombinationsDiscovered(ID),
    /// A resource was generated
    ResourceGenerated(ID, bool), // explorer_id, success
    /// Resources were combined
    ResourceCombined(ID, bool), // explorer_id, success
    /// An asteroid was sent to a planet
    AsteroidSent(ID),
    /// An asteroid hit a planet (with defense result)
    AsteroidHit(ID, bool), // planet_id, defended
    /// A sunray was sent to a planet
    SunraySent(ID),
    /// A sunray was received by a planet
    SunrayReceived(ID),
    /// Full system state update
    StateUpdate(GuiState),
}

/// Commands that flow FROM the GUI TO the orchestrator.
/// Commands that flow FROM the GUI TO the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuiCommand {
    /// Send an asteroid to a specific planet
    SendAsteroid {
        /// The ID of the planet to target with an asteroid
        planet_id: ID
    },
    /// Send a sunray to a specific planet
    SendSunray {
        /// The ID of the planet to send a sunray to
        planet_id: ID
    },
    /// Enable or disable a planet's AI
    TogglePlanetAI {
        /// The ID of the planet to toggle
        planet_id: ID,
        /// Whether to enable (true) or disable (false) the AI
        enabled: bool
    },
    /// Enable or disable an explorer's AI
    ToggleExplorerAI {
        /// The ID of the explorer to toggle
        explorer_id: ID,
        /// Whether to enable (true) or disable (false) the AI
        enabled: bool
    },
    /// Pause the entire simulation
    PauseSimulation,
    /// Resume the simulation from paused state
    ResumeSimulation,
    /// Change simulation cycle length
    SetSimulationCycleLengthInMillis {
        millis: u64
    },
    /// Set galaxy ai parameters
    SetGalaxyAIParameters {
        phase: AIPhase,
        phase_length: u32,
        phase_change: bool
    },
    /// Enable galaxy ai
    EnableGalaxyAI,
    /// Disable galaxy ai
    DisableGalaxyAI
}

/// GUI representation of the complete system state.
/// This is a snapshot used by the frontend to render the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiState {
    /// All planets currently in the simulation
    pub planets: Vec<GuiPlanet>,
    /// All explorers currently in the simulation
    pub explorers: Vec<GuiExplorer>,
    /// Game-wide statistics
    pub game_stats: GuiGameStats,
    /// Current simulation time in milliseconds
    pub simulation_time: u64,
}

/// GUI representation of a single planet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiPlanet {
    /// Unique identifier for the planet
    pub id: ID,
    /// Planet type as a string (e.g., "A", "B", "C", "D")
    pub planet_type: String,
    /// Current energy level (0.0 = empty, 1.0 = full)
    pub energy_level: f32,
    /// Number of charged energy cells (0-5 depending on planet type)
    pub charged_cells: usize,
    /// Whether the planet has a constructed rocket
    pub has_rocket: bool,
    /// Whether the planet's AI is currently active
    pub ai_active: bool,
    /// X-coordinate for rendering on screen
    pub x: f32,
    /// Y-coordinate for rendering on screen
    pub y: f32,
    /// IDs of adjacent planets (for travel visualization)
    pub neighbors: Vec<ID>,
    /// IDs of explorers currently on this planet
    pub explorers: Vec<ID>,
    /// Types of resources this planet can generate
    pub available_resources: Vec<String>,
}

/// GUI representation of a single explorer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiExplorer {
    /// Unique identifier for the explorer
    pub id: ID,
    /// ID of the planet where the explorer is currently located
    pub current_planet: ID,
    /// Health level (0.0 = dead, 1.0 = full health)
    pub health: f32,
    /// Number of resources currently carried in the explorer's bag
    pub bag_count: usize,
    /// Whether the explorer's AI is currently active
    pub ai_active: bool,
    /// Current operating mode ("Auto" or "Manual")
    pub mode: String,
    /// X-coordinate for rendering on screen
    pub x: f32,
    /// Y-coordinate for rendering on screen
    pub y: f32,
}

/// GUI representation of game-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiGameStats {
    /// Total number of asteroids sent by the user
    pub asteroids_sent: u32,
    /// Total number of sunrays sent by the user
    pub sunrays_sent: u32,
    /// Total number of planets destroyed
    pub planets_destroyed: u32,
    /// Total number of explorers killed
    pub explorers_killed: u32,
    /// Total number of resources generated (basic + complex)
    pub resources_generated: u32,
}

impl GuiState {
    /// Creates a GUI state snapshot from the system state.
    #[must_use]
    pub fn from_system_state(system_state: &SystemState) -> Self {
        let planets: Vec<GuiPlanet> = system_state
            .get_alive_planets_sorted()
            .into_iter()
            .enumerate()
            .map(|(i, planet_id)| {
                let stats = system_state.get_planet_stats(planet_id)
                    .cloned()
                    .unwrap_or_default();

                let planet_count = system_state.get_alive_planets().len();
                let radius = 300.0;
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / (planet_count as f32).max(1.0);

                GuiPlanet {
                    id: planet_id,
                    planet_type: stats.planet_type.to_string(),
                    energy_level: stats.energy_level,
                    charged_cells: stats.charged_cells,
                    has_rocket: stats.has_rocket,
                    ai_active: true, // TODO: Get actual AI state from planet
                    x: radius * angle.cos(),
                    y: radius * angle.sin(),
                    neighbors: system_state.get_neighbors(planet_id),
                    explorers: system_state.get_explorers_on_planet(planet_id),
                    available_resources: stats.available_resources,
                }
            })
            .collect();

        let explorers: Vec<GuiExplorer> = system_state
            .explorer_locations()
            .iter()
            .map(|(&explorer_id, &planet_id)| {
                let stats = system_state.explorer_stats(explorer_id)
                    .cloned()
                    .unwrap_or_default();

                let planet_pos = planets.iter()
                    .find(|p| p.id == planet_id)
                    .map(|p| (p.x, p.y))
                    .unwrap_or((0.0, 0.0));

                let offset = 20.0;
                let angle = (explorer_id as f32) * 0.5;

                GuiExplorer {
                    id: explorer_id,
                    current_planet: planet_id,
                    health: stats.health,
                    bag_count: stats.bag_count,
                    ai_active: stats.ai_active,
                    mode: stats.mode.to_string(),
                    x: planet_pos.0 + offset * angle.cos(),
                    y: planet_pos.1 + offset * angle.sin(),
                }
            })
            .collect();

        let game_stats = system_state.game_stats();

        Self {
            planets,
            explorers,
            game_stats: GuiGameStats {
                asteroids_sent: game_stats.asteroids_sent,
                sunrays_sent: game_stats.sunrays_sent,
                planets_destroyed: game_stats.planets_destroyed,
                explorers_killed: game_stats.explorers_killed,
                resources_generated: game_stats.resources_generated,
            },
            simulation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Gets the current mood emoji based on recent events.
    #[must_use]
    pub fn get_current_mood(&self) -> &'static str {
        if self.game_stats.asteroids_sent > 0 && self.game_stats.sunrays_sent == 0 {
            "😠"
        } else if self.game_stats.sunrays_sent > 0 && self.game_stats.asteroids_sent == 0 {
            "😊"
        } else if self.game_stats.asteroids_sent > 0 && self.game_stats.sunrays_sent > 0 {
            "🎲"
        } else {
            "😐"
        }
    }
}