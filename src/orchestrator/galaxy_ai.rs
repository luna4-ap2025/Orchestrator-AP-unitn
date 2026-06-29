//! Galaxy AI that makes decisions about sending asteroids and sunrays.
//!
//! The Galaxy AI works with a phase system: each phase lasts for a random amount of cycles
//! and determines the percent chance that a random planet will receive a sunray, asteroid, or nothing.
//! Once the phase length ends, a new phase is chosen randomly.

use common_game::utils::ID;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Actions that the Galaxy AI can take
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GalaxyAction {
    /// Send an asteroid to attack a planet
    SendAsteroid { target_planet: ID },
    /// Send a sunray to help a planet
    SendSunray { target_planet: ID },
    /// Do nothing this cycle
    DoNothing,
}

/// Strategy that the Galaxy AI uses to make decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIPhase {
    /// 50% sunrays, 10% asteroids, 40% nothing
    Prosperous,
    /// 10% sunrays, 50% asteroids, 40% nothing
    Destructive,
    /// 40% sunrays, 60% asteroids
    Chaotic,
    /// 30% sunrays, 10% asteroids, 60% nothing
    Calm,
    /// 100% nothing, default phase
    Dormant,
}

/// Galaxy AI that decides which planets get either a sunray or an asteroid, mostly random
#[derive(Debug, Clone)]
pub struct GalaxyAI {
    /// Current intention/action to take
    current_intention: GalaxyAction,
    /// Strategy being used
    phase: AIPhase,
    /// Cycle counter for balanced strategy
    current_phase_length: u32,
    /// Should change phase on update
    phase_change: bool,
}

impl GalaxyAI {
    /// Creates a new Galaxy AI that is dormant and will NOT wake in the next cycle
    #[must_use]
    pub fn new_inactive() -> Self {
        Self {
            current_intention: GalaxyAction::DoNothing,
            phase: AIPhase::Dormant,
            current_phase_length: 0,
            phase_change: false,
        }
    }

    /// Creates a new Galaxy AI that is dormant and will wake in the next cycle
    #[must_use]
    pub fn new_active() -> Self {
        Self {
            current_intention: GalaxyAction::DoNothing,
            phase: AIPhase::Dormant,
            current_phase_length: 0,
            phase_change: true,
        }
    }

    /// Set the current phase with a random length and active phase change
    fn change_phase_random(&mut self) {
        for _ in 0..1000 {
            let new_phase = Self::random_phase();
            if new_phase != self.phase {
                self.phase = new_phase;
                self.current_phase_length = Self::random_phase_length();
                self.phase_change = true;
                break;
            }
        }
    }

    /// Set the current phase with a custom phase length and phase change
    pub fn set_ai(&mut self, phase: AIPhase, phase_length: u32, phase_change: bool) {
        self.phase = phase;
        self.current_phase_length = phase_length;
        self.phase_change = phase_change;
    }

    /// Enable the AI by setting phase change to true and phase length to 0
    pub fn enable_ai(&mut self) {
        self.set_ai(AIPhase::Dormant, 0, true);
    }

    /// Disable the AI by setting the phase change to false and the phase to dormant
    pub fn disable_ai(&mut self) {
        self.set_ai(AIPhase::Dormant, 0, false);
    }

    /// Change current intention
    fn set_intention(&mut self, intention: GalaxyAction) {
        self.current_intention = intention;
    }

    /// Generate a phase length between 100 and 1000 cycles
    fn random_phase_length() -> u32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(100..=1000)
    }

    /// Selects an optional random planet ID from the given list
    fn select_random_planet(planet_list: &[ID]) -> Option<ID> {
        let mut rng = rand::thread_rng();
        planet_list.choose(&mut rng).copied()
    }

    /// Generate a random action with parameters
    fn random_action_integer_percent(
        send_sunray: u32,
        send_asteroid: u32,
        do_nothing: u32,
        planet_list: &[ID],
    ) -> GalaxyAction {
        if send_sunray + send_asteroid + do_nothing != 100 {
            log::error!("Total action percentage chance is invalid");
            GalaxyAction::DoNothing
        } else if let Some(planet_id) = Self::select_random_planet(planet_list) {
            let mut rng = rand::thread_rng();
            let roll = rng.gen_range(0..100);
            if roll < send_sunray {
                GalaxyAction::SendSunray { target_planet: planet_id }
            } else if roll < send_sunray + send_asteroid {
                GalaxyAction::SendAsteroid { target_planet: planet_id }
            } else {
                GalaxyAction::DoNothing
            }
        } else {
            GalaxyAction::DoNothing
        }
    }

    /// Generate a random phase, dormant phase only if RNG fails
    fn random_phase() -> AIPhase {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => AIPhase::Prosperous,
            1 => AIPhase::Destructive,
            2 => AIPhase::Chaotic,
            3 => AIPhase::Calm,
            _ => AIPhase::Dormant,
        }
    }

    /// Update method to change phase and intentions every cycle based on current planet list
    pub fn update(&mut self, planet_list: &[ID]) {
        if self.phase_change {
            if self.current_phase_length == 0 {
                self.change_phase_random();
            } else {
                self.current_phase_length -= 1;
            }
        }

        match self.phase {
            AIPhase::Dormant => self.set_intention(GalaxyAction::DoNothing),
            AIPhase::Prosperous => self.set_intention(Self::random_action_integer_percent(50, 10, 40, planet_list)),
            AIPhase::Destructive => self.set_intention(Self::random_action_integer_percent(10, 50, 40, planet_list)),
            AIPhase::Chaotic => self.set_intention(Self::random_action_integer_percent(40, 60, 0, planet_list)),
            AIPhase::Calm => self.set_intention(Self::random_action_integer_percent(30, 10, 60, planet_list)),
        }
    }

    /// Getters for internal variables
    pub fn get_phase(&self) -> AIPhase {
        self.phase
    }

    pub fn get_phase_as_str(&self) -> &str {
        match self.phase {
            AIPhase::Dormant => "Dormant",
            AIPhase::Prosperous => "Prosperous",
            AIPhase::Destructive => "Destructive",
            AIPhase::Chaotic => "Chaotic",
            AIPhase::Calm => "Calm",
        }
    }

    pub fn get_intention(&self) -> GalaxyAction {
        self.current_intention.clone()
    }

    pub fn get_current_phase_length(&self) -> u32 {
        self.current_phase_length
    }

    pub fn get_phase_change(&self) -> bool {
        self.phase_change
    }
}
