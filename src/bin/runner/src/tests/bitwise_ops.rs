//! Runner tests for bitwise_ops machine.
//!
//! Tests:
//! - Goal reachability
//! - All bitwise operator results

use crate::stubs::{TickInfo, bitwise_ops_init, bitwise_ops_tick};

/// Test that bitwise_ops reaches done and all bitwise results are correct.
///
/// Initial: a=12 (0b1100), b=10 (0b1010)
/// Operations in compute state:
///   result_and = a & b = 12 & 10 = 0b1100 & 0b1010 = 0b1000 = 8
///   result_or  = a | b = 12 | 10  = 0b1100 | 0b1010  = 0b1110 = 14
///   result_xor = a ^ b = 12 ^ 10  = 0b1100 ^ 0b1010  = 0b0110 = 6
///   result_not_a = ~a = ~12 = -13 (64-bit two's complement)
#[test]
fn test_bitwise_ops_reaches_done() {
    let mut state = bitwise_ops_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!("should reach done within 20 ticks. state={}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = bitwise_ops_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(ticks, 2, "expected 2 ticks to reach done, got {}", ticks);
    println!("✓ Reached done in {} ticks", ticks);
}

/// Test all bitwise operator results.
#[test]
fn test_bitwise_ops_values() {
    let mut state = bitwise_ops_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = bitwise_ops_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    // &: 12 & 10 = 8 (0b1100 & 0b1010 = 0b1000)
    assert_eq!(state.result_and, 8,
               "result_and (a & b = 12 & 10) should be 8, got {}",
               state.result_and);

    // |: 12 | 10 = 14 (0b1100 | 0b1010 = 0b1110)
    assert_eq!(state.result_or, 14,
               "result_or (a | b = 12 | 10) should be 14, got {}",
               state.result_or);

    // ^: 12 ^ 10 = 6 (0b1100 ^ 0b1010 = 0b0110)
    assert_eq!(state.result_xor, 6,
               "result_xor (a ^ b = 12 ^ 10) should be 6, got {}",
               state.result_xor);

    // ~: ~12 = -13 (two's complement)
    assert_eq!(state.result_not_a, -13,
               "result_not_a (~a = ~12) should be -13, got {}",
               state.result_not_a);

    println!(
        "✓ Bitwise results: and={}, or={}, xor={}, not_a={}",
        state.result_and, state.result_or, state.result_xor, state.result_not_a
    );
}

/// Test initial state.
#[test]
fn test_bitwise_ops_initial_state() {
    let state = bitwise_ops_init();

    assert_eq!(state.state.as_str(), "start");
    assert_eq!(state.a, 12);
    assert_eq!(state.b, 10);
    println!("✓ Initial: a={}, b={} (expected: 12, 10)", state.a, state.b);
}
