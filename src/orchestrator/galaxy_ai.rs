//! Galaxy AI that makes decisions about sending asteroids and sunrays.
//!
//! The Galaxy AI observes the current state of alive planets each cycle
//! and decides whether to send asteroids or sunrays to specific planets.

use common_game::utils::ID;
use rand::Rng;

/// Actions that the Galaxy AI can take
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GalaxyAction {
    /// Send an asteroid to attack a planet
    SendAsteroid {
        /// Target planet ID
        target_planet: ID
    },
    /// Send a sunray to help a planet
    SendSunray {
        /// Target planet ID
        target_planet: ID
    },
    /// Do nothing this cycle
    DoNothing,
}

/// Strategy that the Galaxy AI uses to make decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIStrategy {
    /// Always attacks (sends asteroids)
    Aggressive,
    /// Always helps (sends sunrays)
    Benevolent,
    /// Randomly chooses between asteroid, sunray, or nothing
    Random,
    /// Alternates between attacking and helping
    Balanced,
    /// Does nothing (passive)
    Passive,
}

/// Galaxy AI that observes the game state and makes decisions
#[derive(Debug, Clone)]
pub struct GalaxyAI {
    /// Current intention/action to take
    current_intention: Option<GalaxyAction>,
    /// Strategy being used
    strategy: AIStrategy,
    /// Cycle counter for balanced strategy
    cycle_count: u32,
    /// Probability of taking action (0.0 to 1.0)
    action_probability: f32,
}

impl GalaxyAI {
    /// Creates a new Galaxy AI with the specified strategy
    #[must_use]
    pub fn new(strategy: AIStrategy) -> Self {
        Self {
            current_intention: None,
            strategy,
            cycle_count: 0,
            action_probability: 0.3, // 30% chance per cycle by default
        }
    }

    /// Creates an aggressive AI (always attacks)
    #[must_use]
    pub fn aggressive() -> Self {
        Self::new(AIStrategy::Aggressive)
    }

    /// Creates a benevolent AI (always helps)
    #[must_use]
    pub fn benevolent() -> Self {
        Self::new(AIStrategy::Benevolent)
    }

    /// Creates a random AI
    #[must_use]
    pub fn random() -> Self {
        Self::new(AIStrategy::Random)
    }

    /// Creates a balanced AI (alternates)
    #[must_use]
    pub fn balanced() -> Self {
        Self::new(AIStrategy::Balanced)
    }

    /// Creates a passive AI (does nothing)
    #[must_use]
    pub fn passive() -> Self {
        Self::new(AIStrategy::Passive)
    }

    /// Sets the probability of taking action each cycle (0.0 to 1.0)
    #[must_use]
    pub fn with_action_probability(mut self, probability: f32) -> Self {
        self.action_probability = probability.clamp(0.0, 1.0);
        self
    }

    /// Updates the AI with current game state and returns the next action
    ///
    /// # Arguments
    /// * `alive_planets` - List of currently alive planet IDs
    ///
    /// # Returns
    /// The action to take this cycle (if any)
    pub fn update(&mut self, alive_planets: &[ID]) -> GalaxyAction {
        self.cycle_count += 1;

        // If no planets alive, do nothing
        if alive_planets.is_empty() {
            self.current_intention = Some(GalaxyAction::DoNothing);
            return GalaxyAction::DoNothing;
        }

        // Decide action based on strategy
        let action = match self.strategy {
            AIStrategy::Aggressive => self.decide_aggressive(alive_planets),
            AIStrategy::Benevolent => self.decide_benevolent(alive_planets),
            AIStrategy::Random => self.decide_random(alive_planets),
            AIStrategy::Balanced => self.decide_balanced(alive_planets),
            AIStrategy::Passive => GalaxyAction::DoNothing,
        };

        self.current_intention = Some(action.clone());
        action
    }

    /// Aggressive strategy: attack random planet
    fn decide_aggressive(&self, alive_planets: &[ID]) -> GalaxyAction {
        if self.should_take_action() {
            let target = self.pick_random_planet(alive_planets);
            GalaxyAction::SendAsteroid {
                target_planet: target,
            }
        } else {
            GalaxyAction::DoNothing
        }
    }

    /// Benevolent strategy: help random planet
    fn decide_benevolent(&self, alive_planets: &[ID]) -> GalaxyAction {
        if self.should_take_action() {
            let target = self.pick_random_planet(alive_planets);
            GalaxyAction::SendSunray {
                target_planet: target,
            }
        } else {
            GalaxyAction::DoNothing
        }
    }

    /// Random strategy: randomly pick asteroid, sunray, or nothing
    fn decide_random(&self, alive_planets: &[ID]) -> GalaxyAction {
        if !self.should_take_action() {
            return GalaxyAction::DoNothing;
        }

        let mut rng = rand::thread_rng();
        let choice = rng.gen_range(0..3);
        let target = self.pick_random_planet(alive_planets);

        match choice {
            0 => GalaxyAction::SendAsteroid {
                target_planet: target,
            },
            1 => GalaxyAction::SendSunray {
                target_planet: target,
            },
            _ => GalaxyAction::DoNothing,
        }
    }

    /// Balanced strategy: alternate between asteroid and sunray
    fn decide_balanced(&self, alive_planets: &[ID]) -> GalaxyAction {
        if !self.should_take_action() {
            return GalaxyAction::DoNothing;
        }

        let target = self.pick_random_planet(alive_planets);

        if self.cycle_count % 2 == 0 {
            GalaxyAction::SendAsteroid {
                target_planet: target,
            }
        } else {
            GalaxyAction::SendSunray {
                target_planet: target,
            }
        }
    }

    /// Determines if action should be taken based on probability
    fn should_take_action(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() < self.action_probability
    }

    /// Picks a random planet from the list
    fn pick_random_planet(&self, alive_planets: &[ID]) -> ID {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..alive_planets.len());
        alive_planets[index]
    }

    /// Gets the current intention
    #[must_use]
    pub fn current_intention(&self) -> Option<&GalaxyAction> {
        self.current_intention.as_ref()
    }

    /// Gets the current strategy
    #[must_use]
    pub fn strategy(&self) -> AIStrategy {
        self.strategy
    }

    /// Changes the AI strategy
    pub fn set_strategy(&mut self, strategy: AIStrategy) {
        self.strategy = strategy;
    }

    /// Gets the cycle count
    #[must_use]
    pub fn cycle_count(&self) -> u32 {
        self.cycle_count
    }
}

impl Default for GalaxyAI {
    fn default() -> Self {
        Self::random()
    }
}