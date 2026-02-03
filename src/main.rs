use orchestrator::{SystemState, Orchestrator, GalaxyStructure};
use common_game::utils::ID;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    // ===== File di inizializzazione =====
    // Deve contenere: pianeti e le loro connessioni (ID separati da spazi)
    // Esempio contenuto per 7 pianeti:
    // 1 2 3
    // 2 1 4
    // 3 1 4
    // 4 2 3 5
    // 5 4 6 7
    // 6 5
    // 7 5
    let init_file_path = "init_galaxy.txt";

    // Legge tutte le linee dal file
    let planet_lines: Vec<String> =
        read_lines(init_file_path).expect("Errore nella lettura del file di inizializzazione");

    // ===== Inizializza lo stato del sistema =====
    let mut system_state = SystemState::new();

    // ===== Aggiunge pianeti e connessioni =====
    let galaxy_structure = GalaxyStructure::new_from_file(&planet_lines);
    for &planet_id in galaxy_structure.get_alive_planets() {
        // aggiunge pianeta nello state
        system_state.add_planet(planet_id);

        // aggiunge connessioni con pianeti adiacenti
        let adjacents = galaxy_structure
            .get_adjacents(planet_id)
            .iter()
            .copied()
            .collect::<Vec<ID>>();

        for &adj in &adjacents {
            system_state.add_adjacency(planet_id, adj);
        }
    }

    // ===== Aggiungi esploratori =====
    let explorers = vec![(101, 1), (102, 4)]; // (explorer_id, planet_id)
    for (eid, pid) in &explorers {
        system_state
            .add_explorer(*eid, *pid)
            .expect("Errore nell'aggiunta dell'esploratore");
    }

    // ===== Stampa stato iniziale =====
    println!("Pianeti vivi: {:?}", system_state.get_alive_planets_sorted());
    for (eid, pid) in &explorers {
        println!("Esploratore {} parte dal pianeta {}", eid, pid);
    }

    // ===== Avvia orchestratore in modalità base senza GUI =====
    let mut orchestrator = Orchestrator::new();

    println!("Sistema inizializzato correttamente!");
}

/// Legge tutte le linee di un file in un vettore di stringhe
fn read_lines<P>(filename: P) -> Result<Vec<String>, std::io::Error>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}
