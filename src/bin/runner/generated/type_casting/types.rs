// Types for machine: type_casting
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};
use std::fmt;
use std::convert::TryFrom;

// ── State Enum ───────────────────────────────────────────────────────────────
/// Represents the possible states of machine: type_casting
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum State {
    Start,
    Done,
    CastOps,
}

impl State {
    /// Returns the lowercase string name of this state.
    pub const fn as_str(&self) -> &'static str {
        match self {
            State::Start => "start",
            State::Done => "done",
            State::CastOps => "cast_ops",
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
            "cast_ops" => Ok(State::CastOps),
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
/// Holds all persistent state for machine: type_casting
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Persistent {
    /// Current state (enum)
    pub state: State,
    pub i64_val: i64,
    pub u32_val: u32,
    pub result_i8_u16: i32,
    pub u16_val: u16,
    pub i8_val: i8,
    pub u8_val: u8,
    pub i32_val: i32,
    pub target: u8,
    pub flag: bool,
    pub result_u8_u16: u16,
    pub result_widening: u16,
    pub threshold: u8,
    pub result_truty: bool,
    pub sum: f64,
    pub result_i32_i64: i64,
}
