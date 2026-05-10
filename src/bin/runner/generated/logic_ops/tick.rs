// Tick implementation for machine: logic_ops
// Auto-generated. Do not edit.

use super::logic_ops_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: logic_ops
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Start => {
            y.state = State::Compute;

        }

        State::Done => {

        }

        State::Compute => {
            if y.a  &&  y.b {
            y.flag1 = true;
            }
            if y.a  ||  y.b {
            y.flag2 = true;
            }
            if y.a  ^  y.b {
            y.result_and = y.a  &&  y.b;
            y.result_or = y.a  ||  y.b;
            y.result_xor = y.a  ^  y.b;
            y.result_not_a = !y.a;
            }
            y.state = State::Done;

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: logic_ops
pub fn init() -> Persistent {
    Persistent {
        state: State::Start,
        flag2: false,
        result_xor: false,
        result_not_a: false,
        a: true,
        b: false,
        flag1: false,
        result_and: false,
        result_or: false,
    }
}
