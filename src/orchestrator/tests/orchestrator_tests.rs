//! Integration tests for the orchestrator
//!
//! These tests verify that the orchestrator correctly coordinates
//! planets, explorers, routing, and system state updates.

use crossbeam_channel::bounded;
use log::LevelFilter;

use common_game::components::resource::GenericResource;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator,
    OrchestratorToExplorer,
};
use common_game::protocols::orchestrator_planet::{
    OrchestratorToPlanet,
    PlanetToOrchestrator,
};
use common_game::utils::ID;

use orchestrator::logging;
use orchestrator::Orchestrator;

//
// -------------------- Orchestrator basics --------------------
//

#[test]
fn test_orchestrator_initialization() {
    logging::init(LevelFilter::Warn);

    let orchestrator = Orchestrator::new();
    assert!(orchestrator.is_ok());
}

//
// -------------------- Planet tests --------------------
//

#[test]
fn test_add_seven_planets() {
    let mut orchestrator =
        Orchestrator::new().expect("Orchestrator should initialize");

    for i in 0..7 {
        let planet_id = ID::from(i as u32);

        let (to_planet_tx, _to_planet_rx) =
            bounded::<OrchestratorToPlanet>(10);
        let (_from_planet_tx, from_planet_rx) =
            bounded::<PlanetToOrchestrator>(10);

        orchestrator
            .add_planet(planet_id, to_planet_tx, from_planet_rx)
            .expect("Planet should be added");
    }

    assert_eq!(orchestrator.state().alive_planets().len(), 7);
}

#[test]
fn test_send_asteroid_and_sunray() {
    let mut orchestrator =
        Orchestrator::new().expect("Orchestrator should initialize");

    let planet_id = ID::from(1u32);

    let (to_planet_tx, _to_planet_rx) =
        bounded::<OrchestratorToPlanet>(10);
    let (_from_planet_tx, from_planet_rx) =
        bounded::<PlanetToOrchestrator>(10);

    orchestrator
        .add_planet(planet_id, to_planet_tx, from_planet_rx)
        .expect("Planet should be added");

    orchestrator.send_asteroid_to_planet(planet_id);
    assert_eq!(
        orchestrator.state().game_stats().asteroids_sent,
        1
    );

    orchestrator.send_sunray_to_planet(planet_id);
    assert_eq!(
        orchestrator.state().game_stats().sunrays_sent,
        1
    );
}

//
// -------------------- Explorer tests --------------------
//

#[test]
fn test_add_explorer_to_planet() {
    let mut orchestrator =
        Orchestrator::new().expect("Orchestrator should initialize");

    let planet_id = ID::from(1u32);

    let (to_planet_tx, _to_planet_rx) =
        bounded::<OrchestratorToPlanet>(10);
    let (_from_planet_tx, from_planet_rx) =
        bounded::<PlanetToOrchestrator>(10);

    orchestrator
        .add_planet(planet_id, to_planet_tx, from_planet_rx)
        .expect("Planet should be added");

    let explorer_id = ID::from(42u32);

    let (to_explorer_tx, _to_explorer_rx) =
        bounded::<OrchestratorToExplorer>(10);
    let (_from_explorer_tx, from_explorer_rx) =
        bounded::<ExplorerToOrchestrator<GenericResource>>(10);

    orchestrator
        .add_explorer(
            explorer_id,
            to_explorer_tx,
            from_explorer_rx,
            planet_id,
        )
        .expect("Explorer should be added");

    assert_eq!(
        orchestrator.state().explorer_location(explorer_id),
        Some(planet_id)
    );
}

#[test]
fn test_add_explorer_to_nonexistent_planet() {
    let mut orchestrator =
        Orchestrator::new().expect("Orchestrator should initialize");

    let explorer_id = ID::from(99u32);
    let nonexistent_planet = ID::from(999u32);

    let (to_explorer_tx, _to_explorer_rx) =
        bounded::<OrchestratorToExplorer>(10);
    let (_from_explorer_tx, from_explorer_rx) =
        bounded::<ExplorerToOrchestrator<GenericResource>>(10);

    let result = orchestrator.add_explorer(
        explorer_id,
        to_explorer_tx,
        from_explorer_rx,
        nonexistent_planet,
    );

    assert!(result.is_err());
    assert!(
        orchestrator
            .state()
            .explorer_location(explorer_id)
            .is_none()
    );
}

#[test]
fn test_explorer_movement_between_planets() {
    let mut orchestrator =
        Orchestrator::new().expect("Orchestrator should initialize");

    let planet_a = ID::from(1u32);
    let planet_b = ID::from(2u32);

    let (to_a_tx, _to_a_rx) =
        bounded::<OrchestratorToPlanet>(10);
    let (_from_a_tx, from_a_rx) =
        bounded::<PlanetToOrchestrator>(10);
    orchestrator
        .add_planet(planet_a, to_a_tx, from_a_rx)
        .unwrap();

    let (to_b_tx, _to_b_rx) =
        bounded::<OrchestratorToPlanet>(10);
    let (_from_b_tx, from_b_rx) =
        bounded::<PlanetToOrchestrator>(10);
    orchestrator
        .add_planet(planet_b, to_b_tx, from_b_rx)
        .unwrap();

    orchestrator
        .state
        .add_adjacency(planet_a, planet_b);

    let explorer_id = ID::from(42u32);

    let (to_explorer_tx, _to_explorer_rx) =
        bounded::<OrchestratorToExplorer>(10);
    let (_from_explorer_tx, from_explorer_rx) =
        bounded::<ExplorerToOrchestrator<GenericResource>>(10);

    orchestrator
        .add_explorer(
            explorer_id,
            to_explorer_tx,
            from_explorer_rx,
            planet_a,
        )
        .unwrap();

    let result = orchestrator
        .state
        .move_explorer(explorer_id, planet_a, planet_b);

    assert!(result.is_ok());
    assert_eq!(
        orchestrator.state().explorer_location(explorer_id),
        Some(planet_b)
    );
}
