//! Galaxy AI that makes decisions about sending asteroids and sunrays.
//!
//! The Galaxy AI works with a phase system: each phase lasts for a random amount of cycles
//! and determines the percent chance that a random planet will receive a sunray, asteroid or nothing,
//! once the phase has run for the amount of cycles that was previously decided it kis changed to a different random phase
//!
//! The phases and percent chances are as follows:
//!
//! - Prosperous phase:     the AI will send mostly sunrays and a few asteroids
//!                         50% chance for a sunray, 10% for an asteroid and 40% for nothing to happen
//!
//! - Destructive phase:    the AI will send mostly asteroids and a few sunrays
//!                         10% chance for a sunray, 50% for an asteroid and 40% for nothing to happen
//!
//! - Chaotic phase:        the AI will send mostly asteroids with some sunrays at an increased rate
//!                         40% chance for a sunray, 60% for an asteroid and 0& for nothing so that every cycle something happens
//!
//! - Calm phase:           the AI will send few sunrays and fewer asteroids while doing mostly nothing
//!                         30% chance for s sunray, 10% for an asteroid and 60% for nothing to happen
//!
//! - Dormant phase:        This phase exists mostly as a default state, while in it the AI will do nothing,
//!                         enabling the AI from rest means setting the phase length to 0 and the phase changed to enabled,
//!                         that way on the next update the AI will choose a random phase and start working,
//!                         with the phase change disabled the AI is also disabled. This phase cannot be selected randomly on update

use common_game::utils::ID;
use rand::Rng;
use crate::orchestrator::galaxy_ai::AIPhase::{Calm, Chaotic, Destructive, Dormant, Prosperous};
use crate::orchestrator::galaxy_ai::GalaxyAction::{DoNothing, SendAsteroid, SendSunray};
use serde::{Deserialize, Serialize};

/// Actions that the Galaxy AI can take
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GalaxyAction {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum AIPhase {
    /// 50% sunrays 10% asteroids 40% nothing
    Prosperous,
    /// 10% sunrays 50% asteroids 40% nothing
    Destructive,
    /// 40% sunrays 60% asteroids
    Chaotic,
    /// 30% sunrays 10% asteroids 60% nothing
    Calm,
    /// 100% nothing, default phase
    Dormant,
}

/// Galaxy AI that decides which planets get either a sunray or an asteroid, mostly random
#[derive(Debug, Clone)]
pub(crate) struct GalaxyAI {
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
    /// Creates a new Galaxy AI that is dormant and will wake in the next cycle
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_intention: DoNothing,
            phase: Dormant,
            current_phase_length: 0,
            phase_change: false,
        }
    }

    ///Set the current phase with a random length and active phase change
    fn change_phase_random(&mut self) {

        for _ in 0..1000  {
            let new_phase = Self::random_phase();
            if new_phase != self.phase{     //make sure new phase is not the same as the old
                self.phase = new_phase;
                self.current_phase_length = Self::random_phase_length();
                self.phase_change = true;

                break;
            }
        }
    }

    ///Set the current phase with a custom phase length and phase change
    pub fn set_ai(&mut self, phase: AIPhase, phase_length: u32, phase_change: bool) {
        self.phase = phase;
        self.current_phase_length = phase_length;
        self.phase_change = phase_change;
    }

    ///Enable the ai by setting phase change to true and phase length to 0
    pub fn enable_ai (&mut self) {
        self.set_ai(Dormant, 0, true);
    }

    ///Disable the ai by setting the phase change to false and the phase to dormant
    pub fn disable_ai (&mut self) {
        self.set_ai(Dormant, 0, false);
    }

    ///change current intention
    fn set_intention(&mut self, intention: GalaxyAction) {
        self.current_intention = intention;
    }

    ///Generate a phase length between 100 and 1000 cycles
    fn random_phase_length() -> u32 {
        rand::rng().random_range(100..1001) as u32
    }

    ///Selects an optional random planet ID from the given list
    fn select_random_planet (planet_list: &[ID]) -> Option<ID> {
        if planet_list.is_empty() {
            None
        }else {
            let index = rand::rng().random_range(0..planet_list.len());
            Some(planet_list[index])
        }
    }

    ///Generate a random action with parameters
    fn random_action_integer_percent(send_sunray: u32, send_asteroid: u32, do_nothing: u32, planet_list: &[ID]) -> GalaxyAction {
        if send_sunray + send_asteroid + do_nothing != 100 {
            DoNothing

        }else {
            let planet = Self::select_random_planet(planet_list);

            match planet {
                //If planet list is empty do nothing
                None => {
                    DoNothing
                }
                Some(planet_id) => {
                    let rng = rand::rng().random_range(0..101);

                    if rng <= send_sunray {
                        SendSunray {target_planet: planet_id}
                    }else if rng <= send_asteroid {
                        SendAsteroid {target_planet: planet_id}
                    }else {
                        DoNothing
                    }
                }
            }
        }
    }

    ///Generate a random phase, dormant phase is a result only in case of rng error
    fn random_phase() -> AIPhase {
        let rng = rand::rng().random_range(0..4);
        match rng {
            0 => Prosperous,
            1 => Destructive,
            2 => Chaotic,
            3 => Calm,
            _ => Dormant,
        }
    }

    ///Update method to change phase and intentions every cycle based on current planet list
    pub fn update (&mut self, planet_list: &[ID]) {
        //if phase change is true update the phase
        if self.get_phase_change() == &true {
            //if current phase length is 0 change phase and generate a new phase length, otherwise decrease length by 1
            if self.get_current_phase_length() == &0 {
                self.change_phase_random();
            }else {
                self.current_phase_length = self.current_phase_length - 1;
            }
        }

        //match current phase and change intention based on a predetermined set of percentage chances
        match self.phase{
            Dormant => {self.set_intention(GalaxyAction::DoNothing)},
            Prosperous => {self.set_intention(Self::random_action_integer_percent(50,10,40, planet_list)) },
            Destructive => {self.set_intention(Self::random_action_integer_percent(10,50,40, planet_list))},
            Chaotic => {self.set_intention(Self::random_action_integer_percent(40,60,0, planet_list))},
            Calm => {self.set_intention(Self::random_action_integer_percent(30,10,60, planet_list))},
        };
    }

    ///Get methods for internal variables
    pub fn get_phase(&self) -> &AIPhase {
        &self.phase
    }

    pub fn get_intention(&self) -> &GalaxyAction {
        &self.current_intention
    }

    pub fn get_current_phase_length(&self) -> &u32 {
        &self.current_phase_length
    }

    pub fn get_phase_change(&self) -> &bool {
        &self.phase_change
    }


}