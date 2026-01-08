use common_game::protocols::*;
use crate::{routing, Orchestrator};

pub fn poll_explorers(orchestrator: &mut Orchestrator) {
    let explorer_ids: Vec<u32> =
        orchestrator.explorer_receivers.keys().cloned().collect();

    for eid in explorer_ids {
        if let Some(rx) = orchestrator.explorer_receivers.get(&eid) {
            while let Ok(msg) = rx.try_recv() {
                handle_explorer_msg(orchestrator, msg);
            }
        }
    }
}

fn handle_explorer_msg(
    orchestrator: &mut Orchestrator,
    msg: ExplorerToOrchestrator<Vec<GenericResource>>,
) {
    match msg {
        ExplorerToOrchestrator::TravelToPlanetRequest {
            explorer_id,
            current_planet_id,
            dst_planet_id,
        } => {
            routing::handle_travel_request(
                orchestrator,
                explorer_id,
                current_planet_id,
                dst_planet_id,
            );
        }
        _ => {}
    }
}
