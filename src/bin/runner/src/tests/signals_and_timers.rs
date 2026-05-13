//! Runner tests for signals_and_timers machine.
//!
//! Tests:
//! - Goal reachability (reach done state)
//! - Signal values (computed expressions)
//! - Timer accumulation (always + conditional)
//! - Constants and inputs used correctly

use crate::stubs::SignalsAndTimersPersistent;
use crate::stubs::{SignalsAndTimersState, signals_and_timers_init, signals_and_timers_tick};
use crate::stubs::TickInfo;

/// Test that signals_and_timers reaches the done state.
///
/// Expected behavior:
/// - Tick 1: start → compute (counter=1, signal_double=2, signal_plus_one=2)
/// - Tick 2: compute (counter=2, signal_double=4, signal_plus_one=3)
/// - Tick 3: compute (counter=3, signal_flag=true)
/// - Tick 4: compute (counter=4, signal_double=8, signal_plus_one=5)
/// - Tick 5: compute (counter=5, counter >= 5 → transition to done)
///
/// After 5 ticks, counter=5, done state.
#[test]
fn test_signals_and_timers_reaches_done() {
    let mut state: SignalsAndTimersPersistent = signals_and_timers_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!(
                "machine should reach done within 20 ticks. Final state: {}",
                state.state.as_str()
            );
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = signals_and_timers_tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }

    assert!(ticks >= 5, "should take at least 5 ticks");
    println!(
        "✓ signals_and_timers reached done at tick {} (counter={})",
        ticks, state.counter
    );
}

/// Test signal values and timer accumulation after reaching done.
///
/// Expected values after 5 ticks:
/// - counter = 5 (incremented each tick)
/// - signal_double_counter = counter * 2 = 10
/// - signal_counter_plus_one = counter + 1 = 6
/// - signal_flag = counter >= 3 = true
/// - timer_always = accumulated for 5 ticks = 5000ms = 5 (if delta_ms=1000)
/// - timer_cond = accumulated when counter < 10 = 5 ticks (counter never >= 10) = 5000ms = 5
#[test]
fn test_signals_and_timers_values() {
    let mut state: SignalsAndTimersPersistent = signals_and_timers_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!(
                "machine should reach done within 20 ticks. Final state: {}",
                state.state.as_str()
            );
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = signals_and_timers_tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(state.state.as_str(), "done");

    // Signal values (computed from expressions)
    assert_eq!(
        state.signal_double_counter,
        10,
        "signal_double_counter should be counter * 2 = 5 * 2 = 10"
    );
    assert_eq!(
        state.signal_counter_plus_one,
        6,
        "signal_counter_plus_one should be counter + 1 = 5 + 1 = 6"
    );
    assert!(
        state.signal_flag,
        "signal_flag should be true (counter 5 >= 3)"
    );

    // Timer values (accumulated in ms)
    // 6 ticks total: Start→Compute (1) + Compute×5 (5) = 6
    // timer_always: accumulated every tick, 6 ticks * 1000ms = 6000
    assert_eq!(
        state.timer_always,
        6000,
        "timer_always should be 6 * 1000 = 6000"
    );
    // timer_cond: accumulated when counter < 10 (always true for counter<=5)
    assert_eq!(
        state.timer_cond,
        6000,
        "timer_cond should be 6 * 1000 = 6000"
    );

    // Variable values
    assert_eq!(state.counter, 5, "counter should be 5");
    assert_eq!(state.result_signal, 15, "result_signal should be 5 * 2 + 5 = 15");
    assert_eq!(state.doubled, 500, "doubled should be 5 * 100 = 500");
    assert!(state.flag, "flag should be true (counter >= 3)");
    assert_eq!(state.input_val, 0, "input_val should not be modified (read-only)");
}

/// Test initial state values.
#[test]
fn test_signals_and_timers_initial_state() {
    let state: SignalsAndTimersPersistent = signals_and_timers_init();

    assert_eq!(state.state.as_str(), "start");
    assert_eq!(state.counter, 0);
    assert_eq!(state.result_signal, 0);
    assert_eq!(state.doubled, 0);
    assert!(!state.flag);
    assert_eq!(state.timer_always, 0);
    assert_eq!(state.timer_cond, 0);
    assert_eq!(state.input_val, 0);
}
