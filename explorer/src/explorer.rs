use crossbeam_channel::{Receiver, Sender};

use common_game::protocols::messages::*;
use common_game::components::resource::GenericResource;

use crate::state::ExplorerState;
use crate::movement;

pub struct Explorer {
    id: u32,

    pub state: ExplorerState,

    pub orchestrator_tx: Option<Sender<ExplorerToOrchestrator<Vec<GenericResource>>>>,
    orchestrator_rx: Option<Receiver<OrchestratorToExplorer>>,

    pub planet_tx: Option<Sender<ExplorerToPlanet>>,
    planet_rx: Option<Receiver<PlanetToExplorer>>,
}

impl Explorer {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            state: ExplorerState::new(),
            orchestrator_tx: None,
            orchestrator_rx: None,
            planet_tx: None,
            planet_rx: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(rx) = &self.orchestrator_rx {
                while let Ok(msg) = rx.try_recv() {
                    self.handle_orchestrator_msg(msg);
                }
            }

            if let Some(rx) = &self.planet_rx {
                while let Ok(msg) = rx.try_recv() {
                    self.handle_planet_msg(msg);
                }
            }
        }
    }
}


impl Explorer {
    fn handle_orchestrator_msg(&mut self, msg: OrchestratorToExplorer) {
        match msg {
            OrchestratorToExplorer::StartExplorerAI => {
                if let Some(tx) = &self.orchestrator_tx {
                    let _ = tx.send(
                        ExplorerToOrchestrator::StartExplorerAIResult {
                            explorer_id: self.id,
                        },
                    );
                }
            }

            OrchestratorToExplorer::KillExplorerAI => {
                if let Some(tx) = &self.orchestrator_tx {
                    let _ = tx.send(
                        ExplorerToOrchestrator::KillExplorerAIResult {
                            explorer_id: self.id,
                        },
                    );
                }
                std::process::exit(0);
            }

            OrchestratorToExplorer::MoveToPlanet {
                sender_to_new_planet,
            } => {
                movement::handle_move(self, sender_to_new_planet);
            }

            _ => {}
        }
    }
}


impl Explorer {
    fn handle_orchestrator_msg(&mut self, msg: OrchestratorToExplorer) {
        match msg {
            OrchestratorToExplorer::StartExplorerAI => {
                if let Some(tx) = &self.orchestrator_tx {
                    let _ = tx.send(
                        ExplorerToOrchestrator::StartExplorerAIResult {
                            explorer_id: self.id,
                        },
                    );
                }
            }

            OrchestratorToExplorer::KillExplorerAI => {
                if let Some(tx) = &self.orchestrator_tx {
                    let _ = tx.send(
                        ExplorerToOrchestrator::KillExplorerAIResult {
                            explorer_id: self.id,
                        },
                    );
                }
                std::process::exit(0);
            }

            OrchestratorToExplorer::MoveToPlanet {
                sender_to_new_planet,
            } => {
                movement::handle_move(self, sender_to_new_planet);
            }

            _ => {}
        }
    }
}


impl Explorer {
    fn handle_planet_msg(&mut self, msg: PlanetToExplorer) {
        match msg {
            PlanetToExplorer::GenerateResourceResponse { resource } => {
                if let Some(res) = resource {
                    self.state.basic_resources.push(res);
                }
            }

            PlanetToExplorer::CombineResourceResponse { complex_response } => {
                if let Ok(res) = complex_response {
                    self.state.complex_resources.push(res);
                }
            }

            _ => {}
        }
    }
}
