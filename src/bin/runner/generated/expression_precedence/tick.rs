// Tick implementation for machine: expression_precedence
// Auto-generated. Do not edit.

use super::expression_precedence_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: expression_precedence
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Compute => {
            y.result_precedence = y.a + y.b * y.c;
            y.result_parens = (y.a + y.b) * y.c;
            y.result_mixed = y.a * y.b + y.c * y.a;
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
/// Create initial x state for machine: expression_precedence
pub fn init() -> Persistent {
    Persistent {
        state: State::Start,
        b: 4i64,
        c: 5i64,
        result_parens: 0i64,
        a: 3i64,
        result_precedence: 0i64,
        result_mixed: 0i64,
    }
}
