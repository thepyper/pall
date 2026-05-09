// Tick implementation for machine: arithmetic_ops
// Auto-generated. Do not edit.

use super::arithmetic_ops_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: arithmetic_ops
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Start => {
            y.state = State::Compute;

        }

        State::Compute => {
            y.result_add = y.base + y.adder;
            y.result_sub = y.base - y.adder;
            y.result_mul = y.base * y.multiplier;
            y.result_div = y.base / y.divisor;
            y.result_mod = y.base % y.divisor;
            y.state = State::Done;

        }

        State::Done => {

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: arithmetic_ops
pub fn init() -> Persistent {
    Persistent {
        state: State::Start,
        multiplier: 4i64,
        base: 10i64,
        result_add: 0i64,
        divisor: 3i64,
        result_div: 0i64,
        result_mod: 0i64,
        adder: 3i64,
        result_sub: 0i64,
        result_mul: 0i64,
    }
}
