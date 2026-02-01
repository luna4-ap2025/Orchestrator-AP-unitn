//! Integration tests for the orchestrator

use crossbeam_channel::bounded;

use common_game::components::resource::GenericResource;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::orchestrator_planet::{
    OrchestratorToPlanet, PlanetToOrchestrator,
};
use common_game::utils::ID;

use orchestrator::Orchestrator;

// ==================== Helper Functions ====================

fn create_test_orchestrator() -> Orchestrator {
    Orchestrator::new().expect("Failed to create orchestrator")
}

fn add_test_planet(orchestrator: &mut Orchestrator, planet_id: ID) {
    let (to_planet_tx, _to_planet_rx) = bounded::<OrchestratorToPlanet>(10);
    let (_from_planet_tx, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);

    orchestrator
        .add_planet(planet_id, to_planet_tx, from_planet_rx)
        .expect("Failed to add planet");
}

fn add_test_explorer(
    orchestrator: &mut Orchestrator,
    explorer_id: ID,
    planet_id: ID,
) -> (
    crossbeam_channel::Receiver<OrchestratorToExplorer>,
    crossbeam_channel::Sender<ExplorerToOrchestrator<GenericResource>>,
) {
    let (to_explorer_tx, to_explorer_rx) = bounded::<OrchestratorToExplorer>(10);
    let (from_explorer_tx, from_explorer_rx) =
        bounded::<ExplorerToOrchestrator<GenericResource>>(10);

    orchestrator
        .add_explorer(explorer_id, to_explorer_tx, from_explorer_rx, planet_id)
        .expect("Failed to add explorer");

    (to_explorer_rx, from_explorer_tx)
}

// ==================== Orchestrator Initialization Tests ====================

#[test]
fn test_orchestrator_initialization() {
    let orchestrator = Orchestrator::new();
    assert!(orchestrator.is_ok(), "Orchestrator should initialize successfully");
}

#[test]
fn test_orchestrator_default() {
    let orchestrator = Orchestrator::default();
    assert_eq!(orchestrator.state().alive_planets().len(), 0);
}

// ==================== Planet Management Tests ====================

#[test]
fn test_add_single_planet() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    add_test_planet(&mut orchestrator, planet_id);

    assert!(orchestrator.state().is_planet_alive(planet_id));
    assert_eq!(orchestrator.state().alive_planets().len(), 1);
}

#[test]
fn test_add_multiple_planets() {
    let mut orchestrator = create_test_orchestrator();

    for i in 0..7 {
        add_test_planet(&mut orchestrator, ID::from(i));
    }

    assert_eq!(orchestrator.state().alive_planets().len(), 7);
}

#[test]
fn test_add_duplicate_planet_fails() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    add_test_planet(&mut orchestrator, planet_id);

    let (to_planet_tx, _) = bounded::<OrchestratorToPlanet>(10);
    let (_, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);

    let result = orchestrator.add_planet(planet_id, to_planet_tx, from_planet_rx);
    assert!(result.is_err(), "Adding duplicate planet should fail");
}

#[test]
fn test_destroy_planet() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    add_test_planet(&mut orchestrator, planet_id);
    assert!(orchestrator.state().is_planet_alive(planet_id));

    orchestrator.destroy_planet(planet_id);
    assert!(!orchestrator.state().is_planet_alive(planet_id));
}

#[test]
fn test_destroy_planet_kills_explorers() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);
    let explorer_id = ID::from(42u32);

    add_test_planet(&mut orchestrator, planet_id);
    add_test_explorer(&mut orchestrator, explorer_id, planet_id);

    assert_eq!(orchestrator.state().explorer_location(explorer_id), Some(planet_id));

    orchestrator.destroy_planet(planet_id);

    assert_eq!(orchestrator.state().explorer_location(explorer_id), None);
    assert_eq!(orchestrator.state().game_stats().explorers_killed, 1);
}

// ==================== Explorer Management Tests ====================

#[test]
fn test_add_explorer_to_planet() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);
    let explorer_id = ID::from(42u32);

    add_test_planet(&mut orchestrator, planet_id);
    add_test_explorer(&mut orchestrator, explorer_id, planet_id);

    assert_eq!(orchestrator.state().explorer_location(explorer_id), Some(planet_id));
}

#[test]
fn test_add_explorer_to_dead_planet_fails() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(999u32);
    let explorer_id = ID::from(42u32);

    let (to_explorer_tx, _) = bounded::<OrchestratorToExplorer>(10);
    let (_, from_explorer_rx) = bounded::<ExplorerToOrchestrator<GenericResource>>(10);

    let result = orchestrator.add_explorer(
        explorer_id,
        to_explorer_tx,
        from_explorer_rx,
        planet_id,
    );

    assert!(result.is_err(), "Adding explorer to dead planet should fail");
}

#[test]
fn test_kill_explorer() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);
    let explorer_id = ID::from(42u32);

    add_test_planet(&mut orchestrator, planet_id);
    add_test_explorer(&mut orchestrator, explorer_id, planet_id);

    assert!(orchestrator.state().explorer_location(explorer_id).is_some());

    orchestrator.kill_explorer(explorer_id);

    assert!(orchestrator.state().explorer_location(explorer_id).is_none());
    assert_eq!(orchestrator.state().game_stats().explorers_killed, 1);
}

#[test]
fn test_multiple_explorers_on_same_planet() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    add_test_planet(&mut orchestrator, planet_id);

    for i in 0..5 {
        add_test_explorer(&mut orchestrator, ID::from(100 + i), planet_id);
    }

    let explorers = orchestrator.state().get_explorers_on_planet(planet_id);
    assert_eq!(explorers.len(), 5);
}

// ==================== Adjacency/Graph Tests ====================

#[test]
fn test_add_adjacency() {
    let mut orchestrator = create_test_orchestrator();
    let planet_a = ID::from(1u32);
    let planet_b = ID::from(2u32);

    add_test_planet(&mut orchestrator, planet_a);
    add_test_planet(&mut orchestrator, planet_b);

    orchestrator.state.add_adjacency(planet_a, planet_b).unwrap();

    assert!(orchestrator.state().is_adjacent(planet_a, planet_b));
    assert!(orchestrator.state().is_adjacent(planet_b, planet_a));
}

#[test]
fn test_get_neighbors() {
    let mut orchestrator = create_test_orchestrator();
    let center = ID::from(0u32);

    add_test_planet(&mut orchestrator, center);

    for i in 1..=4 {
        let neighbor = ID::from(i);
        add_test_planet(&mut orchestrator, neighbor);
        orchestrator.state.add_adjacency(center, neighbor).unwrap();
    }

    let neighbors = orchestrator.state().get_neighbors(center);
    assert_eq!(neighbors.len(), 4);
}

#[test]
fn test_adjacency_with_dead_planet_fails() {
    let mut orchestrator = create_test_orchestrator();
    let planet_a = ID::from(1u32);
    let planet_b = ID::from(999u32); // Dead planet

    add_test_planet(&mut orchestrator, planet_a);

    let result = orchestrator.state.add_adjacency(planet_a, planet_b);
    assert!(result.is_err());
}

// ==================== Explorer Movement Tests ====================

#[test]
fn test_explorer_movement_between_adjacent_planets() {
    let mut orchestrator = create_test_orchestrator();
    let planet_a = ID::from(1u32);
    let planet_b = ID::from(2u32);
    let explorer_id = ID::from(42u32);

    add_test_planet(&mut orchestrator, planet_a);
    add_test_planet(&mut orchestrator, planet_b);
    orchestrator.state.add_adjacency(planet_a, planet_b).unwrap();

    add_test_explorer(&mut orchestrator, explorer_id, planet_a);

    let result = orchestrator.state.move_explorer(explorer_id, planet_a, planet_b);
    assert!(result.is_ok());
    assert_eq!(orchestrator.state().explorer_location(explorer_id), Some(planet_b));
}

#[test]
fn test_explorer_movement_to_non_adjacent_fails() {
    let mut orchestrator = create_test_orchestrator();
    let planet_a = ID::from(1u32);
    let planet_b = ID::from(2u32);
    let explorer_id = ID::from(42u32);

    add_test_planet(&mut orchestrator, planet_a);
    add_test_planet(&mut orchestrator, planet_b);
    // NO adjacency added

    add_test_explorer(&mut orchestrator, explorer_id, planet_a);

    let result = orchestrator.state.move_explorer(explorer_id, planet_a, planet_b);
    assert!(result.is_err());
    assert_eq!(orchestrator.state().explorer_location(explorer_id), Some(planet_a));
}

// ==================== Asteroid/Sunray Tests ====================

#[test]
fn test_send_asteroid_to_planet() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    let (to_planet_tx, to_planet_rx) = bounded::<OrchestratorToPlanet>(10);
    let (_, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);

    orchestrator
        .add_planet(planet_id, to_planet_tx, from_planet_rx)
        .unwrap();

    let result = orchestrator.send_asteroid_to_planet(planet_id);
    assert!(result.is_ok());

    // Verify asteroid was sent
    let msg = to_planet_rx.try_recv();
    assert!(msg.is_ok());
    assert!(matches!(msg.unwrap(), OrchestratorToPlanet::Asteroid(_)));

    // Verify stats
    assert_eq!(orchestrator.state().game_stats().asteroids_sent, 1);
}

#[test]
fn test_send_asteroid_to_dead_planet_fails() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(999u32);

    let result = orchestrator.send_asteroid_to_planet(planet_id);
    assert!(result.is_err());
}

#[test]
fn test_send_sunray_to_planet() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    let (to_planet_tx, to_planet_rx) = bounded::<OrchestratorToPlanet>(10);
    let (_, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);

    orchestrator
        .add_planet(planet_id, to_planet_tx, from_planet_rx)
        .unwrap();

    orchestrator.send_sunray_to_planet(planet_id);

    // Verify sunray was sent
    let msg = to_planet_rx.try_recv();
    assert!(msg.is_ok());
    assert!(matches!(msg.unwrap(), OrchestratorToPlanet::Sunray(_)));

    // Verify stats
    assert_eq!(orchestrator.state().game_stats().sunrays_sent, 1);
}

// ==================== Game State Tests ====================

#[test]
fn test_game_starts_in_running_state() {
    let orchestrator = create_test_orchestrator();
    assert!(orchestrator.state().is_running());
    assert!(!orchestrator.state().is_paused());
    assert!(!orchestrator.state().is_ended());
}

#[test]
fn test_pause_and_resume_game() {
    let mut orchestrator = create_test_orchestrator();

    orchestrator.state.pause();
    assert!(orchestrator.state().is_paused());
    assert!(!orchestrator.state().is_running());

    orchestrator.state.resume();
    assert!(orchestrator.state().is_running());
    assert!(!orchestrator.state().is_paused());
}

#[test]
fn test_game_ends_when_all_planets_destroyed() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    add_test_planet(&mut orchestrator, planet_id);
    assert!(orchestrator.state().should_continue());

    orchestrator.state.remove_planet(planet_id);
    assert!(orchestrator.state().is_ended());
    assert!(!orchestrator.state().should_continue());
}

// ==================== Statistics Tests ====================

#[test]
fn test_asteroid_statistics() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    let (to_planet_tx, _) = bounded::<OrchestratorToPlanet>(10);
    let (_, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);
    orchestrator.add_planet(planet_id, to_planet_tx, from_planet_rx).unwrap();

    for _ in 0..5 {
        let _ = orchestrator.send_asteroid_to_planet(planet_id);
    }

    assert_eq!(orchestrator.state().game_stats().asteroids_sent, 5);
}

#[test]
fn test_planet_destruction_statistics() {
    let mut orchestrator = create_test_orchestrator();

    for i in 0..3 {
        add_test_planet(&mut orchestrator, ID::from(i));
    }

    orchestrator.destroy_planet(ID::from(0u32));
    orchestrator.destroy_planet(ID::from(1u32));

    assert_eq!(orchestrator.state().game_stats().planets_destroyed, 2);
}

// ==================== Integration Tests ====================

#[test]
fn test_complete_galaxy_setup() {
    let mut orchestrator = create_test_orchestrator();

    // Create 7 planets
    for i in 0..7 {
        add_test_planet(&mut orchestrator, ID::from(i));
    }

    // Create ring topology
    for i in 0..7 {
        let next = (i + 1) % 7;
        orchestrator.state.add_adjacency(ID::from(i), ID::from(next)).unwrap();
    }

    // Add explorers
    for i in 0..7 {
        add_test_explorer(&mut orchestrator, ID::from(100 + i), ID::from(i));
    }

    assert_eq!(orchestrator.state().alive_planets().len(), 7);
    assert_eq!(orchestrator.state().explorer_locations().len(), 7);

    // Each planet should have 2 neighbors (ring)
    for i in 0..7 {
        assert_eq!(orchestrator.state().get_neighbors(ID::from(i)).len(), 2);
    }
}

#[test]
fn test_cascading_planet_destruction() {
    let mut orchestrator = create_test_orchestrator();
    let planet_id = ID::from(1u32);

    add_test_planet(&mut orchestrator, planet_id);

    // Add 10 explorers on the planet
    for i in 0..10 {
        add_test_explorer(&mut orchestrator, ID::from(100 + i), planet_id);
    }

    assert_eq!(orchestrator.state().get_explorers_on_planet(planet_id).len(), 10);

    // Destroy planet
    orchestrator.destroy_planet(planet_id);

    // All explorers should be dead
    assert_eq!(orchestrator.state().explorer_locations().len(), 0);
    assert_eq!(orchestrator.state().game_stats().explorers_killed, 10);
    assert_eq!(orchestrator.state().game_stats().planets_destroyed, 1);
}