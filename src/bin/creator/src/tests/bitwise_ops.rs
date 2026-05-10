//! Creator tests for bitwise_ops machine.
//!
//! Tests:
//! - YAML vs programmatic equality
//! - Compilation succeeds
//! - Tests bitwise operators: &, |, ^, ~

use std::collections::HashMap;

use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

// ── YAML String ──────────────────────────────────────────────────────────────

const YAML_BITWISE_OPS: &str = r#"
id: bitwise_ops
initial: start
variables:
  a:
    type: I64
    initial: 12
  b:
    type: I64
    initial: 10
  result_and:
    type: I64
    initial: 0
  result_or:
    type: I64
    initial: 0
  result_xor:
    type: I64
    initial: 0
  result_not_a:
    type: I64
    initial: 0
states:
  start:
    transitions:
      - when: null
        do: []
        target: compute
  compute:
    actions:
      - when: null
        do:
          - result_and = a & b
          - result_or = a | b
          - result_xor = a ^ b
          - result_not_a = ~a
    transitions:
      - when: null
        do: []
        target: done
  done:
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_bitwise_ops() -> StateMachine {
    let mut states = HashMap::new();

    let start_state = State {
        actions: vec![],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "compute".to_string(),
        }],
    };
    states.insert("start".to_string(), start_state);

    let compute_state = State {
        actions: vec![Action {
            when: None,
            r#do: vec![
                FullStatement::parse("result_and = a & b").unwrap(),
                FullStatement::parse("result_or = a | b").unwrap(),
                FullStatement::parse("result_xor = a ^ b").unwrap(),
                FullStatement::parse("result_not_a = ~a").unwrap(),
            ],
        }],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "done".to_string(),
        }],
    };
    states.insert("compute".to_string(), compute_state);

    let done_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("done".to_string(), done_state);

    let mut variables = HashMap::new();
    variables.insert("a".to_string(), variable_i64(12));
    variables.insert("b".to_string(), variable_i64(10));
    variables.insert("result_and".to_string(), variable_i64(0));
    variables.insert("result_or".to_string(), variable_i64(0));
    variables.insert("result_xor".to_string(), variable_i64(0));
    variables.insert("result_not_a".to_string(), variable_i64(0));

    StateMachine {
        id: "bitwise_ops".to_string(),
        initial: Some("start".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

fn variable_i64(initial: i64) -> Variable {
    Variable {
        r#type: Type::I64,
        initial: Some(Value::Integer(IntegerValue {
            value: initial,
            fmt: IntegerFmt::Dec,
        })),
        output: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_bitwise_ops_yaml_vs_programmatic() {
    use super::comparison::compare_state_machines;

    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_BITWISE_OPS)
        .expect("YAML should parse");
    let prog_sm = build_bitwise_ops();

    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ bitwise_ops YAML and programmatic StateMachines are equal");
}

#[test]
fn test_bitwise_ops_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_BITWISE_OPS)
        .expect("YAML should parse");
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);

    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ bitwise_ops compilation succeeded");
}
