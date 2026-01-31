//! Main orchestrator module.
//!
//! This module contains all the components needed to coordinate the space
//! simulation, including planet management, explorer control, message routing,
//! and GUI integration.

pub mod routing;
pub mod state;
pub mod planet_control;
pub mod explorer_control;
pub mod gui_interface;



mod orchestrator;


pub use orchestrator::Orchestrator;
pub use state::SystemState;
pub use gui_interface::{GuiEvent, GuiState};