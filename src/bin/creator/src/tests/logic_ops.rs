//! Creator tests for logic_ops machine.
//!
//! Tests:
//! - YAML vs programmatic equality
//! - Compilation succeeds
//! - Tests logical operators: &&, ||, ^^, !

use std::collections::HashMap;

use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

// ── YAML String ──────────────────────────────────────────────────────────────

const YAML_LOGIC_OPS: &str = r#"
id: logic_ops
initial: start
variables:
  a:
    type: Bool
    initial: true
  b:
    type: Bool
    initial: false
  flag1:
    type: Bool
    initial: false
  flag2:
    type: Bool
    initial: false
  result_and:
    type: Bool
    initial: false
  result_or:
    type: Bool
    initial: false
  result_xor:
    type: Bool
    initial: false
  result_not_a:
    type: Bool
    initial: false
states:
  start:
    transitions:
      - when: null
        do: []
        target: compute
  compute:
    actions:
      - when: a && b
        do:
          - flag1 = true
      - when: a || b
        do:
          - flag2 = true
      - when: a ^^ b
        do:
          - result_and = a && b
          - result_or = a || b
          - result_xor = a ^^ b
          - result_not_a = !a
    transitions:
      - when: null
        do: []
        target: done
  done:
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_logic_ops() -> StateMachine {
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
        actions: vec![
            Action {
                when: Some(FullExpression::parse("a && b").unwrap()),
                r#do: vec![FullStatement::parse("flag1 = true").unwrap()],
            },
            Action {
                when: Some(FullExpression::parse("a || b").unwrap()),
                r#do: vec![FullStatement::parse("flag2 = true").unwrap()],
            },
            Action {
                when: Some(FullExpression::parse("a ^^ b").unwrap()),
                r#do: vec![
                    FullStatement::parse("result_and = a && b").unwrap(),
                    FullStatement::parse("result_or = a || b").unwrap(),
                    FullStatement::parse("result_xor = a ^^ b").unwrap(),
                    FullStatement::parse("result_not_a = !a").unwrap(),
                ],
            },
        ],
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
    variables.insert("a".to_string(), variable_bool(true));
    variables.insert("b".to_string(), variable_bool(false));
    variables.insert("flag1".to_string(), variable_bool(false));
    variables.insert("flag2".to_string(), variable_bool(false));
    variables.insert("result_and".to_string(), variable_bool(false));
    variables.insert("result_or".to_string(), variable_bool(false));
    variables.insert("result_xor".to_string(), variable_bool(false));
    variables.insert("result_not_a".to_string(), variable_bool(false));

    StateMachine {
        id: "logic_ops".to_string(),
        initial: Some("start".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

fn variable_bool(initial: bool) -> Variable {
    Variable {
        r#type: Type::Bool,
        initial: Some(Value::Bool(initial)),
        output: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_logic_ops_yaml_vs_programmatic() {
    use super::comparison::compare_state_machines;

    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_LOGIC_OPS)
        .expect("YAML should parse");
    let prog_sm = build_logic_ops();

    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ logic_ops YAML and programmatic StateMachines are equal");
}

#[test]
fn test_logic_ops_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_LOGIC_OPS)
        .expect("YAML should parse");
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);

    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ logic_ops compilation succeeded");
}
