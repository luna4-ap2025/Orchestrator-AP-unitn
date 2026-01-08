mod explorer;
mod state;
mod movement;
mod logging;

use explorer::Explorer;

fn main() -> Result<(), String> {
    logging::init();

    let mut explorer = Explorer::new(1);
    explorer.run();

    Ok(())
}
