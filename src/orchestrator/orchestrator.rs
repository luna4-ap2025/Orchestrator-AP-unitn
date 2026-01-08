/*use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};

use common_game::components::forge::Forge;
use common_game::components::resource::GenericResource;
use common_game::protocols::*;
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use crate::state::SystemState;
use crate::{planet_control, explorer_control};

pub struct Orchestrator {
    forge: Forge,

    pub(crate) planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>>,
    pub planet_receivers: HashMap<u32, Receiver<PlanetToOrchestrator>>,

    pub(crate) explorer_senders: HashMap<u32, Sender<OrchestratorToExplorer>>,
    pub explorer_receivers:
        HashMap<u32, Receiver<ExplorerToOrchestrator<Vec<GenericResource>>>>,

    pub state: SystemState,
}

impl Orchestrator {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            forge: Forge::new()?,
            planet_senders: HashMap::new(),
            planet_receivers: HashMap::new(),
            explorer_senders: HashMap::new(),
            explorer_receivers: HashMap::new(),
            state: SystemState::new(),
        })
    }

    pub fn run(&mut self) {
        loop {
            planet_control::poll_planets(self);
            explorer_control::poll_explorers(self);
        }
    }

    pub fn forge(&self) -> &Forge {
        &self.forge
    }
}
*/