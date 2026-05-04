mod machine;
mod compiler;

use machine::*;
use std::collections::HashMap;

fn build_sample_machine() -> StateMachine {
    let mut states = HashMap::new();

    // Initial state
    let mut initial_state = State {
        actions: vec![],
        transitions: vec![],
    };
    initial_state.transitions.push(Transition {
        when: Some(FullExpression::parse("counter > 10").unwrap()),
        r#do: vec![FullStatement::parse("counter += 1").unwrap()],
        target: "running".to_string(),
    });
    states.insert("initial".to_string(), initial_state);

    // Running state
    let mut running_state = State {
        actions: vec![],
        transitions: vec![],
    };
    running_state.transitions.push(Transition {
        when: Some(FullExpression::parse("error_flag").unwrap()),
        r#do: vec![],
        target: "error".to_string(),
    });
    running_state.transitions.push(Transition {
        when: None, // always-true fallback
        r#do: vec![],
        target: "initial".to_string(),
    });
    states.insert("running".to_string(), running_state);

    // Error state
    let mut error_state = State {
        actions: vec![],
        transitions: vec![],
    };
    error_state.transitions.push(Transition {
        when: Some(FullExpression::parse("reset_button").unwrap()),
        r#do: vec![],
        target: "initial".to_string(),
    });
    states.insert("error".to_string(), error_state);

    StateMachine {
        id: "counter_machine".to_string(),
        initial: Some("initial".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables: [
            (
                "counter".to_string(),
                Variable {
                    r#type: Type::U32,
                    initial: Some(Value::Integer(IntegerValue {
                        value: 0,
                        fmt: IntegerFmt::Dec,
                    })),
                    output: false,
                },
            ),
        ]
        .into_iter()
        .collect(),
        constants: [
            (
                "THRESHOLD".to_string(),
                Constant {
                    r#type: Type::U32,
                    value: Value::Integer(IntegerValue {
                        value: 10,
                        fmt: IntegerFmt::Dec,
                    }),
                    output: false,
                },
            ),
        ]
        .into_iter()
        .collect(),
    }
}

fn main() {
    let machine = build_sample_machine();
    let machines = vec![machine];

    // Compile
    let rust_backend = compiler::RustBackend::new();
    let compiler = compiler::Compiler::new(rust_backend);

    match compiler.compile(&machines) {
        Ok(files) => {
            println!("Generated {} file(s):\n", files.len());
            let mut keys: Vec<_> = files.keys().collect();
            keys.sort();
            for key in keys {
                println!("=== {} ===", key);
                println!("{}", files[key]);
                println!();
            }
        }
        Err(errors) => {
            eprintln!("Compilation errors:");
            for err in &errors {
                eprintln!("  - [{}] {}", err.kind, err.message);
            }
        }
    }
}
