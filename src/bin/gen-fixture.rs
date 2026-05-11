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
    Action, FloatFmt, FloatValue, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

fn main() {
    let machines = vec![
        build_counter_test(),
        build_traffic_light(),
        build_binary_counter(),
        build_conditional_action(),
        build_arithmetic_ops(),
        build_assignment_ops(),
        build_logic_ops(),
        build_bitwise_ops(),
        build_expression_precedence(),
        build_type_casting(),
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

// ── binary_counter machine ───────────────────────────────────────────────────

fn build_binary_counter() -> StateMachine {
    let mut states = HashMap::new();

    // Idle state: conditional transitions based on count
    let idle_state = State {
        actions: vec![],
        transitions: vec![
            Transition {
                when: Some(FullExpression::parse("count < 4").unwrap()),
                r#do: vec![],
                target: "counting".to_string(),
            },
            Transition {
                when: Some(FullExpression::parse("count >= 4").unwrap()),
                r#do: vec![],
                target: "done".to_string(),
            },
        ],
    };
    states.insert("idle".to_string(), idle_state);

    // Counting state: increment count, go back to idle
    let counting_state = State {
        actions: vec![Action {
            when: None,
            r#do: vec![FullStatement::parse("count += 1").unwrap()],
        }],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "idle".to_string(),
        }],
    };
    states.insert("counting".to_string(), counting_state);

    // Done state: dead end
    let done_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("done".to_string(), done_state);

    let mut variables = HashMap::new();
    variables.insert(
        "count".to_string(),
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
        id: "binary_counter".to_string(),
        initial: Some("idle".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

// ── conditional_action machine ───────────────────────────────────────────────

fn build_conditional_action() -> StateMachine {
    let mut states = HashMap::new();

    // Setup: always → work
    let setup_state = State {
        actions: vec![],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "work".to_string(),
        }],
    };
    states.insert("setup".to_string(), setup_state);

    // Work: conditional action (counter < 5), then transition
    let work_state = State {
        actions: vec![Action {
            when: Some(FullExpression::parse("counter < 5").unwrap()),
            r#do: vec![FullStatement::parse("counter += 1").unwrap()],
        }],
        transitions: vec![
            Transition {
                when: Some(FullExpression::parse("counter >= 5").unwrap()),
                r#do: vec![],
                target: "done".to_string(),
            },
            Transition {
                when: None,
                r#do: vec![],
                target: "work".to_string(),
            },
        ],
    };
    states.insert("work".to_string(), work_state);

    // Done: dead end
    let done_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("done".to_string(), done_state);

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
        id: "conditional_action".to_string(),
        initial: Some("setup".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

// ── arithmetic_ops machine ───────────────────────────────────────────────────

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
    variables.insert("base".to_string(), variable_i64(10));
    variables.insert("adder".to_string(), variable_i64(3));
    variables.insert("multiplier".to_string(), variable_i64(4));
    variables.insert("divisor".to_string(), variable_i64(3));
    variables.insert("result_add".to_string(), variable_i64(0));
    variables.insert("result_sub".to_string(), variable_i64(0));
    variables.insert("result_mul".to_string(), variable_i64(0));
    variables.insert("result_div".to_string(), variable_i64(0));
    variables.insert("result_mod".to_string(), variable_i64(0));

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

// ── assignment_ops machine ───────────────────────────────────────────────────

fn build_assignment_ops() -> StateMachine {
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
                FullStatement::parse("result_add += x").unwrap(),
                FullStatement::parse("result_sub -= y").unwrap(),
                FullStatement::parse("result_mul *= z").unwrap(),
                FullStatement::parse("result_div /= x").unwrap(),
                FullStatement::parse("result_mod %= y").unwrap(),
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
    variables.insert("x".to_string(), variable_i64(10));
    variables.insert("y".to_string(), variable_i64(5));
    variables.insert("z".to_string(), variable_i64(2));
    variables.insert("result_add".to_string(), variable_i64(0));
    variables.insert("result_sub".to_string(), variable_i64(0));
    variables.insert("result_mul".to_string(), variable_i64(0));
    variables.insert("result_div".to_string(), variable_i64(0));
    variables.insert("result_mod".to_string(), variable_i64(0));

    StateMachine {
        id: "assignment_ops".to_string(),
        initial: Some("start".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

// ── logic_ops machine ────────────────────────────────────────────────────────

fn variable_bool(initial: bool) -> Variable {
    Variable {
        r#type: Type::Bool,
        initial: Some(Value::Bool(initial)),
        output: false,
    }
}

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

// ── bitwise_ops machine ──────────────────────────────────────────────────────

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

// ── expression_precedence machine ────────────────────────────────────────────

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
                FullStatement::parse("result_precedence = a + b * c").unwrap(),
                FullStatement::parse("result_parens = (a + b) * c").unwrap(),
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

fn build_type_casting() -> StateMachine {
    let mut states = HashMap::new();

    let start_state = State {
        actions: vec![],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "cast_ops".to_string(),
        }],
    };
    states.insert("start".to_string(), start_state);

    let cast_ops_state = State {
        actions: vec![Action {
            when: None,
            r#do: vec![
                // U8 + U16 → U16 (implicit cast by compiler)
                FullStatement::parse("result_u8_u16 = u8_val + u16_val").unwrap(),
                // I8 + U16 → I32 (smallest common: I32 and I64, I32 is smaller)
                FullStatement::parse("result_i8_u16 = i8_val + u16_val").unwrap(),
                // I32 + I64 → I64 (signed widening)
                FullStatement::parse("result_i32_i64 = i32_val + i64_val").unwrap(),
                // U8 → U16 assignment (widening accepted)
                FullStatement::parse("result_widening = u8_val").unwrap(),
                // Truthiness: flag && (u8_val > threshold)
                FullStatement::parse("result_truty = flag && (u8_val > threshold)").unwrap(),
                // U8 → U8 (no cast, same type)
                FullStatement::parse("target = u8_val").unwrap(),
                // Float addition (F64 literal + F64 var)
                FullStatement::parse("sum = 3.14").unwrap(),
            ],
        }],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "done".to_string(),
        }],
    };
    states.insert("cast_ops".to_string(), cast_ops_state);

    let done_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("done".to_string(), done_state);

    let mut variables = HashMap::new();
    variables.insert("u8_val".to_string(), variable_u8(10));
    variables.insert("u16_val".to_string(), variable_u16(20));
    variables.insert("u32_val".to_string(), variable_u32(5));
    variables.insert("i8_val".to_string(), variable_i8(3));
    variables.insert("i32_val".to_string(), variable_i32(7));
    variables.insert("i64_val".to_string(), variable_i64(100));
    variables.insert("result_u8_u16".to_string(), variable_u16(0));
    variables.insert("result_i8_u16".to_string(), variable_i32(0));
    variables.insert("result_i32_i64".to_string(), variable_i64(0));
    variables.insert("result_widening".to_string(), variable_u16(0));
    variables.insert("result_truty".to_string(), variable_bool(false));
    variables.insert("flag".to_string(), variable_bool(true));
    variables.insert("threshold".to_string(), variable_u8(5));
    variables.insert("sum".to_string(), variable_f64(0.0));
    variables.insert("target".to_string(), variable_u8(0));

    StateMachine {
        id: "type_casting".to_string(),
        initial: Some("start".to_string()),
        states,
        inputs: HashMap::new(),
        signals: HashMap::new(),
        timers: HashMap::new(),
        variables,
        constants: HashMap::new(),
    }
}

fn variable_u8(val: u8) -> Variable {
    Variable {
        r#type: Type::U8,
        initial: Some(Value::Integer(IntegerValue { value: val as i64, fmt: IntegerFmt::Dec })),
        output: false,
    }
}

fn variable_u16(val: u16) -> Variable {
    Variable {
        r#type: Type::U16,
        initial: Some(Value::Integer(IntegerValue { value: val as i64, fmt: IntegerFmt::Dec })),
        output: false,
    }
}

fn variable_u32(val: u32) -> Variable {
    Variable {
        r#type: Type::U32,
        initial: Some(Value::Integer(IntegerValue { value: val as i64, fmt: IntegerFmt::Dec })),
        output: false,
    }
}

fn variable_i8(val: i8) -> Variable {
    Variable {
        r#type: Type::I8,
        initial: Some(Value::Integer(IntegerValue { value: val as i64, fmt: IntegerFmt::Dec })),
        output: false,
    }
}

fn variable_i32(val: i32) -> Variable {
    Variable {
        r#type: Type::I32,
        initial: Some(Value::Integer(IntegerValue { value: val as i64, fmt: IntegerFmt::Dec })),
        output: false,
    }
}

fn variable_f64(val: f64) -> Variable {
    Variable {
        r#type: Type::F64,
        initial: Some(Value::Float(FloatValue { value: val, fmt: FloatFmt::Decimal })),
        output: false,
    }
}
