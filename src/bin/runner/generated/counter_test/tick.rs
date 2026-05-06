// Tick implementation for machine: counter_test
// Auto-generated. Do not edit.

use super::counter_test_types::{Persistent, Update, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: counter_test
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Update, TickError> {
    let mut y = Update::default();

    match x.state {
        State::Initial => {
            y.state = Some(State::Counting);
            return Ok(y);

        }

        State::Goal => {
        }

        State::Counting => {
                y.counter = Some(x.counter + 1i64);

            if x.counter >= 10i64 {
            y.state = Some(State::Goal);
            return Ok(y);
            }

        }

    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: counter_test
pub fn init() -> Persistent {
    Persistent {
        state: State::Initial,
        state_name: "initial".to_string(),
        counter: 0i64,
    }
}
