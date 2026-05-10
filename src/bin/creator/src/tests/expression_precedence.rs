//! Creator tests for expression_precedence machine.
//!
//! Tests:
//! - YAML vs programmatic equality
//! - Compilation succeeds
//! - Tests operator precedence: + vs *, () override

use std::collections::HashMap;

use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

// ── YAML String ──────────────────────────────────────────────────────────────

const YAML_EXPRESSION_PRECEDENCE: &str = r#"
id: expression_precedence
initial: start
variables:
  a:
    type: I64
    initial: 3
  b:
    type: I64
    initial: 4
  c:
    type: I64
    initial: 5
  result_precedence:
    type: I64
    initial: 0
  result_parens:
    type: I64
    initial: 0
  result_mixed:
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
          - result_precedence = a + b * c
          - result_parens = (a + b) * c
          - result_mixed = a * b + c * a
    transitions:
      - when: null
        do: []
        target: done
  done:
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_expression_precedence() -> StateMachine {
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
                // a + b * c = 3 + 4 * 5 = 3 + 20 = 23
                FullStatement::parse("result_precedence = a + b * c").unwrap(),
                // (a + b) * c = (3 + 4) * 5 = 7 * 5 = 35
                FullStatement::parse("result_parens = (a + b) * c").unwrap(),
                // a * b + c * a = 3 * 4 + 5 * 3 = 12 + 15 = 27
                FullStatement::parse("result_mixed = a * b + c * a").unwrap(),
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
    variables.insert("a".to_string(), variable_i64(3));
    variables.insert("b".to_string(), variable_i64(4));
    variables.insert("c".to_string(), variable_i64(5));
    variables.insert("result_precedence".to_string(), variable_i64(0));
    variables.insert("result_parens".to_string(), variable_i64(0));
    variables.insert("result_mixed".to_string(), variable_i64(0));

    StateMachine {
        id: "expression_precedence".to_string(),
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
fn test_expression_precedence_yaml_vs_programmatic() {
    use super::comparison::compare_state_machines;

    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_EXPRESSION_PRECEDENCE)
        .expect("YAML should parse");
    let prog_sm = build_expression_precedence();

    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ expression_precedence YAML and programmatic StateMachines are equal");
}

#[test]
fn test_expression_precedence_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_EXPRESSION_PRECEDENCE)
        .expect("YAML should parse");
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);

    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ expression_precedence compilation succeeded");
}
