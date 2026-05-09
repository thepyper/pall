//! Creator tests for conditional_action machine.
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

const YAML_CONDITIONAL_ACTION: &str = r#"
id: conditional_action
initial: setup
variables:
  counter:
    type: I64
    initial: 0
states:
  setup:
    transitions:
      - when: null
        do: []
        target: work
  work:
    actions:
      - when: counter < 5
        do:
          - counter += 1
    transitions:
      - when: counter >= 5
        do: []
        target: done
      - when: null
        do: []
        target: work
  done:
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_conditional_action() -> StateMachine {
    let mut states = HashMap::new();

    // Setup: always → work
    let setup_state = State {
        actions: vec![],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "work".to_string(),
        }],
    };
    states.insert("setup".to_string(), setup_state);

    // Work: conditional action (counter < 5), then transition
    let work_state = State {
        actions: vec![Action {
            when: Some(FullExpression::parse("counter < 5").unwrap()),
            r#do: vec![FullStatement::parse("counter += 1").unwrap()],
        }],
        transitions: vec![
            Transition {
                when: Some(FullExpression::parse("counter >= 5").unwrap()),
                r#do: vec![],
                target: "done".to_string(),
            },
            Transition {
                when: None,
                r#do: vec![],
                target: "work".to_string(),
            },
        ],
    };
    states.insert("work".to_string(), work_state);

    // Done: dead end
    let done_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("done".to_string(), done_state);

    let mut variables = HashMap::new();
    variables.insert(
        "counter".to_string(),
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
        id: "conditional_action".to_string(),
        initial: Some("setup".to_string()),
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
fn test_conditional_action_yaml_vs_programmatic() {
    use super::comparison::compare_state_machines;

    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_CONDITIONAL_ACTION)
        .expect("YAML should parse");
    let prog_sm = build_conditional_action();

    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ conditional_action YAML and programmatic StateMachines are equal");
}

#[test]
fn test_conditional_action_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_CONDITIONAL_ACTION)
        .expect("YAML should parse");
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);

    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ conditional_action compilation succeeded");
}
