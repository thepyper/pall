//! Runner tests for assignment_ops machine.
//!
//! Tests:
//! - Goal reachability
//! - All assignment operator results

use crate::stubs::{TickInfo, assignment_ops_init, assignment_ops_tick};

/// Test that assignment_ops reaches done and all assignment results are correct.
///
/// Initial: x=10, y=5, z=2
/// Operations on result variables (initial 0):
///   result_add += x  : 0 + 10 = 10
///   result_sub -= y  : 0 - 5  = -5
///   result_mul *= z  : 0 * 2  = 0
///   result_div /= x  : 0 / 10 = 0
///   result_mod %= y  : 0 % 5  = 0
#[test]
fn test_assignment_ops_reaches_done() {
    let mut state = assignment_ops_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!("should reach done within 20 ticks. state={}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = assignment_ops_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(ticks, 2, "expected 2 ticks to reach done, got {}", ticks);
    println!("✓ Reached done in {} ticks", ticks);
}

/// Test all assignment operator results.
#[test]
fn test_assignment_ops_values() {
    let mut state = assignment_ops_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = assignment_ops_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    // +=: 0 + 10 = 10
    assert_eq!(state.result_add, 10,
               "result_add (0 += x=10) should be 10, got {}",
               state.result_add);

    // -=: 0 - 5 = -5
    assert_eq!(state.result_sub, -5,
               "result_sub (0 -= y=5) should be -5, got {}",
               state.result_sub);

    // *=: 0 * 2 = 0
    assert_eq!(state.result_mul, 0,
               "result_mul (0 *= z=2) should be 0, got {}",
               state.result_mul);

    // /=: 0 / 10 = 0
    assert_eq!(state.result_div, 0,
               "result_div (0 /= x=10) should be 0, got {}",
               state.result_div);

    // %=: 0 % 5 = 0
    assert_eq!(state.result_mod, 0,
               "result_mod (0 %= y=5) should be 0, got {}",
               state.result_mod);

    println!(
        "✓ Assignment results: add={}, sub={}, mul={}, div={}, mod={}",
        state.result_add, state.result_sub, state.result_mul,
        state.result_div, state.result_mod
    );
}

/// Test initial state.
#[test]
fn test_assignment_ops_initial_state() {
    let state = assignment_ops_init();

    assert_eq!(state.state.as_str(), "start");
    assert_eq!(state.x, 10);
    assert_eq!(state.y, 5);
    assert_eq!(state.z, 2);
    println!("✓ Initial: x={}, y={}, z={}", state.x, state.y, state.z);
}
