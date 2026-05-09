//! Runner tests for binary_counter machine.
//!
//! Tests:
//! - Goal reachability (reaches "done" state)
//! - Final count value
//! - Tick-by-tick behavior

use crate::stubs::{TickInfo, TickError, binary_counter_init, binary_counter_tick};

/// Test that binary_counter reaches the "done" state.
///
/// Expected tick-by-tick:
/// Tick 0: idle, count=0 (initial)
/// Tick 1: idle→counting, count=0
/// Tick 2: counting→idle, count=1 (count += 1 in counting)
/// Tick 3: idle→counting, count=1
/// Tick 4: counting→idle, count=2
/// Tick 5: idle→counting, count=2
/// Tick 6: counting→idle, count=3
/// Tick 7: idle→counting, count=3
/// Tick 8: counting→idle, count=4
/// Tick 9: idle→done (count >= 4), count=4
#[test]
fn test_binary_counter_reaches_done() {
    let mut state: crate::stubs::BinaryCounterPersistent = binary_counter_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!("machine should reach done within 20 ticks. Final state: {}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        match binary_counter_tick(&state, &tick_info) {
            Ok(s) => state = s,
            Err(e) => panic!("tick error: {}", e.message),
        };
        ticks += 1;
    }

    assert_eq!(ticks, 9, "expected 9 ticks to reach done, got {}", ticks);
    println!(
        "✓ binary_counter reached done at tick {} (count={})",
        ticks, state.count
    );
}

/// Test final count value when done.
#[test]
fn test_binary_counter_final_count() {
    let mut state: crate::stubs::BinaryCounterPersistent = binary_counter_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        match binary_counter_tick(&state, &tick_info) {
            Ok(s) => state = s,
            Err(e) => panic!("tick error: {}", e.message),
        };
        ticks += 1;
    }

    assert_eq!(state.count, 4, "count should be 4 when done, got {}", state.count);
    println!("✓ Final count at done: {} (expected 4)", state.count);
}

/// Test initial state.
#[test]
fn test_binary_counter_initial_state() {
    let state: crate::stubs::BinaryCounterPersistent = binary_counter_init();

    assert_eq!(
        state.state.as_str(),
        "idle",
        "initial state should be 'idle', got '{}'",
        state.state.as_str()
    );
    assert_eq!(
        state.count, 0,
        "initial count should be 0, got {}",
        state.count
    );

    println!("✓ Initial: state='{}', count={} (expected: idle, 0)", state.state.as_str(), state.count);
}
