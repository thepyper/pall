//! Runner tests for type_casting machine.
//!
//! Tests:
//! - Goal reachability (reach done state)
//! - Common type resolution (U8 + U16 → U16, I8 + U16 → I32, I32 + I64 → I64)
//! - Assignment widening (U8 → U16 ✅)
//! - Truthiness (flag && (u8_val > threshold))
//! - Literal type inference (100 is I64, 3.14 is F64)

use crate::stubs::{TickInfo, TypeCastingPersistent, type_casting_init, type_casting_tick};
use super::helper::*;

/// Test that type_casting reaches the done state.
///
/// Expected behavior:
/// - Tick 1: start → cast_ops
/// - Tick 2: cast_ops actions execute, transition to done
#[test]
fn test_type_casting_reaches_done() {
    let mut state: TypeCastingPersistent = type_casting_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 10 {
            panic!("machine should reach done within 10 ticks. Final state: {}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = type_casting_tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }

    assert!(ticks >= 1, "should take at least 1 tick");
    println!("✓ type_casting reached done at tick {}", ticks);
}

/// Test values after reaching done state.
///
/// Expected values after cast_ops actions:
/// - result_u8_u16: u8_val(10) + u16_val(20) → U16 + U16 = 30
/// - result_i8_u16: i8_val(3) + u16_val(20) → I32 + I32 = 23 (I8 cast to I32, U16 cast to I32)
/// - result_i32_i64: i32_val(7) + i64_val(100) → I64 + I64 = 107
/// - result_widening: u8_val(10) widened to U16
/// - result_truty: flag(true) && (u8_val(10) > threshold(5)) = true && true = true
/// - target: u8_val = 10
/// - sum: 3.14 (F64)
#[test]
fn test_type_casting_values() {
    let mut state: TypeCastingPersistent = type_casting_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 10 {
            panic!("machine should reach done within 10 ticks. Final state: {}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = type_casting_tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(state.state.as_str(), "done");

    // result_u8_u16: U8 + U16 → U16, 10 + 20 = 30
    assert_eq!(state.result_u8_u16, 30, "result_u8_u16 should be u8_val + u16_val = 30");

    // result_i8_u16: I8 + U16 → I32, 3 + 20 = 23
    assert_eq!(state.result_i8_u16, 23, "result_i8_u16 should be i8_val + u16_val = 23");

    // result_i32_i64: I32 + I64 → I64, 7 + 100 = 107
    assert_eq!(state.result_i32_i64, 107, "result_i32_i64 should be i32_val + i64_val = 107");

    // result_widening: u8_val = 10, widened to U16
    assert_eq!(state.result_widening, 10, "result_widening should be u8_val = 10");

    // result_truty: flag && (u8_val > threshold) = true && (10 > 5) = true && true = true
    assert!(state.result_truty, "result_truty should be true (flag && (u8_val > threshold))");

    // target: u8_val = 10, same type, no cast needed
    assert_eq!(state.target, 10, "target should be u8_val = 10");

    // sum: 3.14 (F64 literal)
    assert!((state.sum - 3.14).abs() < 0.001, "sum should be approximately 3.14");
}

/// Test initial state values.
#[test]
fn test_type_casting_initial_state() {
    let state: TypeCastingPersistent = type_casting_init();

    assert_eq!(state.state.as_str(), "start");
    assert_eq!(state.u8_val, 10);
    assert_eq!(state.u16_val, 20);
    assert_eq!(state.u32_val, 5);
    assert_eq!(state.i8_val, 3);
    assert_eq!(state.i32_val, 7);
    assert_eq!(state.i64_val, 100);
    assert!(state.flag);
    assert_eq!(state.threshold, 5);
}
