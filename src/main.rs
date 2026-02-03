use crossbeam_channel::{unbounded, Sender, Receiver};
use std::thread;
use common_game::utils::ID;
use orchestrator::SystemState;
use orchestrator::Orchestrator;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};

// Pianeti come librerie
use orbitron;
use skycartel;
use rustrelli;
use the_compiler_strikes_back;
use planet as crabtorio;
use Planet as houston;
use enterprise;

fn main() {
    // Sistema dello stato del gioco
    let mut system_state = SystemState::new();

    // Lista di pianeti e thread handles
    let mut handles = vec![];

    // Struttura per tenere i canali pianeta <-> orchestratore
    struct PlanetChannels {
        tx_to_orchestrator: Sender<PlanetToOrchestrator>,
        rx_from_orchestrator: Receiver<OrchestratorToPlanet>,
    }
    let mut planet_channels: std::collections::HashMap<ID, PlanetChannels> = std::collections::HashMap::new();

    // Funzione helper per spawnare un pianeta
    fn spawn_planet<F>(
        id: ID,
        start_fn: F,
        planet_channels: &mut std::collections::HashMap<ID, PlanetChannels>,
    ) -> thread::JoinHandle<()>
    where
        F: FnOnce(Sender<PlanetToOrchestrator>, Receiver<OrchestratorToPlanet>) + Send + 'static,
    {
        let (tx_to_orchestrator, rx_from_planet) = unbounded();
        let (tx_to_planet, rx_from_orchestrator) = unbounded();

        planet_channels.insert(id, PlanetChannels {
            tx_to_orchestrator: tx_to_orchestrator.clone(),
            rx_from_orchestrator,
        });

        thread::spawn(move || {
            start_fn(tx_to_orchestrator, rx_from_orchestrator);
        })
    }

    // Avvio dei pianeti
    handles.push(spawn_planet(1, orbitron::start, &mut planet_channels));
    handles.push(spawn_planet(2, skycartel::start, &mut planet_channels));
    handles.push(spawn_planet(3, rustrelli::start, &mut planet_channels));
    handles.push(spawn_planet(4, the_compiler_strikes_back::start, &mut planet_channels));
    handles.push(spawn_planet(5, crabtorio::start, &mut planet_channels));
    handles.push(spawn_planet(6, houston::start, &mut planet_channels));
    handles.push(spawn_planet(7, enterprise::start, &mut planet_channels));

    // === Aggiungi esploratori nello stato ===
    let explorers = vec![(101, 1), (102, 4)];
    for (eid, pid) in &explorers {
        system_state.add_explorer(*eid, *pid).expect("Errore nell'aggiunta dell'esploratore");
    }

    // === Avvio orchestratore ===
    let mut orchestrator = Orchestrator::new(system_state);

    println!("Sistema inizializzato, pianeti e esploratori avviati!");

    // === Loop principale orchestratore (esempio semplice) ===
    loop {
        for (pid, channels) in &planet_channels {
            while let Ok(msg) = channels.tx_to_orchestrator.try_recv() {
                println!("Messaggio dal pianeta {}: {:?}", pid, msg);
            }
        }
        // Breve sleep per non bloccare la CPU
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Se vuoi attendere la fine dei thread dei pianeti
    // for handle in handles {
    //     handle.join().unwrap();
    // }
}
