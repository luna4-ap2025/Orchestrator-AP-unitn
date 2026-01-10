//! Main orchestrator implementation.
//!
//! The `Orchestrator` struct coordinates all communication between planets and
//! explorers, manages the galaxy state, and provides a GUI-friendly interface.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender, bounded, TryRecvError};
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
use crate::orchestrator::{planet_control, explorer_control};

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

    /// Control flag for graceful shutdown
    pub(crate) should_stop: Arc<AtomicBool>,
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
            should_stop: Arc::new(AtomicBool::new(false)),
        })
    }

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
        sender: Sender<OrchestratorToExplorer>,
        receiver: Receiver<ExplorerToOrchestrator<GenericResource>>,
        initial_planet: ID,
    ) -> Result<(), String> {
        if self.explorer_senders.contains_key(&explorer_id) {
            return Err(format!("Explorer with ID {explorer_id} already exists"));
        }

        if !self.state.has_planet(initial_planet) {
            return Err(format!("Initial planet {initial_planet} doesn't exist"));
        }

        self.explorer_senders.insert(explorer_id, sender.clone());
        self.explorer_receivers.insert(explorer_id, receiver);
        self.state.add_explorer(explorer_id, initial_planet)?;

        sender.send(OrchestratorToExplorer::MoveToPlanet {
            sender_to_new_planet: None,
            planet_id : initial_planet,
        })
            .map_err(|e| format!("Failed to send initial location to explorer: {e}"))?;

        let _ = self.gui_event_sender.send(GuiEvent::ExplorerAdded(explorer_id, initial_planet));

        Ok(())
    }

    /// Runs the orchestrator main loop.
    pub fn run(&mut self) -> Result<(), String> {
        log::info!("Orchestrator starting main loop");

        let mut last_gui_update = Instant::now();
        let gui_update_interval = Duration::from_millis(500);

        while !self.should_stop.load(Ordering::Relaxed) {
            self.poll_all_messages();

            let now = Instant::now();
            if now.duration_since(last_gui_update) >= gui_update_interval {
                self.periodic_tasks();
                self.update_gui_state();
                last_gui_update = now;
            }

            std::thread::sleep(Duration::from_millis(1));
        }

        log::info!("Orchestrator main loop stopped");
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

    /// Stops the orchestrator gracefully.
    pub fn stop(&self) {
        log::info!("Stopping orchestrator...");
        self.should_stop.store(true, Ordering::Relaxed);

        for (planet_id, sender) in &self.planet_senders {
            let _ = sender.send(OrchestratorToPlanet::KillPlanet);
            log::debug!("Sent KillPlanet to planet {planet_id}");
        }

        for (explorer_id, sender) in &self.explorer_senders {
            let _ = sender.send(OrchestratorToExplorer::KillExplorer);
            log::debug!("Sent KillExplorer to explorer {explorer_id}");
        }
    }

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

    /// Handle GUI commands from user
    fn handle_gui_command(&mut self, command: GuiCommand) {
        match command {
            GuiCommand::SendAsteroid { planet_id } => {
                self.send_asteroid_to_planet(planet_id);
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
                log::info!("Simulation pause requested");
            }
            GuiCommand::ResumeSimulation => {
                log::info!("Simulation resume requested");
            }
        }
    }

    /// Send an asteroid to a planet
    pub(crate) fn send_asteroid_to_planet(&mut self, planet_id: ID) {
        if let Some(sender) = self.planet_senders.get(&planet_id) {
            let asteroid = self.forge.generate_asteroid();
            if sender.send(OrchestratorToPlanet::Asteroid(asteroid)).is_ok() {
                log::info!("Sent asteroid to planet {planet_id}");
                self.state.increment_asteroids_sent();

                let _ = self.gui_event_sender.send(GuiEvent::AsteroidSent(planet_id));
            }
        }
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

    fn is_planet_alive(&self, planet_id: ID) -> bool {
        todo!()
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

    /// Periodic maintenance tasks

    fn periodic_tasks(&mut self) {
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
            self.handle_planet_death(planet_id);
        }
    }


    /// Update GUI with current state
    fn update_gui_state(&self) {
        let _ = self.gui_event_sender.send(GuiEvent::StateUpdate(self.get_gui_state()));
    }

    /// Handle planet death (disconnection)
    pub(crate) fn handle_planet_death(&mut self, planet_id: ID) {
        log::warn!("Planet {planet_id} appears to be dead, cleaning up");

        let explorers_on_planet: Vec<ID> = self.state
            .get_explorers_on_planet(planet_id);

        for explorer_id in explorers_on_planet {
            if let Some(sender) = self.explorer_senders.get(&explorer_id) {
                let _ = sender.send(OrchestratorToExplorer::KillExplorer);
            }
            self.state.remove_explorer(explorer_id);

            let _ = self.gui_event_sender.send(GuiEvent::ExplorerRemoved(explorer_id));
        }

        self.planet_senders.remove(&planet_id);
        self.planet_receivers.remove(&planet_id);
        self.state.remove_planet(planet_id);

        let _ = self.gui_event_sender.send(GuiEvent::PlanetRemoved(planet_id));
    }
}