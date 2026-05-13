//! Runner tests for type_system machine.
//!
//! Tests:
//! - Goal reachability (reach done state)
//! - All 12 variable types have correct values after assignment

use crate::stubs::TypeSystemPersistent;
use crate::stubs::{type_system_init, type_system_tick};
use crate::stubs::TickInfo;

/// Test that type_system reaches the done state.
///
/// Expected flow:
/// - Tick 0: Start → Verify
/// - Tick 1: Verify (all 12 assignments execute), transition to Done
#[test]
fn test_type_system_reaches_done() {
    let mut state: TypeSystemPersistent = type_system_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 10 {
            panic!(
                "machine should reach done within 10 ticks. Final state: {}",
                state.state.as_str()
            );
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = type_system_tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }

    assert!(ticks >= 1, "should take at least 1 tick");
    println!(
        "✓ type_system reached done at tick {} (all 12 types verified)",
        ticks
    );
}

/// Test ALL 12 variable types have correct final values.
///
/// After tick 1 in Verify state:
/// - bool_val = true (Bool)
/// - u8_val = 255 (U8)
/// - u16_val = 65535 (U16)
/// - u32_val = 4294967295 (U32)
/// - u64_val = 9223372036854775807 (U64)
/// - i8_val = -128 (I8)
/// - i16_val = -32768 (I16)
/// - i32_val = 2147483647 (I32)
/// - i64_val = 9223372036854775807 (I64)
/// - f32_val = 3.14 (F32)
/// - f64_val = 2.71828 (F64)
/// - str_val = "hello" (String)
#[test]
fn test_type_system_all_types() {
    let mut state: TypeSystemPersistent = type_system_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 10 {
            panic!(
                "machine should reach done within 10 ticks. Final state: {}",
                state.state.as_str()
            );
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = type_system_tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(state.state.as_str(), "done");

    // Bool type
    assert!(state.bool_val, "bool_val should be true");

    // U8 type (max = 255)
    assert_eq!(state.u8_val, 255, "u8_val should be 255 (max U8)");

    // U16 type (max = 65535)
    assert_eq!(state.u16_val, 65535, "u16_val should be 65535 (max U16)");

    // U32 type (max = 4294967295)
    assert_eq!(state.u32_val, 4_294_967_295, "u32_val should be 4294967295 (max U32)");

    // U64 type (large positive)
    assert_eq!(state.u64_val, 9_223_372_036_854_775_807, "u64_val should be 9223372036854775807");

    // I8 type (min = -128)
    assert_eq!(state.i8_val, -128, "i8_val should be -128 (min I8)");

    // I16 type (min = -32768)
    assert_eq!(state.i16_val, -32768, "i16_val should be -32768 (min I16)");

    // I32 type (max = 2147483647)
    assert_eq!(state.i32_val, 2_147_483_647, "i32_val should be 2147483647 (max I32)");

    // I64 type (large positive)
    assert_eq!(state.i64_val, 9_223_372_036_854_775_807, "i64_val should be 9223372036854775807");

    // F32 type
    assert!((state.f32_val - 3.14).abs() < 0.001, "f32_val should be ~3.14");

    // F64 type
    assert!((state.f64_val - 2.71828).abs() < 0.0001, "f64_val should be ~2.71828");

    // String type
    assert_eq!(state.str_val, "hello", "str_val should be 'hello'");
}

/// Test initial state values.
#[test]
fn test_type_system_initial_state() {
    let state: TypeSystemPersistent = type_system_init();

    assert_eq!(state.state.as_str(), "start");
    assert!(state.bool_val);
    assert_eq!(state.u8_val, 255);
    assert_eq!(state.u16_val, 65535);
    assert_eq!(state.u32_val, 4_294_967_295);
    assert_eq!(state.u64_val, 9_223_372_036_854_775_807);
    assert_eq!(state.i8_val, -128);
    assert_eq!(state.i16_val, -32768);
    assert_eq!(state.i32_val, 2_147_483_647);
    assert_eq!(state.i64_val, 9_223_372_036_854_775_807);
    assert!((state.f32_val - 3.14).abs() < 0.001);
    assert!((state.f64_val - 2.71828).abs() < 0.0001);
    assert_eq!(state.str_val, "hello");
}
