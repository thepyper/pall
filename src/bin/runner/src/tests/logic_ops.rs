//! Runner tests for logic_ops machine.
//!
//! Tests:
//! - Goal reachability
//! - All logical operator results

use crate::stubs::{TickInfo, logic_ops_init, logic_ops_tick};

/// Test that logic_ops reaches done and all logical results are correct.
///
/// Initial: a=true, b=false
/// Actions in compute state (executed in order):
///   Action 1: when a && b → flag1 = true
///     a && b = true && false = false → action doesn't fire
///     flag1 stays false
///   Action 2: when a || b → flag2 = true
///     a || b = true || false = true → flag2 = true
///   Action 3: when a ^^ b → set results
///     a ^^ b = true ^^ false = true → action fires
///     result_and = a && b = false
///     result_or = a || b = true
///     result_xor = a ^^ b = true
///     result_not_a = !a = false
#[test]
fn test_logic_ops_reaches_done() {
    let mut state = logic_ops_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!("should reach done within 20 ticks. state={}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = logic_ops_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(ticks, 2, "expected 2 ticks to reach done, got {}", ticks);
    println!("✓ Reached done in {} ticks", ticks);
}

/// Test all logical operator results.
#[test]
fn test_logic_ops_values() {
    let mut state = logic_ops_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = logic_ops_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    // a && b = true && false = false
    assert_eq!(state.result_and, false,
               "result_and (a && b = true && false) should be false, got {}",
               state.result_and);

    // a || b = true || false = true
    assert_eq!(state.result_or, true,
               "result_or (a || b = true || false) should be true, got {}",
               state.result_or);

    // a ^^ b = true ^^ false = true
    assert_eq!(state.result_xor, true,
               "result_xor (a ^^ b = true ^^ false) should be true, got {}",
               state.result_xor);

    // !a = !true = false
    assert_eq!(state.result_not_a, false,
               "result_not_a (!a = !true) should be false, got {}",
               state.result_not_a);

    // Flag checks
    assert_eq!(state.flag1, false,
               "flag1 (a && b = false) should be false, got {}",
               state.flag1);
    assert_eq!(state.flag2, true,
               "flag2 (a || b = true) should be true, got {}",
               state.flag2);

    println!(
        "✓ Logic results: and={}, or={}, xor={}, not_a={}, flag1={}, flag2={}",
        state.result_and, state.result_or, state.result_xor,
        state.result_not_a, state.flag1, state.flag2
    );
}

/// Test initial state.
#[test]
fn test_logic_ops_initial_state() {
    let state = logic_ops_init();

    assert_eq!(state.state.as_str(), "start");
    assert_eq!(state.a, true);
    assert_eq!(state.b, false);
    println!("✓ Initial: a={}, b={} (expected: true, false)", state.a, state.b);
}
