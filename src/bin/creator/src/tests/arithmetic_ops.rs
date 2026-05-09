//! Creator tests for arithmetic_ops machine.
//!
//! Tests:
//! - YAML vs programmatic equality
//! - Compilation succeeds
//! - Tests all arithmetic operators: +, -, *, /, %

use std::collections::HashMap;

use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

// ── YAML String ──────────────────────────────────────────────────────────────

const YAML_ARITHMETIC_OPS: &str = r#"
id: arithmetic_ops
initial: start
variables:
  base:
    type: I64
    initial: 10
  adder:
    type: I64
    initial: 3
  multiplier:
    type: I64
    initial: 4
  divisor:
    type: I64
    initial: 3
  result_add:
    type: I64
    initial: 0
  result_sub:
    type: I64
    initial: 0
  result_mul:
    type: I64
    initial: 0
  result_div:
    type: I64
    initial: 0
  result_mod:
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
          - result_add = base + adder
          - result_sub = base - adder
          - result_mul = base * multiplier
          - result_div = base / divisor
          - result_mod = base % divisor
    transitions:
      - when: null
        do: []
        target: done
  done:
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_arithmetic_ops() -> StateMachine {
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
                FullStatement::parse("result_add = base + adder").unwrap(),
                FullStatement::parse("result_sub = base - adder").unwrap(),
                FullStatement::parse("result_mul = base * multiplier").unwrap(),
                FullStatement::parse("result_div = base / divisor").unwrap(),
                FullStatement::parse("result_mod = base % divisor").unwrap(),
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
    variables.insert("base".to_string(), variable_i64(10, false));
    variables.insert("adder".to_string(), variable_i64(3, false));
    variables.insert("multiplier".to_string(), variable_i64(4, false));
    variables.insert("divisor".to_string(), variable_i64(3, false));
    variables.insert("result_add".to_string(), variable_i64(0, false));
    variables.insert("result_sub".to_string(), variable_i64(0, false));
    variables.insert("result_mul".to_string(), variable_i64(0, false));
    variables.insert("result_div".to_string(), variable_i64(0, false));
    variables.insert("result_mod".to_string(), variable_i64(0, false));

    StateMachine {
        id: "arithmetic_ops".to_string(),
        initial: Some("start".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

fn variable_i64(initial: i64, _output: bool) -> Variable {
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
fn test_arithmetic_ops_yaml_vs_programmatic() {
    use super::comparison::compare_state_machines;

    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_ARITHMETIC_OPS)
        .expect("YAML should parse");
    let prog_sm = build_arithmetic_ops();

    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ arithmetic_ops YAML and programmatic StateMachines are equal");
}

#[test]
fn test_arithmetic_ops_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_ARITHMETIC_OPS)
        .expect("YAML should parse");
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);

    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ arithmetic_ops compilation succeeded");
}
