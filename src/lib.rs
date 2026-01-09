//! # Orchestrator Library - Luna4 Group
//!
//! This library provides the main Orchestrator implementation for the AP galaxy simulation game.
//! The Orchestrator coordinates communication between planets and explorers, manages the
//! galaxy topology, and provides a GUI-friendly interface for visualization.
//!
//! ## Key Components
//!
//! - Orchestrator (orchestrator::Orchestrator): Main orchestrator struct
//! - SystemState (orchestrator::state::SystemState): Galaxy state tracking
//! - GUI integration through GuiEvent (orchestrator::gui_interface::GuiEvent) system

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod logging;
pub mod orchestrator;
mod test;

/// Re-export commonly used types for convenience
pub use orchestrator::Orchestrator;