//! Planet message handling and control.
//!
//! This module handles incoming messages from planets and coordinates
//! planet-related operations like destruction, asteroid defense, and
//! resource management.

//! Planet message handling and control.

use common_game::protocols::orchestrator_explorer::OrchestratorToExplorer;
use common_game::protocols::orchestrator_planet::PlanetToOrchestrator;
use common_game::utils::ID;

use crate::Orchestrator;

/// Handles incoming messages from a planet.
pub fn handle_planet_msg(
    orchestrator: &mut Orchestrator,
    planet_id: ID,
    msg: PlanetToOrchestrator,
) {
    match msg {
        PlanetToOrchestrator::KillPlanetResult { planet_id: msg_planet_id } => {
            debug_assert_eq!(planet_id, msg_planet_id, "Planet ID mismatch");
            handle_planet_destruction(orchestrator, planet_id);
        }
        PlanetToOrchestrator::AsteroidAck { planet_id: msg_planet_id, rocket } => {
            debug_assert_eq!(planet_id, msg_planet_id, "Planet ID mismatch");
            handle_asteroid_ack(orchestrator, planet_id, rocket);
        }
        PlanetToOrchestrator::SunrayAck { planet_id: msg_planet_id } => {
            debug_assert_eq!(planet_id, msg_planet_id, "Planet ID mismatch");
            handle_sunray_ack(orchestrator, planet_id);
        }
        PlanetToOrchestrator::InternalStateResponse { planet_id: msg_planet_id, planet_state } => {
            debug_assert_eq!(planet_id, msg_planet_id, "Planet ID mismatch");
            handle_internal_state_response(orchestrator, planet_id, planet_state);
        }
        PlanetToOrchestrator::IncomingExplorerResponse { planet_id: msg_planet_id, explorer_id, res } => {
            debug_assert_eq!(planet_id, msg_planet_id, "Planet ID mismatch");
            handle_incoming_explorer_response(orchestrator, planet_id, explorer_id, res);
        }
        PlanetToOrchestrator::OutgoingExplorerResponse { planet_id: msg_planet_id, explorer_id, res } => {
            debug_assert_eq!(planet_id, msg_planet_id, "Planet ID mismatch");
            handle_outgoing_explorer_response(orchestrator, planet_id, explorer_id, res);
        }
        PlanetToOrchestrator::StartPlanetAIResult { planet_id: msg_planet_id }
        | PlanetToOrchestrator::StopPlanetAIResult { planet_id: msg_planet_id }
        | PlanetToOrchestrator::Stopped { planet_id: msg_planet_id } => {
            debug_assert_eq!(planet_id, msg_planet_id, "Planet ID mismatch");
            log::debug!("Planet {planet_id} acknowledged command");
        }
    }
}

/// Handles planet destruction.
fn handle_planet_destruction(orchestrator: &mut Orchestrator, planet_id: ID) {
    log::info!("Planet {planet_id} has been destroyed");

    let doomed_explorers: Vec<ID> = orchestrator
        .state
        .get_explorers_on_planet(planet_id)
        .to_vec();

    for explorer_id in doomed_explorers {
        if let Some(tx) = orchestrator.explorer_senders.get(&explorer_id) {
            if tx.send(OrchestratorToExplorer::KillExplorer).is_err() {
                log::warn!("Failed to send KillExplorer to explorer {explorer_id}");
            }
            log::debug!("Sent KillExplorer to explorer {explorer_id}");
        }
        orchestrator.state.remove_explorer(explorer_id);

        let _ = orchestrator.gui_event_sender.send(
            crate::orchestrator::gui_interface::GuiEvent::ExplorerRemoved(explorer_id)
        );
    }

    orchestrator.state.remove_planet(planet_id);
    orchestrator.planet_senders.remove(&planet_id);
    orchestrator.planet_receivers.remove(&planet_id);

    let _ = orchestrator.gui_event_sender.send(
        crate::orchestrator::gui_interface::GuiEvent::PlanetRemoved(planet_id)
    );
}

/// Handles asteroid acknowledgment from a planet.
fn handle_asteroid_ack(
    orchestrator: &mut Orchestrator,
    planet_id: ID,
    rocket: Option<common_game::components::rocket::Rocket>,
) {
    if rocket.is_none() {
        log::warn!("Planet {planet_id} couldn't defend against asteroid, killing it");

        if let Some(tx) = orchestrator.planet_senders.get(&planet_id) {
            if tx.send(common_game::protocols::orchestrator_planet::OrchestratorToPlanet::KillPlanet).is_err() {
                log::warn!("Failed to send KillPlanet to planet {planet_id}");
            }
        }

        let _ = orchestrator.gui_event_sender.send(
            crate::orchestrator::gui_interface::GuiEvent::AsteroidHit(planet_id, false)
        );
    } else {
        log::info!("Planet {planet_id} successfully defended against asteroid");

        let _ = orchestrator.gui_event_sender.send(
            crate::orchestrator::gui_interface::GuiEvent::AsteroidHit(planet_id, true)
        );
    }
}

/// Handles sunray acknowledgment from a planet.
fn handle_sunray_ack(orchestrator: &mut Orchestrator, planet_id: ID) {
    log::debug!("Planet {planet_id} acknowledged sunray");

    let _ = orchestrator.gui_event_sender.send(
        crate::orchestrator::gui_interface::GuiEvent::SunrayReceived(planet_id)
    );
}

/// Handles internal state response from a planet.
fn handle_internal_state_response(
    orchestrator: &mut Orchestrator,
    planet_id: ID,
    dummy_state: common_game::components::planet::DummyPlanetState,
) {
    let energy_level = if dummy_state.energy_cells.is_empty() {
        0.0
    } else {
        dummy_state.charged_cells_count as f32 / dummy_state.energy_cells.len() as f32
    };

    let stats = crate::orchestrator::state::PlanetStats {
        energy_level,
        charged_cells: dummy_state.charged_cells_count,
        has_rocket: dummy_state.has_rocket,
        planet_type: crate::orchestrator::state::PlanetType::Unknown,
        available_resources: Vec::new(),
    };

    orchestrator.state.update_planet_stats(planet_id, stats);

    let _ = orchestrator.gui_event_sender.send(
        crate::orchestrator::gui_interface::GuiEvent::PlanetStateUpdated(planet_id)
    );
}

/// Handles incoming explorer response from a planet.
fn handle_incoming_explorer_response(
    orchestrator: &mut Orchestrator,
    planet_id: ID,
    explorer_id: ID,
    res: Result<(), String>,
) {
    match res {
        Ok(()) => {
            log::info!("Planet {planet_id} accepted incoming explorer {explorer_id}");

            let _ = orchestrator.gui_event_sender.send(
                crate::orchestrator::gui_interface::GuiEvent::ExplorerArrived(explorer_id, planet_id)
            );
        }
        Err(e) => {
            log::error!("Planet {planet_id} rejected explorer {explorer_id}: {e}");

            if let Some(tx) = orchestrator.explorer_senders.get(&explorer_id) {
                if tx.send(OrchestratorToExplorer::MoveToPlanet {
                    sender_to_new_planet: None,
                }).is_err() {
                    log::warn!("Failed to notify explorer {explorer_id} about move rejection");
                }
            }

            let _ = orchestrator.gui_event_sender.send(
                crate::orchestrator::gui_interface::GuiEvent::ExplorerMoveRejected(explorer_id, planet_id, e)
            );
        }
    }
}

/// Handles outgoing explorer response from a planet.
fn handle_outgoing_explorer_response(
    _orchestrator: &mut Orchestrator,
    planet_id: ID,
    explorer_id: ID,
    res: Result<(), String>,
) {
    match res {
        Ok(()) => {
            log::debug!("Planet {planet_id} acknowledged explorer {explorer_id} departure");
        }
        Err(e) => {
            log::warn!("Planet {planet_id} had issue with explorer {explorer_id} departure: {e}");
        }
    }
}