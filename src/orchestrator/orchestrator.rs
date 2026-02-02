//! Main orchestrator implementation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender, TryRecvError, bounded};

use common_game::components::forge::Forge;
use common_game::components::resource::GenericResource;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::orchestrator_planet::{
    OrchestratorToPlanet, PlanetToOrchestrator,
};
use common_game::utils::ID;

use crate::orchestrator::state::SystemState;
use crate::orchestrator::gui_interface::{GuiEvent, GuiState, GuiCommand};
use crate::orchestrator::galaxy_ai::*;
use crate::orchestrator::galaxy_topology::GalaxyTopology;
use crate::orchestrator::{planet_control, explorer_control};
use crate::orchestrator::galaxy_ai::AIPhase::Dormant;

/// Main orchestrator structure that manages the entire simulation.
pub struct Orchestrator {
    /// Forge for generating asteroids and sunrays
    forge: Forge,

    /// Communication channels to planets
    pub(crate) planet_senders: HashMap<ID, Sender<OrchestratorToPlanet>>,
    pub(crate) planet_receivers: HashMap<ID, Receiver<PlanetToOrchestrator>>,

    /// Communication channels to explorers
    pub(crate) explorer_senders: HashMap<ID, Sender<OrchestratorToExplorer>>,
    explorer_receivers: HashMap<ID, Receiver<ExplorerToOrchestrator<GenericResource>>>,

    /// Current system state
    pub(crate) state: SystemState,

    /// GUI event channel (orchestrator → GUI)
    pub(crate) gui_event_sender: Sender<GuiEvent>,
    gui_event_receiver: Receiver<GuiEvent>,

    /// GUI command channel (GUI → orchestrator)
    gui_command_sender: Sender<GuiCommand>,
    gui_command_receiver: Receiver<GuiCommand>,

    /// Galaxy AI that makes decisions
    galaxy_ai: GalaxyAI,

    /// Control flag for graceful shutdown
    pub(crate) should_stop: Arc<AtomicBool>,

    ///Partial simulation speed control
    pub(crate) cycle_duration_in_millis: u64,
}

impl Orchestrator {
    /// Creates a new orchestrator instance.
    pub fn new() -> Result<Self, String> {
        let forge = Forge::new().map_err(|e| format!("Failed to create forge: {e}"))?;
        let (gui_event_sender, gui_event_receiver) = bounded(1000);
        let (gui_command_sender, gui_command_receiver) = bounded(100);

        Ok(Self {
            forge,
            planet_senders: HashMap::new(),
            planet_receivers: HashMap::new(),
            explorer_senders: HashMap::new(),
            explorer_receivers: HashMap::new(),
            state: SystemState::new(),
            gui_event_sender,
            gui_event_receiver,
            gui_command_sender,
            gui_command_receiver,
            galaxy_ai: GalaxyAI::new(),
            should_stop: Arc::new(AtomicBool::new(false)),
            cycle_duration_in_millis: 500,
        })
    }

    // ==================== Entity Management ====================

    /// Adds a planet to the orchestrator's management.
    pub fn add_planet(
        &mut self,
        planet_id: ID,
        sender: Sender<OrchestratorToPlanet>,
        receiver: Receiver<PlanetToOrchestrator>,
    ) -> Result<(), String> {
        if self.planet_senders.contains_key(&planet_id) {
            return Err(format!("Planet with ID {planet_id} already exists"));
        }

        self.planet_senders.insert(planet_id, sender);
        self.planet_receivers.insert(planet_id, receiver);
        self.state.add_planet(planet_id);

        let _ = self.gui_event_sender.send(GuiEvent::PlanetAdded(planet_id));
        Ok(())
    }

    /// Adds an explorer to the orchestrator's management.
    pub fn add_explorer(
        &mut self,
        explorer_id: ID,
        to_explorer_tx: Sender<OrchestratorToExplorer>,
        from_explorer_rx: Receiver<ExplorerToOrchestrator<GenericResource>>,
        planet_id: ID,
    ) -> Result<(), String> {
        if !self.state.is_planet_alive(planet_id) {
            return Err(format!("Planet {planet_id} is dead"));
        }

        self.explorer_senders.insert(explorer_id, to_explorer_tx);
        self.explorer_receivers.insert(explorer_id, from_explorer_rx);
        self.state.add_explorer(explorer_id, planet_id)?;

        let _ = self.gui_event_sender.send(GuiEvent::ExplorerAdded(explorer_id, planet_id));
        Ok(())
    }

    // ==================== Galaxy Setup ====================

    /// Creates a standard 7-planet galaxy with the specified topology.
    ///
    /// This automatically:
    /// - Creates 7 planets
    /// - Sets up communication channels
    /// - Connects them according to the topology
    ///
    /// # Arguments
    /// * `topology` - The connection pattern for the planets
    ///
    /// # Returns
    /// A vector of the created planet IDs (0-6)
    ///
    /// # Errors
    ///
    /// Returns an error if planet creation or topology setup fails
    pub fn setup_default_galaxy(&mut self, topology: GalaxyTopology) -> Result<Vec<ID>, String> {
        log::info!("Setting up 7-planet galaxy with {:?} topology", topology);

        let planet_ids: Vec<ID> = (0..7).map(|i| ID::from(i as u32)).collect();

        // Create all 7 planets with their channels
        for &planet_id in &planet_ids {
            let (to_planet_tx, _to_planet_rx) = bounded::<OrchestratorToPlanet>(100);
            let (_from_planet_tx, from_planet_rx) = bounded::<PlanetToOrchestrator>(100);

            // Store the channels
            self.planet_senders.insert(planet_id, to_planet_tx);
            self.planet_receivers.insert(planet_id, from_planet_rx);
            self.state.add_planet(planet_id);

            let _ = self.gui_event_sender.send(GuiEvent::PlanetAdded(planet_id));

            log::debug!("Created planet {}", planet_id);
        }

        // Create connections based on topology
        self.create_topology(&planet_ids, topology)?;

        log::info!("Successfully created 7-planet galaxy");
        Ok(planet_ids)
    }

    /// Creates the specified topology for the given planets
    fn create_topology(&mut self, planet_ids: &[ID], topology: GalaxyTopology) -> Result<(), String> {
        match topology {
            GalaxyTopology::Ring => self.create_ring_topology(planet_ids),
            GalaxyTopology::FullyConnected => self.create_fully_connected_topology(planet_ids),
            GalaxyTopology::Star => self.create_star_topology(planet_ids),
            GalaxyTopology::Line => self.create_line_topology(planet_ids),
            GalaxyTopology::Hub => self.create_hub_topology(planet_ids),
        }
    }

    /// Creates a ring topology: 0-1-2-3-4-5-6-0 (each planet has 2 neighbors)
    fn create_ring_topology(&mut self, planet_ids: &[ID]) -> Result<(), String> {
        for i in 0..planet_ids.len() {
            let current = planet_ids[i];
            let next = planet_ids[(i + 1) % planet_ids.len()];
            self.state.add_adjacency(current, next)?;
            log::debug!("Connected planet {} to planet {}", current, next);
        }
        log::info!("Created ring topology");
        Ok(())
    }

    /// Creates a fully connected topology: every planet connected to every other
    fn create_fully_connected_topology(&mut self, planet_ids: &[ID]) -> Result<(), String> {
        for i in 0..planet_ids.len() {
            for j in (i + 1)..planet_ids.len() {
                self.state.add_adjacency(planet_ids[i], planet_ids[j])?;
                log::debug!("Connected planet {} to planet {}", planet_ids[i], planet_ids[j]);
            }
        }
        log::info!("Created fully connected topology");
        Ok(())
    }

    /// Creates a star topology: planet 0 at center, connected to all others
    fn create_star_topology(&mut self, planet_ids: &[ID]) -> Result<(), String> {
        let center = planet_ids[0];
        for &planet in &planet_ids[1..] {
            self.state.add_adjacency(center, planet)?;
            log::debug!("Connected center planet {} to planet {}", center, planet);
        }
        log::info!("Created star topology with center planet {}", center);
        Ok(())
    }

    /// Creates a line topology: 0-1-2-3-4-5-6 (linear chain)
    fn create_line_topology(&mut self, planet_ids: &[ID]) -> Result<(), String> {
        for i in 0..(planet_ids.len() - 1) {
            self.state.add_adjacency(planet_ids[i], planet_ids[i + 1])?;
            log::debug!("Connected planet {} to planet {}", planet_ids[i], planet_ids[i + 1]);
        }
        log::info!("Created line topology");
        Ok(())
    }

    /// Creates a hub topology: 0 and 3 are hubs, others connect to nearest hub
    fn create_hub_topology(&mut self, planet_ids: &[ID]) -> Result<(), String> {
        let hub1 = planet_ids[0];
        let hub2 = planet_ids[3];

        // Hub 1 connections
        for &planet in &planet_ids[1..3] {
            self.state.add_adjacency(hub1, planet)?;
        }

        // Connect the two hubs
        self.state.add_adjacency(hub1, hub2)?;

        // Hub 2 connections
        for &planet in &planet_ids[4..] {
            self.state.add_adjacency(hub2, planet)?;
        }

        log::info!("Created hub topology with hubs at {} and {}", hub1, hub2);
        Ok(())
    }

    // ==================== Galaxy AI Management ====================

    /// Enables the Galaxy AI with the specified strategy
    pub fn enable_galaxy_ai(&mut self) {
        log::info!("Galaxy AI enabled with phase");
        self.galaxy_ai.enable_ai();
    }

    /// Disables the Galaxy AI
    pub fn disable_galaxy_ai(&mut self) {
        log::info!("Galaxy AI disabled");
        self.galaxy_ai.enable_ai();
    }

    /// Checks if Galaxy AI is enabled
    #[must_use]
    pub fn is_galaxy_ai_enabled(&self) -> bool {
        if self.galaxy_ai.get_phase() == &Dormant && self.galaxy_ai.get_phase_change() == &false {
            false
        }else {
            true
        }
    }

    /// Gets a reference to the Galaxy AI (if enabled)
    #[must_use]
    pub fn galaxy_ai(&self) -> &GalaxyAI {
        &self.galaxy_ai
    }

    /// Gets a mutable reference to the Galaxy AI (if enabled)
    pub fn galaxy_ai_mut(&mut self) -> &mut GalaxyAI {
        &mut self.galaxy_ai
    }

    // ==================== Main Loop ====================

    /// Runs the orchestrator main loop.
    pub fn run(&mut self) -> Result<(), String> {
        log::info!("Orchestrator starting main loop");

        let mut last_gui_update = Instant::now();
        let gui_update_interval = Duration::from_millis(500);
        self.enable_galaxy_ai();

        // Main loop runs until game ends OR stop requested
        while !self.should_stop.load(Ordering::Relaxed) && self.state.should_continue() {
            // Only process messages if game is running (not paused)
            if self.state.is_running() {
                self.poll_all_messages();
                self.periodic_tasks();
                self.run_galaxy_ai();
            }

            // Always update GUI (even when paused, to show pause state)
            let now = Instant::now();
            if now.duration_since(last_gui_update) >= gui_update_interval {
                self.update_gui_state();
                last_gui_update = now;
            }

            // Small sleep to prevent busy-waiting
            std::thread::sleep(Duration::from_millis(self.cycle_duration_in_millis));
        }

        log::info!("Orchestrator main loop stopped (game_state: {:?})", self.state.game_state());
        Ok(())
    }

    /// Polls all incoming messages from planets, explorers, and GUI
    fn poll_all_messages(&mut self) {
        // ---- PLANETS ----
        let mut planet_msgs = Vec::new();
        for (&planet_id, rx) in &self.planet_receivers {
            while let Ok(msg) = rx.try_recv() {
                planet_msgs.push((planet_id, msg));
            }
        }
        for (planet_id, msg) in planet_msgs {
            planet_control::handle_planet_msg(self, planet_id, msg);
        }

        // ---- EXPLORERS ----
        let mut explorer_msgs = Vec::new();
        for (&explorer_id, rx) in &self.explorer_receivers {
            while let Ok(msg) = rx.try_recv() {
                explorer_msgs.push((explorer_id, msg));
            }
        }
        for (explorer_id, msg) in explorer_msgs {
            explorer_control::handle_explorer_msg(self, explorer_id, msg);
        }

        // ---- GUI ----
        while let Ok(command) = self.gui_command_receiver.try_recv() {
            self.handle_gui_command(command);
        }
    }

    /// Periodic maintenance tasks
    fn periodic_tasks(&mut self) {
        // Check for disconnected planets
        let dead_planets: Vec<ID> = self.planet_receivers
            .iter()
            .filter_map(|(&id, rx)| {
                match rx.try_recv() {
                    Err(TryRecvError::Disconnected) => Some(id),
                    _ => None,
                }
            })
            .collect();

        for planet_id in dead_planets {
            self.destroy_planet(planet_id);
        }
    }

    /// Runs the Galaxy AI to make decisions
    fn run_galaxy_ai(&mut self) {
        let alive_planets = self.state.alive_planets_sorted();

        self.galaxy_ai.update(&alive_planets);

        let action = self.galaxy_ai.get_intention();

        match action {
            GalaxyAction::SendAsteroid { target_planet } => {
                log::info!("Galaxy AI decided to send asteroid to planet {target_planet}");
                let _ = self.send_asteroid_to_planet(*target_planet);
            }
            GalaxyAction::SendSunray { target_planet } => {
                log::info!("Galaxy AI decided to send sunray to planet {target_planet}");
                self.send_sunray_to_planet(*target_planet);
            }
            GalaxyAction::DoNothing => {
                // AI chose to do nothing this cycle
            }
        }
    }

    /// Update GUI with current state
    fn update_gui_state(&self) {
        let _ = self.gui_event_sender.send(GuiEvent::StateUpdate(self.get_gui_state()));
    }

    /// Stops the orchestrator gracefully.
    pub fn stop(&mut self) {
        log::info!("Stopping orchestrator...");
        self.should_stop.store(true, Ordering::Relaxed);
        self.disable_galaxy_ai();

        for (planet_id, sender) in &self.planet_senders {
            let _ = sender.send(OrchestratorToPlanet::KillPlanet);
            log::debug!("Sent KillPlanet to planet {planet_id}");
        }

        for (explorer_id, sender) in &self.explorer_senders {
            let _ = sender.send(OrchestratorToExplorer::KillExplorer);
            log::debug!("Sent KillExplorer to explorer {explorer_id}");
        }
    }

    // ==================== GUI Interface ====================

    /// Returns a snapshot of the current GUI state.
    #[must_use]
    pub fn get_gui_state(&self) -> GuiState {
        GuiState::from_system_state(&self.state)
    }

    /// Returns the GUI event receiver.
    #[must_use]
    pub fn gui_event_receiver(&self) -> Receiver<GuiEvent> {
        self.gui_event_receiver.clone()
    }

    /// Returns the GUI command sender.
    #[must_use]
    pub fn gui_command_sender(&self) -> Sender<GuiCommand> {
        self.gui_command_sender.clone()
    }

    /// Handle GUI commands from user
    fn handle_gui_command(&mut self, command: GuiCommand) {
        match command {
            GuiCommand::SendAsteroid { planet_id } => {
                let _ = self.send_asteroid_to_planet(planet_id);
            }
            GuiCommand::SendSunray { planet_id } => {
                self.send_sunray_to_planet(planet_id);
            }
            GuiCommand::TogglePlanetAI { planet_id, enabled } => {
                self.toggle_planet_ai(planet_id, enabled);
            }
            GuiCommand::ToggleExplorerAI { explorer_id, enabled } => {
                self.toggle_explorer_ai(explorer_id, enabled);
            }
            GuiCommand::PauseSimulation => {
                self.state.pause();
                log::info!("Simulation paused");
            }
            GuiCommand::ResumeSimulation => {
                self.state.resume();
                log::info!("Simulation resumed");
            }
            GuiCommand::SetSimulationCycleLengthInMillis { millis } => {
                self.cycle_duration_in_millis = millis;
            }
        }
    }

    // ==================== Game Actions ====================

    /// Send an asteroid to a planet
    pub fn send_asteroid_to_planet(&mut self, planet_id: ID) -> Result<(), String> {
        if !self.state.is_planet_alive(planet_id) {
            return Err("Planet is dead".into());
        }

        let tx = self.planet_senders
            .get(&planet_id)
            .ok_or("Planet sender missing")?;

        let asteroid = self.forge.generate_asteroid();
        tx.send(OrchestratorToPlanet::Asteroid(asteroid))
            .map_err(|_| "Send failed")?;

        self.state.increment_asteroids_sent();
        let _ = self.gui_event_sender.send(GuiEvent::AsteroidSent(planet_id));

        Ok(())
    }

    /// Send a sunray to a planet
    pub(crate) fn send_sunray_to_planet(&mut self, planet_id: ID) {
        if let Some(sender) = self.planet_senders.get(&planet_id) {
            let sunray = self.forge.generate_sunray();
            if sender.send(OrchestratorToPlanet::Sunray(sunray)).is_ok() {
                log::info!("Sent sunray to planet {planet_id}");
                self.state.increment_sunrays_sent();
                let _ = self.gui_event_sender.send(GuiEvent::SunraySent(planet_id));
            }
        }
    }

    /// Toggle planet AI on/off
    pub(crate) fn toggle_planet_ai(&mut self, planet_id: ID, enabled: bool) {
        if let Some(sender) = self.planet_senders.get(&planet_id) {
            let msg = if enabled {
                OrchestratorToPlanet::StartPlanetAI
            } else {
                OrchestratorToPlanet::StopPlanetAI
            };

            if sender.send(msg).is_ok() {
                log::info!("{} planet AI on planet {planet_id}",
                    if enabled { "Started" } else { "Stopped" });
            }
        }
    }

    /// Toggle explorer AI on/off
    fn toggle_explorer_ai(&mut self, explorer_id: ID, enabled: bool) {
        if let Some(sender) = self.explorer_senders.get(&explorer_id) {
            let msg = if enabled {
                OrchestratorToExplorer::StartExplorerAI
            } else {
                OrchestratorToExplorer::StopExplorerAI
            };

            if sender.send(msg).is_ok() {
                log::info!("{} explorer AI on explorer {explorer_id}",
                    if enabled { "Started" } else { "Stopped" });
            }
        }
    }

    // ==================== Planet/Explorer Lifecycle ====================

    /// Destroys a planet and all explorers on it.
    /// This is the PUBLIC method that planet_controls.rs should call.
    pub(crate) fn destroy_planet(&mut self, planet_id: ID) {
        log::warn!("Destroying planet {planet_id}");

        // Get all explorers on the dying planet
        let explorers_on_planet: Vec<ID> = self.state.get_explorers_on_planet(planet_id);

        // Kill all explorers on the planet
        for explorer_id in explorers_on_planet {
            self.kill_explorer(explorer_id);
        }

        // Send kill command to planet (if still connected)
        if let Some(sender) = self.planet_senders.get(&planet_id) {
            let _ = sender.send(OrchestratorToPlanet::KillPlanet);
        }

        // Clean up planet from orchestrator
        self.planet_senders.remove(&planet_id);
        self.planet_receivers.remove(&planet_id);

        // Remove from state (this also updates game stats)
        self.state.remove_planet(planet_id);

        // Notify GUI
        let _ = self.gui_event_sender.send(GuiEvent::PlanetRemoved(planet_id));
    }

    /// Kills an explorer
    pub(crate) fn kill_explorer(&mut self, explorer_id: ID) {
        log::info!("Killing explorer {explorer_id}");

        // Send kill command to explorer (if still connected)
        if let Some(sender) = self.explorer_senders.get(&explorer_id) {
            let _ = sender.send(OrchestratorToExplorer::KillExplorer);
        }

        // Clean up from orchestrator
        self.explorer_senders.remove(&explorer_id);
        self.explorer_receivers.remove(&explorer_id);

        // Remove from state (this also updates game stats)
        self.state.remove_explorer(explorer_id);

        // Notify GUI
        let _ = self.gui_event_sender.send(GuiEvent::ExplorerRemoved(explorer_id));
    }

    // ==================== Accessors ====================

    /// Returns a reference to the forge.
    #[must_use]
    pub fn forge(&self) -> &Forge {
        &self.forge
    }

    /// Returns a reference to the system state.
    #[must_use]
    pub fn state(&self) -> &SystemState {
        &self.state
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new().expect("Failed to create default Orchestrator")
    }
}