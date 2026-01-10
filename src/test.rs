//! Integration tests for the orchestrator
//!
//! These tests verify that the orchestrator correctly coordinates between
//! planets and explorers, manages state, and handles errors appropriately

#[cfg(test)]
mod tests {
    use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
    use crate::orchestrator::{Orchestrator};
    use crate::logging;
    use log::LevelFilter;
    use common_game::utils::ID;
    use common_game::protocols::orchestrator_planet::{
        OrchestratorToPlanet,
        PlanetToOrchestrator,
    };
    use crossbeam_channel::bounded;
    use common_game::components::resource::GenericResource;
    use common_game::protocols::planet_explorer::{ExplorerToPlanet};

    /// Test basic orchestrator initialization
    #[test]
    fn test_orchestrator_initialization() {
        logging::init(LevelFilter::Warn);

        let orchestrator = Orchestrator::new();
        assert!(orchestrator.is_ok());
    }

    #[test]
    fn test_add_seven_planets() {
        let mut orchestrator = Orchestrator::new().expect("Orchestrator should initialize");

        //add 7 fake planets
        for i in 0..7 {
            let planet_id = ID::from(i as u32);

            let (to_planet_tx, _to_planet_rx) =
                bounded::<OrchestratorToPlanet>(10);
            let (_from_planet_tx, from_planet_rx) =
                bounded::<PlanetToOrchestrator>(10);

            orchestrator
                .add_planet(planet_id, to_planet_tx, from_planet_rx)
                .expect("Planet should be added successfully");

        }

        //are there 7 planets?
        let alive_planets = orchestrator.state().alive_planets();
        assert_eq!(alive_planets.len(), 7);
    }

    #[test]
    fn test_add_explorer_to_planet() {
        let mut orchestrator = Orchestrator::new()
            .expect("Orchestrator should initialize");

        //add planet
        let planet_id = ID::from(1u32);

        let (to_planet_tx, _to_planet_rx) =
            bounded::<OrchestratorToPlanet>(10);
        let (_from_planet_tx, from_planet_rx) =
            bounded::<PlanetToOrchestrator>(10);

        orchestrator
            .add_planet(planet_id, to_planet_tx, from_planet_rx)
            .expect("Planet should be added");

        //add explorer
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
        let mut orchestrator = Orchestrator::new().expect("Orchestrator should initialize");

        let nonexistent_planet_id = ID::from(999u32);

        let explorer_id = ID::from(42u32);

        let (to_explorer_tx, _to_explorer_rx) = crossbeam_channel::bounded::<OrchestratorToExplorer>(10);
        let (_from_explorer_tx, from_explorer_rx) =
            crossbeam_channel::bounded::<ExplorerToOrchestrator<common_game::components::resource::GenericResource>>(10);

        let result = orchestrator.add_explorer(explorer_id, to_explorer_tx, from_explorer_rx, nonexistent_planet_id);

        assert!(result.is_err());

        assert!(orchestrator.state().explorer_location(explorer_id).is_none());
    }

    #[test]
    fn test_explorer_movement_between_planets() {
        logging::init(LevelFilter::Warn);

        let mut orchestrator = Orchestrator::new().expect("Orchestrator should initialize");

        //create 2 planets
        let planet_a = ID::from(1u32);
        let planet_b = ID::from(2u32);

        let (to_planet_a_tx, _to_planet_a_rx) = bounded::<OrchestratorToPlanet>(10);
        let (_from_planet_a_tx, from_planet_a_rx) = bounded::<PlanetToOrchestrator>(10);
        orchestrator
            .add_planet(planet_a, to_planet_a_tx, from_planet_a_rx)
            .expect("Planet A should be added");

        let (to_planet_b_tx, _to_planet_b_rx) = bounded::<OrchestratorToPlanet>(10);
        let (_from_planet_b_tx, from_planet_b_rx) = bounded::<PlanetToOrchestrator>(10);
        orchestrator
            .add_planet(planet_b, to_planet_b_tx, from_planet_b_rx)
            .expect("Planet B should be added");
        orchestrator.state.add_adjacency(planet_a, planet_b);

        //explorer on planet a
        let explorer_id = ID::from(42u32);
        let (to_explorer_tx, _to_explorer_rx) = bounded::<OrchestratorToExplorer>(10);
        let (_from_explorer_tx, from_explorer_rx) = bounded::<ExplorerToOrchestrator<common_game::components::resource::GenericResource>>(10);

        orchestrator
            .add_explorer(explorer_id, to_explorer_tx, from_explorer_rx, planet_a)
            .expect("Explorer should be added");

        assert_eq!(
            orchestrator.state().explorer_location(explorer_id),
            Some(planet_a)
        );

        //move explorer
        let result = orchestrator.state.move_explorer(explorer_id, planet_a, planet_b);

        assert!(
            result.is_ok(),
            "Explorer should be able to move between connected planets"
        );
        assert_eq!(
            orchestrator.state().explorer_location(explorer_id),
            Some(planet_b)
        )
    }

    #[test]
    fn test_send_asteroid_and_sunray() {
        let mut orchestrator = Orchestrator::new().expect("Orchestrator should initialize");

        let planet_id = ID::from(1u32);
        let (to_planet_tx, _to_planet_rx) = bounded::<OrchestratorToPlanet>(10);
        let (_from_planet_tx, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);

        orchestrator
            .add_planet(planet_id, to_planet_tx.clone(), from_planet_rx)
            .expect("Planet should be added");

        //send asteroid
        orchestrator.send_asteroid_to_planet(planet_id);
        assert_eq!(orchestrator.state().game_stats().asteroids_sent, 1);

        //send sunray
        orchestrator.send_sunray_to_planet(planet_id);
        assert_eq!(orchestrator.state().game_stats().sunrays_sent, 1);
    }

    #[test]
    fn test_planet_destruction_removes_explorers() {
        let mut orchestrator = Orchestrator::new().expect("Orchestrator should initialize");

        let planet_id = ID::from(1u32);

        let (to_planet_tx, _to_planet_rx) = bounded::<OrchestratorToPlanet>(10);
        let (_from_planet_tx, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);

        orchestrator.add_planet(planet_id, to_planet_tx, from_planet_rx).unwrap();

        let explorer_id = ID::from(101u32);
        let (to_explorer_tx, _to_explorer_rx) = bounded::<OrchestratorToExplorer>(10);
        let (_from_explorer_tx, from_explorer_rx) =
            bounded::<ExplorerToOrchestrator<common_game::components::resource::GenericResource>>(10);

        orchestrator
            .add_explorer(explorer_id, to_explorer_tx, from_explorer_rx, planet_id)
            .unwrap();

        //death star
        orchestrator.handle_planet_death(planet_id);

        assert!(!orchestrator.state().has_planet(planet_id));

        //all the rebels died ahahahahahah
        assert!(orchestrator.state().explorer_location(explorer_id).is_none());
    }

    #[test]
    fn test_asteroid_hits_planet() {
        let mut orchestrator = Orchestrator::new().expect("Orchestrator should initialize");

        let planet_id = ID::from(1u32);

        // Canali finti
        let (to_planet_tx, to_planet_rx) = bounded::<common_game::protocols::orchestrator_planet::OrchestratorToPlanet>(10);
        let (from_planet_tx, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);
        let (_to_explorer_tx, to_explorer_rx) = bounded::<ExplorerToPlanet>(10);

        let enterprise = enterprise::create_planet(planet_id, to_planet_rx, from_planet_tx, to_explorer_rx);
        orchestrator.add_planet(planet_id, to_planet_tx, from_planet_rx).unwrap();
        orchestrator.toggle_planet_ai(planet_id, true);

        // Pianeta deve essere rimosso
        assert!(!orchestrator.state().has_planet(planet_id));
    }

    #[test]
    fn test_orchestrator_run() {
        use std::thread;
        use std::time::Duration;
        use crossbeam_channel::bounded;
        use common_game::utils::ID;
        use crate::orchestrator::Orchestrator;
        use crate::logging;
        use log::LevelFilter;
        use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
        use common_game::protocols::orchestrator_explorer::{OrchestratorToExplorer, ExplorerToOrchestrator};
        use common_game::components::resource::GenericResource;
        use std::sync::Arc;

        logging::init(LevelFilter::Warn);

        let mut orchestrator = Orchestrator::new().expect("Orchestrator should initialize");

        // Aggiungiamo un pianeta finto
        let planet_id = ID::from(1u32);
        let (to_planet_tx, _to_planet_rx) = bounded::<OrchestratorToPlanet>(10);
        let (_from_planet_tx, from_planet_rx) = bounded::<PlanetToOrchestrator>(10);
        orchestrator.add_planet(planet_id, to_planet_tx, from_planet_rx)
            .expect("Planet should be added");

        // Aggiungiamo un esploratore finto
        let explorer_id = ID::from(42u32);
        let (to_explorer_tx, _to_explorer_rx) = bounded::<OrchestratorToExplorer>(10);
        let (_from_explorer_tx, from_explorer_rx) = bounded::<ExplorerToOrchestrator<GenericResource>>(10);
        orchestrator.add_explorer(explorer_id, to_explorer_tx, from_explorer_rx, planet_id)
            .expect("Explorer should be added");

        // Creiamo un clone del flag di stop
        let stop_flag = Arc::clone(&orchestrator.should_stop);

        // Lanciamo il run in un thread
        let handle = thread::spawn(move || {
            orchestrator.run().expect("Orchestrator run should not panic");
        });

        // Facciamo girare qualche ciclo
        thread::sleep(Duration::from_millis(50));

        // Stop immediato
        stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);

        // Aspettiamo che il thread termini
        handle.join().expect("Orchestrator thread should finish");
    }



}