//! Creator tests for binary_counter machine.
//!
//! Tests:
//! - YAML vs programmatic equality
//! - Compilation succeeds

use std::collections::HashMap;

use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

// ── YAML String ──────────────────────────────────────────────────────────────

const YAML_BINARY_COUNTER: &str = r#"
id: binary_counter
initial: idle
variables:
  count:
    type: I64
    initial: 0
states:
  idle:
    transitions:
      - when: count < 4
        do: []
        target: counting
      - when: count >= 4
        do: []
        target: done
  counting:
    actions:
      - when: null
        do:
          - count += 1
    transitions:
      - when: null
        do: []
        target: idle
  done:
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_binary_counter() -> StateMachine {
    let mut states = HashMap::new();

    // Idle state: transition based on count
    let idle_state = State {
        actions: vec![],
        transitions: vec![
            Transition {
                when: Some(FullExpression::parse("count < 4").unwrap()),
                r#do: vec![],
                target: "counting".to_string(),
            },
            Transition {
                when: Some(FullExpression::parse("count >= 4").unwrap()),
                r#do: vec![],
                target: "done".to_string(),
            },
        ],
    };
    states.insert("idle".to_string(), idle_state);

    // Counting state: increment count, then go back to idle
    let counting_state = State {
        actions: vec![Action {
            when: None,
            r#do: vec![FullStatement::parse("count += 1").unwrap()],
        }],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "idle".to_string(),
        }],
    };
    states.insert("counting".to_string(), counting_state);

    // Done state: dead end
    let done_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("done".to_string(), done_state);

    let mut variables = HashMap::new();
    variables.insert(
        "count".to_string(),
        Variable {
            r#type: Type::I64,
            initial: Some(Value::Integer(IntegerValue {
                value: 0,
                fmt: IntegerFmt::Dec,
            })),
            output: false,
        },
    );

    StateMachine {
        id: "binary_counter".to_string(),
        initial: Some("idle".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_binary_counter_yaml_vs_programmatic() {
    use super::comparison::compare_state_machines;

    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_BINARY_COUNTER)
        .expect("YAML should parse");
    let prog_sm = build_binary_counter();

    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ binary_counter YAML and programmatic StateMachines are equal");
}

#[test]
fn test_binary_counter_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_BINARY_COUNTER)
        .expect("YAML should parse");
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);

    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ binary_counter compilation succeeded");
}
