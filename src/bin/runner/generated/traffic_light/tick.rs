// Tick implementation for machine: traffic_light
// Auto-generated. Do not edit.

use super::traffic_light_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: traffic_light
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Green => {
            y.tick_count = y.tick_count + 1i64;
            y.state = State::Red;

        }

        State::Red => {
            y.tick_count = y.tick_count + 1i64;
            y.state = State::Yellow;

        }

        State::Yellow => {
            y.tick_count = y.tick_count + 1i64;
            y.state = State::Green;

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: traffic_light
pub fn init() -> Persistent {
    Persistent {
        state: State::Red,
        tick_count: 0i64,
    }
}
