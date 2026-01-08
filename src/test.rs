mod orchestrator;
mod state;
mod routing;
mod planet_control;
mod explorer_control;
mod logging;
use orchestrator::Orchestrator;

fn main() -> Result<(), String> {
    logging::init();

    let mut orchestrator = Orchestrator::new()?;
    orchestrator.run();

    Ok(())
}
