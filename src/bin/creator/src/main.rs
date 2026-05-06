//! Creator — generates Rust source code for a state machine and writes it to the runner's generated/ directory.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, AssignmentOperator, BinaryOperator, Constant, Expression, FullExpression,
    FullStatement, State, StateMachine, Transition, Type, Value, Variable,
    IntegerFmt, IntegerValue,
};

/// Build the example counter machine:
///
/// ```text
/// initial ──────► counting ────► goal
///                 │              ▲
///                 │ counter+=1   │ counter >= 10
///                 └──────────────┘
/// ```
fn build_counter_machine() -> StateMachine {
    let mut states = HashMap::new();

    // "initial" state: always transition to "counting"
    let mut initial_state = State {
        actions: vec![],
        transitions: vec![],
    };
    initial_state.transitions.push(Transition {
        when: None, // always-true
        r#do: vec![],
        target: "counting".to_string(),
    });
    states.insert("initial".to_string(), initial_state);

    // "counting" state: increment counter every tick, transition to "goal" when counter >= 10
    let mut counting_state = State {
        actions: vec![],
        transitions: vec![],
    };
    // Action: counter += 1 every tick
    counting_state.actions.push(Action {
        when: None,
        r#do: vec![FullStatement::parse("counter += 1").unwrap()],
    });
    // Transition: when counter >= 10, go to goal
    counting_state.transitions.push(Transition {
        when: Some(FullExpression::parse("counter >= 10").unwrap()),
        r#do: vec![],
        target: "goal".to_string(),
    });
    states.insert("counting".to_string(), counting_state);

    // "goal" state: no outgoing transitions (dead end)
    let mut goal_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("goal".to_string(), goal_state);

    // Variable: counter (I64, initial 0)
    // Using I64 to match codegen's value_to_rust which outputs i64 literals.
    let mut variables: HashMap<String, Variable> = HashMap::new();
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

/// Compile the machine and return generated files.
fn compile_machine(machine: &StateMachine) -> Result<FileSet, Vec<pall::compiler::CompileError>> {
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    compiler.compile(&[machine.clone()])
}

/// Write generated files to the runner's generated/ directory.
fn write_generated_files(files: &FileSet) -> PathBuf {
    // CARGO_MANIFEST_DIR is the parent crate's directory (pall/)
    // since the binary is defined there.
    // Runner generated dir: src/bin/runner/generated/ relative to project root.
    let project_root = env!("CARGO_MANIFEST_DIR");
    let output_dir = PathBuf::from(project_root).join("src/bin/runner/generated/");

    fs::create_dir_all(&output_dir).expect("failed to create generated directory");

    for (name, content) in files {
        let file_path = output_dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                panic!("failed to create directory {}: {}", parent.display(), e);
            });
        }
        fs::write(&file_path, content).unwrap_or_else(|e| {
            panic!("failed to write {}: {}", file_path.display(), e);
        });
        println!("  Written: {}  ({} bytes)", file_path.display(), content.len());
    }

    output_dir
}

fn main() {
    println!("=== Pall Creator ===");
    println!("Building counter_test machine...");

    let machine = build_counter_machine();
    println!("Machine id: {}", machine.id);
    println!("States: {}", machine.states.len());
    println!("Variables: {}", machine.variables.len());

    println!("\nCompiling...");
    let files = match compile_machine(&machine) {
        Ok(files) => files,
        Err(errors) => {
            eprintln!("Compilation errors:");
            for err in &errors {
                eprintln!("  - [{}] {}", err.kind, err.message);
            }
            std::process::exit(1);
        }
    };

    println!("Generated {} file(s):\n", files.len());
    let output_dir = write_generated_files(&files);

    println!("\nAll files written to: {}", output_dir.display());
    println!("=== Creator done ===");
}
