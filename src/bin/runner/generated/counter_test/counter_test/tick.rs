// Tick implementation for machine: counter_test
// Auto-generated. Do not edit.

use super::counter_test_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: counter_test
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Counting => {
            y.counter = y.counter + 1i64;
            if y.counter >= 10i64 {
            y.state = State::Goal;
            }

        }

        State::Goal => {

        }

        State::Initial => {
            y.state = State::Counting;

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: counter_test
pub fn init() -> Persistent {
    Persistent {
        state: State::Initial,
        counter: 0i64,
    }
}
