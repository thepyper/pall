// Tick implementation for machine: assignment_ops
// Auto-generated. Do not edit.

use super::assignment_ops_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: assignment_ops
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Compute => {
            y.result_add = y.result_add + y.x;
            y.result_sub = y.result_sub - y.y;
            y.result_mul = y.result_mul * y.z;
            y.result_div = y.result_div / y.x;
            y.result_mod = y.result_mod % y.y;
            y.state = State::Done;

        }

        State::Start => {
            y.state = State::Compute;

        }

        State::Done => {

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: assignment_ops
pub fn init() -> Persistent {
    Persistent {
        state: State::Start,
        result_mul: 0i64,
        result_div: 0i64,
        y: 5i64,
        x: 10i64,
        result_sub: 0i64,
        result_add: 0i64,
        z: 2i64,
        result_mod: 0i64,
    }
}
