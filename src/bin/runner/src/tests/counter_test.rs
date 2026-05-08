//! Runner tests for counter_test machine.
//!
//! Tests:
//! - Goal reachability (machine reaches goal within expected ticks)
//! - Counter value when goal is reached (should be 10)
//! - Counter value at fixed tick counts (deterministic)

use crate::stubs::*;
use super::helper::*;

/// Test that counter_test reaches the goal state within expected ticks.
///
/// Expected behavior:
/// - Tick 0: initial state, counter = 0
/// - Tick 1: initial → counting (always-true transition), counter still 0
/// - Tick 2: counting, counter += 1 → counter = 1
/// - Tick 3: counter = 2
/// - ...
/// - Tick 11: counter = 9
/// - Tick 12: counter = 10, transition fires (counter >= 10), → goal
#[test]
fn test_counter_test_reaches_goal() {
    let result = run_until(20, 1000, |state| {
        state.state.as_str() == "goal"
    })
        .expect("machine should reach goal within 20 ticks");

    assert!(
        result.ticks_taken >= 1,
        "should have taken at least one tick, got {}",
        result.ticks_taken
    );
    assert!(
        result.ticks_taken <= 15,
        "expected ~12 ticks, got {}",
        result.ticks_taken
    );
    println!(
        "✓ Reached goal at tick {} (final state: {})",
        result.ticks_taken, result.final_state
    );
}

/// Test counter value when goal is reached.
///
/// When the transition `counter >= 10` fires, the counter should be exactly 10.
/// This is because:
/// - The action `counter += 1` executes first in the counting state
/// - Then the transition checks `counter >= 10`
/// - When counter = 10, the condition is true and the transition fires
#[test]
fn test_counter_test_goal_counter_value() {
    let max_ticks = 20;
    let mut ticks: u32 = 0;
    let mut state: Persistent = init();

    loop {
        if state.state.as_str() == "goal" {
            assert_eq!(
                state.counter, 10,
                "counter should be 10 when goal is reached, got {}",
                state.counter
            );
            println!(
                "✓ Goal reached at tick {}, counter = {} (expected 10)",
                ticks, state.counter
            );
            break;
        }

        if ticks >= max_ticks {
            panic!(
                "Goal not reached after {} ticks. Final state: {}",
                max_ticks,
                state.state.as_str()
            );
        }

        let tick_info = TickInfo { delta_ms: 1000 };
        state = tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }
}

/// Test counter value after a fixed number of ticks.
///
/// Tick-by-tick counter behavior:
/// - Tick 0 (before first tick): state=initial, counter=0
/// - After tick 1: state=counting, counter=0 (initial→counting, no action in initial)
/// - After tick 2: state=counting, counter=1 (action: counter += 1)
/// - After tick 3: counter=2
/// - After tick 4: counter=3
/// - After tick 5: counter=4
#[test]
fn test_counter_test_counter_at_tick() {
    let state = run_for(5, 1000);

    assert_eq!(
        state.state.as_str(),
        "counting",
        "after 5 ticks, state should be 'counting', got '{}'",
        state.state.as_str()
    );
    assert_eq!(
        state.counter, 4,
        "after 5 ticks, counter should be 4, got {}",
        state.counter
    );

    println!("✓ After 5 ticks: state={}, counter={} (expected: counting, 4)",
             state.state.as_str(), state.counter);
}

/// Test initial state.
#[test]
fn test_counter_test_initial_state() {
    let state: Persistent = init();

    assert_eq!(
        state.state.as_str(),
        "initial",
        "initial state should be 'initial', got '{}'",
        state.state.as_str()
    );
    assert_eq!(
        state.counter, 0,
        "initial counter should be 0, got {}",
        state.counter
    );

    println!(
        "✓ Initial state: state='{}', counter={}",
        state.state.as_str(), state.counter
    );
}
