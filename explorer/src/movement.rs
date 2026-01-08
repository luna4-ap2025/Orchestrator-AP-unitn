use crossbeam_channel::Sender;
use common_game::protocols::*;

use crate::Explorer;

pub fn handle_move(
    explorer: &mut Explorer,
    sender_to_new_planet: Option<Sender<ExplorerToPlanet>>,
) {
    explorer.planet_tx = sender_to_new_planet;
    explorer.state.current_planet = Some(0); // orchestrator tracks real ID

    if let Some(tx) = &explorer.orchestrator_tx {
        let _ = tx.send(
            ExplorerToOrchestrator::MovedToPlanetResult {
                explorer_id: explorer.id,
            },
        );
    }
}
