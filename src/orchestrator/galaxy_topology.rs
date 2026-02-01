//! Galaxy topology definitions for automated galaxy setup.

/// Defines how planets are connected in the galaxy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalaxyTopology {
    /// Ring topology: Each planet connected to its two neighbors
    /// Example (7 planets): 0-1-2-3-4-5-6-0
    /// Each planet has exactly 2 neighbors
    Ring,

    /// Fully connected: Every planet connected to every other planet
    /// Example (7 planets): All 21 possible connections
    /// Each planet has 6 neighbors
    FullyConnected,

    /// Star topology: One central planet connected to all others
    /// Example (7 planets): 0 at center, connected to 1,2,3,4,5,6
    /// Center has 6 neighbors, others have 1 neighbor
    Star,

    /// Line topology: Linear chain of planets
    /// Example (7 planets): 0-1-2-3-4-5-6
    /// Endpoints have 1 neighbor, others have 2
    Line,

    /// Hub topology: Two hub planets, others connect to nearest hub
    /// Example (7 planets): Hub at 0 and 3
    /// 0 connects to 1,2,3 and 3 connects to 4,5,6
    Hub,
}

impl GalaxyTopology {
    /// Returns a description of the topology
    #[must_use]
    pub fn description(&self) -> &str {
        match self {
            Self::Ring => "Each planet connected to two neighbors in a ring",
            Self::FullyConnected => "Every planet connected to every other planet",
            Self::Star => "One central planet connected to all others",
            Self::Line => "Linear chain of planets",
            Self::Hub => "Two hub planets connecting groups of planets",
        }
    }

    /// Returns the average number of connections per planet for 7 planets
    #[must_use]
    pub fn avg_connections_for_7_planets(&self) -> f32 {
        match self {
            Self::Ring => 2.0,
            Self::FullyConnected => 6.0,
            Self::Star => 12.0 / 7.0, // (6 + 1 + 1 + 1 + 1 + 1 + 1) / 7
            Self::Line => 12.0 / 7.0, // (1 + 2 + 2 + 2 + 2 + 2 + 1) / 7
            Self::Hub => 16.0 / 7.0,  // varies
        }
    }
}

impl std::fmt::Display for GalaxyTopology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ring => write!(f, "Ring"),
            Self::FullyConnected => write!(f, "Fully Connected"),
            Self::Star => write!(f, "Star"),
            Self::Line => write!(f, "Line"),
            Self::Hub => write!(f, "Hub"),
        }
    }
}