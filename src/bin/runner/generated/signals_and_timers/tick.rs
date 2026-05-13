// Tick implementation for machine: signals_and_timers
// Auto-generated. Do not edit.

use super::signals_and_timers_types::{Persistent, State};
use super::super::TickInfo;
use super::super::error::TickError;

// ── Tick Function ────────────────────────────────────────────────────────────
/// Execute one tick of machine: signals_and_timers
pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError> {
    let mut y = x.clone();

    match x.state {
        State::Start => {
            y.state = State::Compute;

        }

        State::Compute => {
            y.counter = y.counter + 1i64;
            y.result_signal = y.counter * 2i64 + 5i64;
            y.doubled = y.counter * 100i64;
            y.flag = y.counter >= 3i64;
            if y.counter >= 5i64 {
            y.state = State::Done;
            }

        }

        State::Done => {

        }


    }

    y.signal_flag = y.counter >= 3i64;
    y.signal_double_counter = y.counter * 2i64;
    y.signal_counter_plus_one = y.counter + 1i64;

    if y.counter < 10i64 {
        y.timer_cond = x.timer_cond + tick_info.delta_ms as i64;
    } else {
        y.timer_cond = 0 as i64;
    }
    y.timer_always = x.timer_always + tick_info.delta_ms as i64;

    Ok(y)
}

// ── Init Function ────────────────────────────────────────────────────────────
/// Create initial x state for machine: signals_and_timers
pub fn init() -> Persistent {
    Persistent {
        state: State::Start,
        input_val: <_>::default(),
        flag: false,
        ratio: 0.0 as f64,
        doubled: 0i64,
        counter: 0i64,
        result_signal: 0i64,
        signal_flag: <_>::default(),
        signal_double_counter: <_>::default(),
        signal_counter_plus_one: <_>::default(),
        timer_cond: 0 as i64,
        timer_always: 0 as i64,
    }
}
