// Types for machine: traffic_light
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};
use std::fmt;
use std::convert::TryFrom;

// ── State Enum ───────────────────────────────────────────────────────────────
/// Represents the possible states of machine: traffic_light
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum State {
    Red,
    Yellow,
    Green,
}

impl State {
    /// Returns the lowercase string name of this state.
    pub const fn as_str(&self) -> &'static str {
        match self {
            State::Red => "red",
            State::Yellow => "yellow",
            State::Green => "green",
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
            "red" => Ok(State::Red),
            "yellow" => Ok(State::Yellow),
            "green" => Ok(State::Green),
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
/// Holds all persistent state for machine: traffic_light
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Persistent {
    /// Current state (enum)
    pub state: State,
    pub tick_count: i64,
}
