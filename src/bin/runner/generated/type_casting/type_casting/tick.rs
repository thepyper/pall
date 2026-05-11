// Tick implementation for machine: type_casting
// Auto-generated. Do not edit.

use super::type_casting_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: type_casting
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::CastOps => {
            y.result_u8_u16 = (y.u8_val as u16) + y.u16_val;
            y.result_i8_u16 = (y.i8_val as i32) + (y.u16_val as i32);
            y.result_i32_i64 = (y.i32_val as i64) + y.i64_val;
            y.result_widening = (y.u8_val as u16);
            y.result_truty = y.flag  &&  (y.u8_val > y.threshold);
            y.target = y.u8_val;
            y.sum = 3.14;
            y.state = State::Done;

        }

        State::Done => {

        }

        State::Start => {
            y.state = State::CastOps;

        }


    }



    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: type_casting
pub fn init() -> Persistent {
    Persistent {
        state: State::Start,
        i32_val: 7i32,
        result_u8_u16: 0u16,
        threshold: 5u8,
        i8_val: 3i8,
        result_truty: false,
        i64_val: 100i64,
        target: 0u8,
        flag: true,
        u16_val: 20u16,
        result_i8_u16: 0i32,
        result_widening: 0u16,
        sum: 0.0 as f64,
        u32_val: 5u32,
        u8_val: 10u8,
        result_i32_i64: 0i64,
    }
}
