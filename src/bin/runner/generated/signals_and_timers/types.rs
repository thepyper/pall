// Types for machine: signals_and_timers
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};
use std::fmt;
use std::convert::TryFrom;

// ── State Enum ───────────────────────────────────────────────────────────────
/// Represents the possible states of machine: signals_and_timers
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum State {
    Start,
    Done,
    Compute,
}

impl State {
    /// Returns the lowercase string name of this state.
    pub const fn as_str(&self) -> &'static str {
        match self {
            State::Start => "start",
            State::Done => "done",
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
            "start" => Ok(State::Start),
            "done" => Ok(State::Done),
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

// ── Constants ────────────────────────────────────────────────────────────────
pub const small_const: u8 = 10u8;
pub const large_const: i64 = 1000i64;

// ── Persistent ───────────────────────────────────────────────────────────────
/// Holds all persistent state for machine: signals_and_timers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Persistent {
    /// Current state (enum)
    pub state: State,
    pub input_val: i64,
    pub ratio: f64,
    pub flag: bool,
    pub counter: i64,
    pub result_signal: i64,
    pub doubled: i64,
    pub signal_counter_plus_one: i64,
    pub signal_flag: bool,
    pub signal_double_counter: i64,
    pub timer_cond: i64,
    pub timer_always: i64,
}
