/*use common_game::protocols::*;
use common_game::protocols::orchestrator_explorer::OrchestratorToExplorer;
use crate::Orchestrator;

pub fn handle_travel_request(
    orchestrator: &mut Orchestrator,
    explorer_id: u32,
    from: u32,
    to: u32,
) {
    if !orchestrator.state.is_adjacent(from, to) {
        return;
    }

    let planet_tx = orchestrator.planet_senders.get(&to).cloned();

    if let Some(tx) = orchestrator.explorer_senders.get(&explorer_id) {
        let _ = tx.send(OrchestratorToExplorer::MoveToPlanet {
            sender_to_new_planet: planet_tx.map(|p| p.into()),
        });
    }

    orchestrator.state.explorer_location.insert(explorer_id, to);
}
*/