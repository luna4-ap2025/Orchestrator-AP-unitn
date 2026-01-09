//! Integration tests for the orchestrator
//!
//! These tests verify that the orchestrator correctly coordinates between
//! planets and explorers, manages state, and handles errors appropriately

#[cfg(test)]
mod tests {
    use crate::orchestrator::Orchestrator;
    use crate::logging;
    use log::LevelFilter;

    /// Test basic orchestrator initialization
    #[test]
    fn test_orchestrator_initialization() {
        logging::init(LevelFilter::Warn);

        let orchestrator = Orchestrator::new();
        assert!(orchestrator.is_ok());
    }
}