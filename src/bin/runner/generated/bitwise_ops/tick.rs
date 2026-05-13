// Tick implementation for machine: bitwise_ops
// Auto-generated. Do not edit.

use super::bitwise_ops_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: bitwise_ops
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Start => {
            y.state = State::Compute;

        }

        State::Compute => {
            y.result_and = y.a  &  y.b;
            y.result_or = y.a  |  y.b;
            y.result_xor = y.a  ^  y.b;
            y.result_not_a = !y.a;
            y.state = State::Done;

        }

        State::Done => {

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: bitwise_ops
pub fn init() -> Persistent {
    Persistent {
        state: State::Start,
        result_xor: 0i64,
        result_not_a: 0i64,
        a: 12i64,
        result_or: 0i64,
        b: 10i64,
        result_and: 0i64,
    }
}
