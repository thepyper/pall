//! Fixture generator — generates machine files for the runner.
//!
//! Run with: cargo run --bin gen-fixture
//!
//! This generates files in src/bin/runner/generated/ for ALL machines.
//! The runner's stubs include them via include! macros.
//!
//! To add a new machine:
//! 1. Add the machine definition to build_{machine_name}()
//! 2. Add build_{machine_name}() to the machines list below

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use pall::compiler::{Compiler, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

fn main() {
    let machines = vec![
        build_counter_test(),
        build_traffic_light(),
    ];

    let output_dir = PathBuf::from("src/bin/runner/generated");
    fs::create_dir_all(&output_dir).ok();

    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let files = compiler.compile(&machines).expect("compile failed");

    for (name, content) in &files {
        let path = output_dir.join(name);
        fs::create_dir_all(path.parent().unwrap()).ok();
        fs::write(&path, content).unwrap();
        println!("  Written: {} ({} bytes)", path.display(), content.len());
    }

    println!("Generated {} files for {} machine(s).", files.len(), machines.len());
}

// ── counter_test machine ─────────────────────────────────────────────────────

fn build_counter_test() -> StateMachine {
    let mut states = HashMap::new();

    let mut initial_state = State {
        actions: vec![],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "counting".to_string(),
        }],
    };
    states.insert("initial".to_string(), initial_state);

    let counting_state = State {
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

    let goal_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("goal".to_string(), goal_state);

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

// ── traffic_light machine ────────────────────────────────────────────────────

fn build_traffic_light() -> StateMachine {
    let mut states = HashMap::new();

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
