// Tick implementation for machine: binary_counter
// Auto-generated. Do not edit.

use super::binary_counter_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: binary_counter
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Done => {

        }

        State::Counting => {
            y.count = y.count + 1i64;
            y.state = State::Idle;

        }

        State::Idle => {
            if y.count < 4i64 {
            y.state = State::Counting;
            } else if y.count >= 4i64 {
            y.state = State::Done;
            }

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: binary_counter
pub fn init() -> Persistent {
    Persistent {
        state: State::Idle,
        count: 0i64,
    }
}
