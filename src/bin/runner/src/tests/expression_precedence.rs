//! Runner tests for expression_precedence machine.
//!
//! Tests:
//! - Goal reachability
//! - All precedence results

use crate::stubs::{TickInfo, expression_precedence_init, expression_precedence_tick};

/// Test that expression_precedence reaches done and all precedence results are correct.
///
/// Initial: a=3, b=4, c=5
/// Operations in compute state:
///   result_precedence = a + b * c = 3 + 4 * 5 = 3 + 20 = 23
///     (* has higher precedence than +, so b*c evaluated first)
///   result_parens = (a + b) * c = (3 + 4) * 5 = 7 * 5 = 35
///     (parentheses override precedence, so a+b evaluated first)
///   result_mixed = a * b + c * a = 3 * 4 + 5 * 3 = 12 + 15 = 27
///     (* before +, so both multiplications evaluated first, then addition)
#[test]
fn test_expression_precedence_reaches_done() {
    let mut state = expression_precedence_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        if ticks >= 20 {
            panic!("should reach done within 20 ticks. state={}", state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = expression_precedence_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    assert_eq!(ticks, 2, "expected 2 ticks to reach done, got {}", ticks);
    println!("✓ Reached done in {} ticks", ticks);
}

/// Test all precedence operator results.
#[test]
fn test_expression_precedence_values() {
    let mut state = expression_precedence_init();
    let mut ticks: u32 = 0;

    loop {
        if state.state.as_str() == "done" {
            break;
        }
        let tick_info = TickInfo { delta_ms: 1000 };
        state = expression_precedence_tick(&state, &tick_info)
            .expect("tick should succeed");
        ticks += 1;
    }

    // a + b * c = 3 + 4 * 5 = 3 + 20 = 23
    assert_eq!(state.result_precedence, 23,
               "result_precedence (a + b * c = 3 + 4*5) should be 23, got {}",
               state.result_precedence);

    // (a + b) * c = (3 + 4) * 5 = 7 * 5 = 35
    assert_eq!(state.result_parens, 35,
               "result_parens ((a + b) * c = (3+4)*5) should be 35, got {}",
               state.result_parens);

    // a * b + c * a = 3 * 4 + 5 * 3 = 12 + 15 = 27
    assert_eq!(state.result_mixed, 27,
               "result_mixed (a * b + c * a = 3*4+5*3) should be 27, got {}",
               state.result_mixed);

    println!(
        "✓ Precedence results: prec={}, parens={}, mixed={}",
        state.result_precedence, state.result_parens, state.result_mixed
    );
}

/// Test initial state.
#[test]
fn test_expression_precedence_initial_state() {
    let state = expression_precedence_init();

    assert_eq!(state.state.as_str(), "start");
    assert_eq!(state.a, 3);
    assert_eq!(state.b, 4);
    assert_eq!(state.c, 5);
    println!("✓ Initial: a={}, b={}, c={} (expected: 3, 4, 5)", state.a, state.b, state.c);
}
