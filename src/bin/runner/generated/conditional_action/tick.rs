// Tick implementation for machine: conditional_action
// Auto-generated. Do not edit.

use super::conditional_action_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: conditional_action
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Setup => {
            y.state = State::Work;

        }

        State::Work => {
            if y.counter < 5i64 {
            y.counter = y.counter + 1i64;
            }
            if y.counter >= 5i64 {
            y.state = State::Done;
            } else {
            y.state = State::Work;
            }

        }

        State::Done => {

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: conditional_action
pub fn init() -> Persistent {
    Persistent {
        state: State::Setup,
        counter: 0i64,
    }
}
