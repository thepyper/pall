//! Creator tests for traffic_light machine.
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

const YAML_TRAFFIC_LIGHT: &str = r#"
id: traffic_light
initial: red
variables:
  tick_count:
    type: I64
    initial: 0
states:
  red:
    actions:
      - when: null
        do:
          - tick_count += 1
    transitions:
      - when: null
        do: []
        target: yellow
  yellow:
    actions:
      - when: null
        do:
          - tick_count += 1
    transitions:
      - when: null
        do: []
        target: green
  green:
    actions:
      - when: null
        do:
          - tick_count += 1
    transitions:
      - when: null
        do: []
        target: red
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_traffic_light() -> StateMachine {
    let mut states = HashMap::new();

    // Red state: tick_count++, then yellow
    let red_state = State {
        actions: vec![Action {
            when: None,
            r#do: vec![FullStatement::parse("tick_count += 1").unwrap()],
        }],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "yellow".to_string(),
        }],
    };
    states.insert("red".to_string(), red_state);

    // Yellow state: tick_count++, then green
    let yellow_state = State {
        actions: vec![Action {
            when: None,
            r#do: vec![FullStatement::parse("tick_count += 1").unwrap()],
        }],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "green".to_string(),
        }],
    };
    states.insert("yellow".to_string(), yellow_state);

    // Green state: tick_count++, then back to red
    let green_state = State {
        actions: vec![Action {
            when: None,
            r#do: vec![FullStatement::parse("tick_count += 1").unwrap()],
        }],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "red".to_string(),
        }],
    };
    states.insert("green".to_string(), green_state);

    let mut variables = HashMap::new();
    variables.insert(
        "tick_count".to_string(),
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
        id: "traffic_light".to_string(),
        initial: Some("red".to_string()),
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
fn test_traffic_light_yaml_vs_programmatic() {
    // Import the shared comparison function
    use super::comparison::compare_state_machines;

    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_TRAFFIC_LIGHT)
        .expect("YAML should parse");
    let prog_sm = build_traffic_light();

    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ traffic_light YAML and programmatic StateMachines are equal");
}

#[test]
fn test_traffic_light_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_TRAFFIC_LIGHT)
        .expect("YAML should parse");
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);

    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ traffic_light compilation succeeded");
}
