//! Creator tests for type_casting machine.
//!
//! Tests:
//! - YAML vs programmatic equality
//! - Compilation succeeds
//! - Tests implicit type casting: common type resolution, unsigned priority,
//!   truthiness, assignment widening, operator-type compatibility

use std::collections::HashMap;

use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

use super::comparison::compare_state_machines;

// ── YAML String ──────────────────────────────────────────────────────────────

const YAML_TYPE_CASTING: &str = r#"
id: type_casting
initial: start
variables:
  u8_val:
    type: U8
    initial: 10
  u16_val:
    type: U16
    initial: 20
  u32_val:
    type: U32
    initial: 5
  i8_val:
    type: I8
    initial: 3
  i32_val:
    type: I32
    initial: 7
  i64_val:
    type: I64
    initial: 100
  result_u8_u16:
    type: U16
    initial: 0
  result_i8_u16:
    type: I32
    initial: 0
  result_i32_i64:
    type: I64
    initial: 0
  result_widening:
    type: U16
    initial: 0
  result_truty:
    type: Bool
    initial: false
  flag:
    type: Bool
    initial: true
  threshold:
    type: U8
    initial: 5
  sum:
    type: F64
    initial: 0.0
  target:
    type: U8
    initial: 0
states:
  start:
    transitions:
      - when: null
        do: []
        target: cast_ops
  cast_ops:
    actions:
      - when: null
        do:
          # U8 + U16 → U16 (widening u8_val to u16)
          - result_u8_u16 = u8_val + u16_val
          # I8 + U16 → I32 (smallest common: I32 and I64, I32 is smaller)
          - result_i8_u16 = i8_val + u16_val
          # I32 + I64 → I64 (signed widening)
          - result_i32_i64 = i32_val + i64_val
          # U8 → U16 assignment (widening accepted)
          - result_widening = u8_val
          # Truthiness: flag && (u8_val > threshold)
          - result_truty = flag && (u8_val > threshold)
          # U8 → U8 (no cast, same type)
          - target = u8_val
          # Float addition (F64 literal + F64 var)
          - sum = 3.14
    transitions:
      - when: null
        do: []
        target: done
  done:
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

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
    variables.insert("u8_val".to_string(), Variable {
        r#type: Type::U8,
        initial: Some(Value::Integer(IntegerValue { value: 10, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("u16_val".to_string(), Variable {
        r#type: Type::U16,
        initial: Some(Value::Integer(IntegerValue { value: 20, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("u32_val".to_string(), Variable {
        r#type: Type::U32,
        initial: Some(Value::Integer(IntegerValue { value: 5, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("i8_val".to_string(), Variable {
        r#type: Type::I8,
        initial: Some(Value::Integer(IntegerValue { value: 3, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("i32_val".to_string(), Variable {
        r#type: Type::I32,
        initial: Some(Value::Integer(IntegerValue { value: 7, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("i64_val".to_string(), Variable {
        r#type: Type::I64,
        initial: Some(Value::Integer(IntegerValue { value: 100, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("result_u8_u16".to_string(), Variable {
        r#type: Type::U16,
        initial: Some(Value::Integer(IntegerValue { value: 0, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("result_i8_u16".to_string(), Variable {
        r#type: Type::I32,
        initial: Some(Value::Integer(IntegerValue { value: 0, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("result_i32_i64".to_string(), Variable {
        r#type: Type::I64,
        initial: Some(Value::Integer(IntegerValue { value: 0, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("result_widening".to_string(), Variable {
        r#type: Type::U16,
        initial: Some(Value::Integer(IntegerValue { value: 0, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("result_truty".to_string(), Variable {
        r#type: Type::Bool,
        initial: Some(Value::Bool(false)),
        output: false,
    });
    variables.insert("flag".to_string(), Variable {
        r#type: Type::Bool,
        initial: Some(Value::Bool(true)),
        output: false,
    });
    variables.insert("threshold".to_string(), Variable {
        r#type: Type::U8,
        initial: Some(Value::Integer(IntegerValue { value: 5, fmt: IntegerFmt::Dec })),
        output: false,
    });
    variables.insert("sum".to_string(), Variable {
        r#type: Type::F64,
        initial: Some(Value::Float(pall::machine::FloatValue {
            value: 0.0,
            fmt: pall::machine::FloatFmt::Decimal,
        })),
        output: false,
    });
    variables.insert("target".to_string(), Variable {
        r#type: Type::U8,
        initial: Some(Value::Integer(IntegerValue { value: 0, fmt: IntegerFmt::Dec })),
        output: false,
    });

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

// ── Equality Test ────────────────────────────────────────────────────────────

#[test]
fn test_type_casting_yaml_vs_programmatic() {
    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_TYPE_CASTING)
        .expect("YAML should parse");
    let prog_sm = build_type_casting();
    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ YAML and programmatic StateMachines are equal");
}

// ── Compilation Test ─────────────────────────────────────────────────────────

#[test]
fn test_type_casting_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_TYPE_CASTING).unwrap();
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);
    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());

    let files = result.unwrap();

    // Write to runner's generated directory
    let output_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| "..".to_string());
    let output_dir = std::path::PathBuf::from(&output_dir)
        .join("src/bin/runner/generated/type_casting");

    std::fs::create_dir_all(&output_dir).expect("failed to create output dir");

    for (name, content) in &files {
        let file_path = output_dir.join(name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).unwrap_or_else(|e| {
            panic!("failed to write {}: {}", file_path.display(), e);
        });
    }

    println!("✓ Compilation succeeded, {} files written", files.len());
}
