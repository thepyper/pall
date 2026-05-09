//! Runner tests for traffic_light machine.
//!
//! Tests:
//! - Goal reachability (cycle back to red)
//! - Tick count verification (3 ticks for one full cycle)
//! - tick_count variable value

use crate::stubs::{TickInfo, TrafficLightPersistent, traffic_light_init, traffic_light_tick_fn};
use super::helper::*;

/// Test that traffic_light cycles through all states and returns to red.
///
/// Expected behavior:
/// - Tick 0: state=red, tick_count=0
/// - Tick 1: red action (tick_count=1), transition to yellow
/// - Tick 2: yellow action (tick_count=2), transition to green
/// - Tick 3: green action (tick_count=3), transition to red
/// - Tick 3: state=red (first time back at red after full cycle)
#[test]
fn test_traffic_light_reaches_red() {
    let mut state: TrafficLightPersistent = traffic_light_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "red" && state.tick_count > 0 {
            break;
        }
        if ticks >= 30 {
            panic!("machine should cycle back to red within 30 ticks. Final state: {}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = traffic_light_tick_fn(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }

    // After one full cycle: red→yellow→green→red = 3 ticks
    assert_eq!(
        ticks, 3,
        "expected 3 ticks for one cycle, got {}",
        ticks
    );
    println!(
        "✓ traffic_light cycled back to red at tick {} (tick_count={})",
        ticks, state.tick_count
    );
}

/// Test tick_count value after one full cycle.
/// After 3 ticks: red(1) → yellow(2) → green(3) → back to red
#[test]
fn test_traffic_light_tick_count() {
    let mut state: TrafficLightPersistent = traffic_light_init();
    for _ in 0..3 {
        let tick_info = TickInfo { delta_ms: 1000 };
        state = traffic_light_tick_fn(&state, &tick_info).expect("tick should succeed");
    }

    assert_eq!(
        state.state.as_str(),
        "red",
        "after 3 ticks, state should be 'red', got '{}'",
        state.state.as_str()
    );
    assert_eq!(
        state.tick_count, 3,
        "after 3 ticks, tick_count should be 3, got {}",
        state.tick_count
    );

    println!(
        "✓ After 3 ticks: state='{}', tick_count={} (expected: red, 3)",
        state.state.as_str(), state.tick_count
    );
}

/// Test initial state.
#[test]
fn test_traffic_light_initial_state() {
    let state: TrafficLightPersistent = traffic_light_init();

    assert_eq!(
        state.state.as_str(),
        "red",
        "initial state should be 'red', got '{}'",
        state.state.as_str()
    );
    assert_eq!(
        state.tick_count, 0,
        "initial tick_count should be 0, got {}",
        state.tick_count
    );

    println!(
        "✓ Initial state: state='{}', tick_count={}",
        state.state.as_str(), state.tick_count
    );
}

/// Test two full cycles (6 ticks).
#[test]
fn test_traffic_light_two_cycles() {
    let mut state: TrafficLightPersistent = traffic_light_init();
    for _ in 0..6 {
        let tick_info = TickInfo { delta_ms: 1000 };
        state = traffic_light_tick_fn(&state, &tick_info).expect("tick should succeed");
    }

    assert_eq!(
        state.state.as_str(),
        "red",
        "after 2 cycles, state should be 'red', got '{}'",
        state.state.as_str()
    );
    assert_eq!(
        state.tick_count, 6,
        "after 2 cycles, tick_count should be 6, got {}",
        state.tick_count
    );

    println!(
        "✓ After 6 ticks (2 cycles): state='{}', tick_count={} (expected: red, 6)",
        state.state.as_str(), state.tick_count
    );
}
