//! Runner tests for conditional_action machine.
//!
//! Tests:
//! - Goal reachability (reaches "done" state)
//! - Final counter value (5 when done)
//! - Initial state

use crate::stubs::{TickInfo, conditional_action_init, conditional_action_tick};

/// Test that conditional_action reaches "done" via conditional action.
///
/// Tick-by-tick:
/// Tick 0: setup, counter=0 (initial)
/// Tick 1: setup→work, counter=0
/// Tick 2: work action (0<5, counter=1), work→work
/// Tick 3: work action (1<5, counter=2), work→work
/// Tick 4: work action (2<5, counter=3), work→work
/// Tick 5: work action (3<5, counter=4), work→work
/// Tick 6: work action (4<5, counter=5), transition (5>=5)→done
///
/// Total: 6 ticks, final counter=5
#[test]
fn test_conditional_action_reaches_done() {
    let mut state = conditional_action_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!("should reach done within 20 ticks. state={}, counter={}",
                   state.state.as_str(), state.counter);
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = conditional_action_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(ticks, 6, "expected 6 ticks to reach done, got {}", ticks);
    assert_eq!(state.counter, 5, "counter should be 5 at done, got {}", state.counter);
    println!(
        "✓ conditional_action reached done at tick {} (counter={})",
        ticks, state.counter
    );
}

/// Test initial state.
#[test]
fn test_conditional_action_initial_state() {
    let state = conditional_action_init();

    assert_eq!(state.state.as_str(), "setup",
               "initial state should be 'setup', got '{}'", state.state.as_str());
    assert_eq!(state.counter, 0,
               "initial counter should be 0, got {}", state.counter);
    println!("✓ Initial: state='{}', counter={} (expected: setup, 0)",
             state.state.as_str(), state.counter);
}
