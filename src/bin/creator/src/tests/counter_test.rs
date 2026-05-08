//! Creator tests for counter_test machine.
//!
//! Tests YAML parsing vs programmatic construction equality,
//! and compilation codegen.

use std::collections::HashMap;
use std::path::PathBuf;

use pall::compiler::{Compiler, CompileError, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

// ── YAML String ──────────────────────────────────────────────────────────────

const YAML_COUNTER_TEST: &str = r#"
id: counter_test
initial: initial
variables:
  counter:
    type: I64
    initial: 0
states:
  initial:
    transitions:
      - when: null
        do: []
        target: counting
  counting:
    actions:
      - when: null
        do:
          - counter += 1
    transitions:
      - when: counter >= 10
        do: []
        target: goal
  goal:
    actions: []
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_counter_programmatic() -> StateMachine {
    let mut states = HashMap::new();

    // "initial" state: always transition to "counting"
    let mut initial_state = State {
        actions: vec![],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "counting".to_string(),
        }],
    };
    states.insert("initial".to_string(), initial_state);

    // "counting" state: counter += 1, transition when counter >= 10
    let mut counting_state = State {
        actions: vec![Action {
            when: None,
            r#do: vec![FullStatement::parse("counter += 1").unwrap()],
        }],
        transitions: vec![Transition {
            when: Some(FullExpression::parse("counter >= 10").unwrap()),
            r#do: vec![],
            target: "goal".to_string(),
        }],
    };
    states.insert("counting".to_string(), counting_state);

    // "goal" state: no transitions (dead end)
    let goal_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("goal".to_string(), goal_state);

    // Variable: counter (U32, initial 0)
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
        id: "counter_test".to_string(),
        initial: Some("initial".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

// ── Semantic Equality Helper ─────────────────────────────────────────────────

/// Compare two StateMachines for semantic (content) equality.
/// HashMaps are compared by key-value content, ignoring order.
fn compare_state_machines(a: &StateMachine, b: &StateMachine) -> Result<(), String> {
    if a.id != b.id {
        return Err(format!("id mismatch: '{}' != '{}'", a.id, b.id));
    }
    if a.initial != b.initial {
        return Err(format!(
            "initial mismatch: '{:?}' != '{:?}'",
            a.initial, b.initial
        ));
    }

    // Compare states (order independent)
    compare_state_map(&a.states, &b.states, "states")?;

    // Compare variables (order independent)
    compare_var_map(&a.variables, &b.variables, "variables")?;

    // Compare inputs (order independent)
    compare_input_map(&a.inputs, &b.inputs, "inputs")?;

    // Compare signals (order independent)
    compare_signal_map(&a.signals, &b.signals, "signals")?;

    // Compare timers (order independent)
    compare_timer_map(&a.timers, &b.timers, "timers")?;

    // Compare constants (order independent)
    compare_const_map(&a.constants, &b.constants, "constants")?;

    Ok(())
}

fn compare_state_map(
    a: &HashMap<String, State>,
    b: &HashMap<String, State>,
    field: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!(
            "{} size mismatch: {} != {}",
            field,
            a.len(),
            b.len()
        ));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                compare_states(val_a, val_b, field)?;
            }
            None => return Err(format!("{} missing key: '{}'", field, key)),
        }
    }
    Ok(())
}

fn compare_states(a: &State, b: &State, parent: &str) -> Result<(), String> {
    let pfx = format!("{}.states[{}]", parent, "???");
    // Compare actions (order matters)
    if a.actions.len() != b.actions.len() {
        return Err(format!(
            "{} actions count mismatch: {} != {}",
            parent,
            a.actions.len(),
            b.actions.len()
        ));
    }
    for (i, (a_act, b_act)) in a.actions.iter().zip(b.actions.iter()).enumerate() {
        compare_actions(a_act, b_act, &format!("{}.actions[{}]", pfx, i))?;
    }
    // Compare transitions (order matters)
    if a.transitions.len() != b.transitions.len() {
        return Err(format!(
            "{} transitions count mismatch: {} != {}",
            parent,
            a.transitions.len(),
            b.transitions.len()
        ));
    }
    for (i, (a_trans, b_trans)) in a
        .transitions
        .iter()
        .zip(b.transitions.iter())
        .enumerate()
    {
        compare_transitions(a_trans, b_trans, &format!("{}.transitions[{}]", pfx, i))?;
    }
    Ok(())
}

fn compare_actions(a: &Action, b: &Action, parent: &str) -> Result<(), String> {
    // Compare when clauses
    match (&a.when, &b.when) {
        (Some(ae), Some(be)) => compare_expressions(ae, be, parent)?,
        (None, None) => {}
        (Some(_), None) => return Err(format!("{} when: Some != None", parent)),
        (None, Some(_)) => return Err(format!("{} when: None != Some", parent)),
    }
    // Compare do statements (order matters)
    if a.r#do.len() != b.r#do.len() {
        return Err(format!(
            "{} do count mismatch: {} != {}",
            parent,
            a.r#do.len(),
            b.r#do.len()
        ));
    }
    for (i, (a_stmt, b_stmt)) in a.r#do.iter().zip(b.r#do.iter()).enumerate() {
        compare_statements(a_stmt, b_stmt, &format!("{}.do[{}]", parent, i))?;
    }
    Ok(())
}

fn compare_transitions(a: &Transition, b: &Transition, parent: &str) -> Result<(), String> {
    // Compare when clauses
    match (&a.when, &b.when) {
        (Some(ae), Some(be)) => compare_expressions(ae, be, parent)?,
        (None, None) => {}
        (Some(_), None) => return Err(format!("{} when: Some != None", parent)),
        (None, Some(_)) => return Err(format!("{} when: None != Some", parent)),
    }
    // Compare do statements (order matters)
    if a.r#do.len() != b.r#do.len() {
        return Err(format!(
            "{} do count mismatch: {} != {}",
            parent,
            a.r#do.len(),
            b.r#do.len()
        ));
    }
    for (i, (a_stmt, b_stmt)) in a.r#do.iter().zip(b.r#do.iter()).enumerate() {
        compare_statements(a_stmt, b_stmt, &format!("{}.do[{}]", parent, i))?;
    }
    // Compare target
    if a.target != b.target {
        return Err(format!("{} target mismatch: '{}' != '{}'", parent, a.target, b.target));
    }
    Ok(())
}

fn compare_expressions(a: &FullExpression, b: &FullExpression, parent: &str) -> Result<(), String> {
    if a.raw != b.raw {
        return Err(format!("{} expression raw mismatch: '{}' != '{}'", parent, a.raw, b.raw));
    }
    // Expression derives PartialEq, so direct comparison works
    if a.expression != b.expression {
        return Err(format!(
            "{} expression content mismatch",
            parent
        ));
    }
    Ok(())
}

fn compare_statements(a: &FullStatement, b: &FullStatement, parent: &str) -> Result<(), String> {
    if a.raw != b.raw {
        return Err(format!("{} statement raw mismatch: '{}' != '{}'", parent, a.raw, b.raw));
    }
    // Statement derives PartialEq, so direct comparison works
    if a.statement != b.statement {
        return Err(format!(
            "{} statement content mismatch",
            parent
        ));
    }
    Ok(())
}

fn compare_var_map(
    a: &HashMap<String, Variable>,
    b: &HashMap<String, Variable>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!(
            "{} size mismatch: {} != {}",
            parent,
            a.len(),
            b.len()
        ));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

fn compare_input_map(
    a: &HashMap<String, pall::machine::Input>,
    b: &HashMap<String, pall::machine::Input>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!(
            "{} size mismatch: {} != {}",
            parent,
            a.len(),
            b.len()
        ));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

fn compare_signal_map(
    a: &HashMap<String, pall::machine::Signal>,
    b: &HashMap<String, pall::machine::Signal>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!(
            "{} size mismatch: {} != {}",
            parent,
            a.len(),
            b.len()
        ));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

fn compare_timer_map(
    a: &HashMap<String, pall::machine::Timer>,
    b: &HashMap<String, pall::machine::Timer>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!(
            "{} size mismatch: {} != {}",
            parent,
            a.len(),
            b.len()
        ));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

fn compare_const_map(
    a: &HashMap<String, pall::machine::Constant>,
    b: &HashMap<String, pall::machine::Constant>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!(
            "{} size mismatch: {} != {}",
            parent,
            a.len(),
            b.len()
        ));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

// ── Test: YAML vs Programmatic Equality ─────────────────────────────────────

#[test]
fn test_counter_test_yaml_vs_programmatic() {
    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_COUNTER_TEST)
        .expect("YAML should parse");
    let prog_sm = build_counter_programmatic();

    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ YAML and programmatic StateMachines are equal");
}

// ── Test: Compilation ────────────────────────────────────────────────────────

/// Compile the machine and write generated files to the runner's generated directory.
/// This test verifies that code generation succeeds and files are written.
fn compile_and_write(
    sm: &StateMachine,
) -> Result<FileSet, Vec<CompileError>> {
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let files = compiler.compile(&[sm.clone()])?;

    // Determine output directory
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/bin/runner/generated/counter_test");

    std::fs::create_dir_all(&output_dir).expect("failed to create output directory");

    for (name, content) in &files {
        let file_path = output_dir.join(name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).unwrap_or_else(|e| {
            panic!("failed to write {}: {}", file_path.display(), e);
        });
        eprintln!("  Written: {} ({} bytes)", file_path.display(), content.len());
    }

    println!("✓ Compiled {} file(s) to {}", files.len(), output_dir.display());
    Ok(files)
}

#[test]
fn test_counter_test_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_COUNTER_TEST)
        .expect("YAML should parse");
    let files = compile_and_write(&sm)
        .expect("compilation should succeed");
    assert!(!files.is_empty(), "should generate at least one file");
}
