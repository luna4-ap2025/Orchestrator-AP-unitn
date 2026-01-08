/*use common_game::protocols::*;
use common_game::protocols::orchestrator_explorer::OrchestratorToExplorer;
use common_game::protocols::orchestrator_planet::PlanetToOrchestrator;
use crate::Orchestrator;

pub fn poll_planets(orchestrator: &mut Orchestrator) {
    let planet_ids: Vec<u32> =
        orchestrator.planet_receivers.keys().cloned().collect();

    for pid in planet_ids {
        if let Some(rx) = orchestrator.planet_receivers.get(&pid) {
            while let Ok(msg) = rx.try_recv() {
                handle_planet_msg(orchestrator, msg);
            }
        }
    }
}

fn handle_planet_msg(orchestrator: &mut Orchestrator, msg: PlanetToOrchestrator) {
    match msg {
        PlanetToOrchestrator::KillPlanetAck { planet_id } => {
            handle_planet_destruction(orchestrator, planet_id);
        }
        _ => {}
    }
}

fn handle_planet_destruction(orchestrator: &mut Orchestrator, planet_id: u32) {
    let doomed: Vec<u32> = orchestrator
        .state
        .explorer_location
        .iter()
        .filter(|(_, &p)| p == planet_id)
        .map(|(&e, _)| e)
        .collect();

    for explorer_id in doomed {
        if let Some(tx) = orchestrator.explorer_senders.get(&explorer_id) {
            let _ = tx.send(OrchestratorToExplorer::KillExplorerAI);
        }
        orchestrator.state.explorer_location.remove(&explorer_id);
    }

    orchestrator.state.alive_planets.remove(&planet_id);
    orchestrator.planet_senders.remove(&planet_id);
    orchestrator.planet_receivers.remove(&planet_id);
}
*/