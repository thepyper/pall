//! Fixture generator — generates counter_test machine files for the runner.
//!
//! Run with: cargo run --bin gen-fixture
//!
//! This generates files in src/bin/runner/generated/counter_test/
//! which the runner's stubs include via include! macros.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use pall::compiler::{Compiler, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

fn main() {
    // Build the machine programmatically
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

    let machine = StateMachine {
        id: "counter_test".to_string(),
        initial: Some("initial".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    };

    // Compile using the compiler
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let files = compiler.compile(&[machine]).expect("compile failed");

    let output_dir = PathBuf::from("src/bin/runner/generated/counter_test");
    fs::create_dir_all(&output_dir).ok();

    for (name, content) in &files {
        let path = output_dir.join(name);
        fs::create_dir_all(path.parent().unwrap()).ok();
        fs::write(&path, content).unwrap();
        println!("Written: {} ({} bytes)", path.display(), content.len());
    }

    println!("Done. {} files generated.", files.len());
}
