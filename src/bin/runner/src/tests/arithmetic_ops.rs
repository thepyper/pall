//! Runner tests for arithmetic_ops machine.
//!
//! Tests:
//! - Goal reachability
//! - All arithmetic operator results

use crate::stubs::{TickInfo, arithmetic_ops_init, arithmetic_ops_tick};

/// Test that arithmetic_ops reaches done and all arithmetic results are correct.
///
/// Variables: base=10, adder=3, multiplier=4, divisor=3
/// Operations:
///   result_add = 10 + 3 = 13
///   result_sub = 10 - 3 = 7
///   result_mul = 10 * 4 = 40
///   result_div = 10 / 3 = 3 (integer division)
///   result_mod = 10 % 3 = 1
#[test]
fn test_arithmetic_ops_reaches_done() {
    let mut state = arithmetic_ops_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!("should reach done within 20 ticks. state={}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = arithmetic_ops_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(ticks, 2, "expected 2 ticks to reach done, got {}", ticks);
    println!("✓ Reached done in {} ticks", ticks);
}

/// Test all arithmetic operator results.
#[test]
fn test_arithmetic_ops_values() {
    let mut state = arithmetic_ops_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = arithmetic_ops_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    // +: 10 + 3 = 13
    assert_eq!(state.result_add, 13,
               "result_add (base + adder = 10 + 3) should be 13, got {}",
               state.result_add);

    // -: 10 - 3 = 7
    assert_eq!(state.result_sub, 7,
               "result_sub (base - adder = 10 - 3) should be 7, got {}",
               state.result_sub);

    // *: 10 * 4 = 40
    assert_eq!(state.result_mul, 40,
               "result_mul (base * multiplier = 10 * 4) should be 40, got {}",
               state.result_mul);

    // /: 10 / 3 = 3 (integer division)
    assert_eq!(state.result_div, 3,
               "result_div (base / divisor = 10 / 3) should be 3, got {}",
               state.result_div);

    // %: 10 % 3 = 1
    assert_eq!(state.result_mod, 1,
               "result_mod (base % divisor = 10 % 3) should be 1, got {}",
               state.result_mod);

    println!(
        "✓ Arithmetic results: add={}, sub={}, mul={}, div={}, mod={}",
        state.result_add, state.result_sub, state.result_mul,
        state.result_div, state.result_mod
    );
}

/// Test initial state.
#[test]
fn test_arithmetic_ops_initial_state() {
    let state = arithmetic_ops_init();

    assert_eq!(state.state.as_str(), "start");
    assert_eq!(state.base, 10);
    assert_eq!(state.adder, 3);
    assert_eq!(state.multiplier, 4);
    assert_eq!(state.divisor, 3);
    println!("✓ Initial: base={}, adder={}, multiplier={}, divisor={}",
             state.base, state.adder, state.multiplier, state.divisor);
}
