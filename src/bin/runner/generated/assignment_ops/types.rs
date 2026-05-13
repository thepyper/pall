// Types for machine: assignment_ops
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};
use std::fmt;
use std::convert::TryFrom;

// ── State Enum ───────────────────────────────────────────────────────────────
/// Represents the possible states of machine: assignment_ops
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum State {
    Done,
    Start,
    Compute,
}

impl State {
    /// Returns the lowercase string name of this state.
    pub const fn as_str(&self) -> &'static str {
        match self {
            State::Done => "done",
            State::Start => "start",
            State::Compute => "compute",
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
            "done" => Ok(State::Done),
            "start" => Ok(State::Start),
            "compute" => Ok(State::Compute),
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
/// Holds all persistent state for machine: assignment_ops
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Persistent {
    /// Current state (enum)
    pub state: State,
    pub result_sub: i64,
    pub result_add: i64,
    pub result_div: i64,
    pub result_mod: i64,
    pub z: i64,
    pub y: i64,
    pub x: i64,
    pub result_mul: i64,
}
