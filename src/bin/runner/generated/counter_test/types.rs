// Types for machine: counter_test
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};
use std::fmt;
use std::convert::TryFrom;

// ── State Enum ───────────────────────────────────────────────────────────────
/// Represents the possible states of machine: counter_test
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum State {
    Goal,
    Counting,
    Initial,
}

impl State {
    /// Returns the lowercase string name of this state.
    pub const fn as_str(&self) -> &'static str {
        match self {
            State::Goal => "goal",
            State::Counting => "counting",
            State::Initial => "initial",
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for State {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "goal" => Ok(State::Goal),
            "counting" => Ok(State::Counting),
            "initial" => Ok(State::Initial),
            _ => Err(format!("unknown state: '{}'", value)),
        }
    }
}

impl TryFrom<String> for State {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}


// ── Persistent ───────────────────────────────────────────────────────────────
/// Holds all persistent state for machine: counter_test
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Persistent {
    /// Current state (enum)
    pub state: State,
    pub counter: i64,
}
