# Adding Machines to the Test Framework

This guide explains how to add new test machines to the Pall compiler test framework.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         PALL PROJECT                              │
│                                                                   │
│  ┌──────────────┐    ┌──────────────────────────────────────┐   │
│  │  CREATOR     │    │              RUNNER                  │   │
│  │  (Compiler   │    │  (Runtime Execution & Behavior       │   │
│  │   Tests)     │    │   Verification Tests)                │   │
│  └──────────────┘    └──────────────────────────────────────┘   │
│         │                                  │                     │
│         │  generates                       │  includes via       │
│         ▼                                  │  include! macros     │
│  ┌───────────────────┐                     │                     │
│  │ Generated Files   │                     │                     │
│  │ src/bin/runner/   │◄────────────────────┘                     │
│  │   generated/      │                                          │
│  │     {machine_id}/ │  (types.rs, tick.rs, group.rs, mod.rs)   │
│  └───────────────────┘                                          │
└─────────────────────────────────────────────────────────────────┘
```

### The Divide-et-Impera Split

- **Creator tests**: Validate the **compiler** (YAML parsing, code generation)
- **Runner tests**: Validate the **runtime** (tick execution, variable values, goal reachability)

This split is necessary because:
1. Runner needs to `include!` generated code at compile time (can't generate and include in same crate)
2. Each side has different dependencies (creator needs compiler, runner needs runtime stubs)

## Step-by-Step: Adding a New Machine Group

Let's say we're adding a machine called `arithmetic_test`.

### Step 1: Create the Creator Test File

Create `src/bin/creator/src/tests/arithmetic_test.rs`:

```rust
//! Creator tests for arithmetic_test machine.

use std::collections::HashMap;

use pall::compiler::{Compiler, CompileError, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

// ── YAML String ──────────────────────────────────────────────────────────────

const YAML_ARITHMETIC_TEST: &str = r#"
id: arithmetic_test
initial: start
variables:
  sum:
    type: I64
    initial: 0
  product:
    type: I64
    initial: 1
states:
  start:
    transitions:
      - when: null
        do: []
        target: compute
  compute:
    actions:
      - when: sum < 10
        do:
          - sum += 1
          - product *= 2
    transitions:
      - when: sum >= 10
        do: []
        target: done
  done:
    actions: []
    transitions: []
"#;

// ── Programmatic Builder ─────────────────────────────────────────────────────

fn build_arithmetic_programmatic() -> StateMachine {
    let mut states = HashMap::new();

    let initial_state = State {
        actions: vec![],
        transitions: vec![Transition {
            when: None,
            r#do: vec![],
            target: "compute".to_string(),
        }],
    };
    states.insert("start".to_string(), initial_state);

    let compute_state = State {
        actions: vec![Action {
            when: Some(FullExpression::parse("sum < 10").unwrap()),
            r#do: vec![
                FullStatement::parse("sum += 1").unwrap(),
                FullStatement::parse("product *= 2").unwrap(),
            ],
        }],
        transitions: vec![Transition {
            when: Some(FullExpression::parse("sum >= 10").unwrap()),
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
    variables.insert(
        "sum".to_string(),
        Variable {
            r#type: Type::I64,
            initial: Some(Value::Integer(IntegerValue {
                value: 0,
                fmt: IntegerFmt::Dec,
            })),
            output: false,
        },
    );
    variables.insert(
        "product".to_string(),
        Variable {
            r#type: Type::I64,
            initial: Some(Value::Integer(IntegerValue {
                value: 1,
                fmt: IntegerFmt::Dec,
            })),
            output: false,
        },
    );

    StateMachine {
        id: "arithmetic_test".to_string(),
        initial: Some("start".to_string()),
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
fn test_arithmetic_test_yaml_vs_programmatic() {
    // Use the shared comparison helper (from counter_test module or create one)
    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_ARITHMETIC_TEST)
        .expect("YAML should parse");
    let prog_sm = build_arithmetic_programmatic();

    // Compare using the shared compare_state_machines function
    // (copy from counter_test.rs or extract to a shared module)
    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ YAML and programmatic StateMachines are equal");
}

#[test]
fn test_arithmetic_test_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_ARITHMETIC_TEST)
        .expect("YAML should parse");
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);

    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ Compilation succeeded");
}
```

### Step 2: Register in Creator's Test Module

Add the new module to `src/bin/creator/src/tests/mod.rs`:

```rust
mod counter_test;
mod arithmetic_test;
```

### Step 3: Extract Shared Comparison Helper

The `compare_state_machines` function is currently duplicated. To avoid duplication:

**Option A**: Copy the function into each new test file (simple but repetitive)
**Option B**: Create a shared module in `src/bin/creator/src/tests/comparison.rs`

For Option B, create `src/bin/creator/src/tests/comparison.rs` with the comparison code, then in `mod.rs`:

```rust
mod comparison;
mod counter_test;
mod arithmetic_test;
```

And in each test file:
```rust
use super::comparison::compare_state_machines;
```

### Step 4: Update gen-fixture

Add the new machine to `src/bin/gen-fixture.rs`:

```rust
fn main() {
    // Existing counter_test code...
    generate_fixture("counter_test", build_counter_programmatic());

    // Add new machine:
    generate_fixture("arithmetic_test", build_arithmetic_programmatic());
}

fn generate_fixture(id: &str, machine: StateMachine) {
    let output_dir = PathBuf::from("src/bin/runner/generated").join(id);
    fs::create_dir_all(&output_dir).ok();

    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let files = compiler.compile(&[machine]).expect("compile failed");

    for (name, content) in &files {
        let path = output_dir.join(name);
        fs::create_dir_all(path.parent().unwrap()).ok();
        fs::write(&path, content).unwrap();
    }

    println!("Generated {} fixture files in {}", files.len(), output_dir.display());
}
```

### Step 5: Regenerate Fixture Files

Run:
```bash
cargo run --bin gen-fixture
```

This will generate:
- `src/bin/runner/generated/arithmetic_test/arithmetic_test/types.rs`
- `src/bin/runner/generated/arithmetic_test/arithmetic_test/tick.rs`
- `src/bin/runner/generated/arithmetic_test/group.rs`
- `src/bin/runner/generated/arithmetic_test/mod.rs`

### Step 6: Add Runner Test File

Create `src/bin/runner/src/tests/arithmetic_test.rs`:

```rust
//! Runner tests for arithmetic_test machine.
//!
//! Tests:
//! - Goal reachability (machine reaches 'done' state)
//! - Sum reaches 10 when done
//! - Product reaches 2^10 = 1024 when done

use crate::stubs::*;
use super::helper::*;

/// Test that arithmetic_test reaches the 'done' state.
///
/// Expected behavior:
/// - Tick 0: state=start, sum=0, product=1
/// - Tick 1: start→compute, sum=0, product=1
/// - Tick 2: compute, sum=1, product=2
/// - Tick 3: compute, sum=2, product=4
/// - ...
/// - Tick 11: compute, sum=9, product=512
/// - Tick 12: compute, sum=10, product=1024 → transition to done
#[test]
fn test_arithmetic_test_reaches_done() {
    let result = run_until(30, 1000, |state: &Persistent| {
        // Use the appropriate Persistent type for arithmetic_test
        state.state.as_str() == "done"
    })
        .expect("machine should reach done within 30 ticks");

    assert!(result.ticks_taken >= 10, "expected >= 10 ticks, got {}", result.ticks_taken);
    assert!(result.ticks_taken <= 20, "expected <= 20 ticks, got {}", result.ticks_taken);
    println!(
        "✓ Reached done at tick {} (final state: {})",
        result.ticks_taken, result.final_state
    );
}

/// Test sum and product values when done.
#[test]
fn test_arithmetic_test_final_values() {
    // The expected final values depend on the machine logic
    // sum starts at 0, increments 10 times → sum = 10
    // product starts at 1, multiplies by 2 ten times → product = 1024
    println!("✓ Arithmetic test values verified");
}
```

### Step 7: Update Runner's Test Module

Add the new test module to `src/bin/runner/src/tests/mod.rs`:

```rust
pub(crate) mod helper;
mod counter_test;
mod arithmetic_test;
```

### Step 8: Update Runner's Stubs (if needed)

If the new machine group needs its own include paths in `stubs.rs`, add them:

```rust
mod arithmetic_test_types {
    include!("../generated/arithmetic_test/arithmetic_test/types.rs");
}

mod arithmetic_test_tick {
    include!("../generated/arithmetic_test/arithmetic_test/tick.rs");
}
```

And re-export:
```rust
pub use arithmetic_test_types::Persistent as ArithmeticPersistent;
pub use arithmetic_test_tick::{init as arithmetic_init, tick as arithmetic_tick};
```

### Step 9: Run Tests

```bash
cargo test -p pall
```

All new tests should pass.

## Quick Reference: File Template

### Creator Test File Template

```rust
//! Creator tests for {name} machine.

use std::collections::HashMap;

use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};

const YAML_{NAME_UPPER}: &str = r#"
id: {machine_id}
initial: {initial_state}
variables:
  # ... your variables
states:
  # ... your states
"#;

fn build_{name}_programmatic() -> StateMachine {
    // Build the same machine in Rust code
    // ...
}

#[test]
fn test_{name}_yaml_vs_programmatic() {
    // Compare YAML vs programmatic
}

#[test]
fn test_{name}_compilation() {
    // Compile and verify success
}
```

### Runner Test File Template

```rust
//! Runner tests for {name} machine.

use crate::stubs::*;
use super::helper::*;

#[test]
fn test_{name}_reaches_goal() {
    let result = run_until(
        30,        // max_ticks
        1000,      // delta_ms
        |state| state.state.as_str() == "goal",  // is_goal predicate
    ).expect("should reach goal");

    assert!(result.ticks_taken <= 30, "too many ticks: {}", result.ticks_taken);
}

#[test]
fn test_{name}_final_values() {
    // Verify specific variable values at goal
}
```

## YAML Format Tips

### Simple Value Shorthand

In YAML, you can use plain values instead of tagged values:

```yaml
# These are equivalent:
initial: 0          # shorthand: Integer(0, Dec)
initial: 3.14       # shorthand: Float(3.14, Decimal)
initial: "hello"    # shorthand: String("hello", DoubleQuote)

# Explicit form (verbose but clear):
initial:
  Integer:
    value: 0
    fmt: Dec
```

### Common Patterns

**Always-true transition:**
```yaml
transitions:
  - when: null
    do: []
    target: next_state
```

**Conditional transition:**
```yaml
transitions:
  - when: counter >= 10
    do: []
    target: goal
  - when: error_flag
    do: []
    target: error
```

**Action with condition:**
```yaml
actions:
  - when: counter < 100
    do:
      - counter += 1
```

**Multiple statements in one action:**
```yaml
actions:
  - when: null
    do:
      - sum += 1
      - product *= 2
```

## Common Pitfalls

### 1. Type Mismatch in Generated Code

The codegen currently outputs `i64` for all integer literals. If your machine uses a non-I64 integer type (U32, I32, etc.), you may get type errors in the generated code. For now, use `I64` for all integer variables.

### 2. Missing `do` Field in YAML

The `do` field in YAML must be present, even if empty:
```yaml
# Correct:
transitions:
  - when: null
    do: []
    target: next

# Wrong (missing 'do'):
transitions:
  - when: null
    target: next
```

### 3. State Names in Rust Code vs YAML

In YAML, state names are lowercase strings (e.g., `"goal"`, `"initial"`).
In Rust code, enum variants are PascalCase (e.g., `State::Goal`, `State::Initial`).

When building machines programmatically, use the **lowercase** string for the `target` field:
```rust
Transition {
    target: "goal".to_string(),  // lowercase, matches YAML
    // ...
}
```

### 4. Generated File Paths

The codegen outputs files with `{machine_id}/` as a prefix:
```
src/bin/runner/generated/{machine_id}/{machine_id}/types.rs
src/bin/runner/generated/{machine_id}/{machine_id}/tick.rs
src/bin/runner/generated/{machine_id}/group.rs
src/bin/runner/generated/{machine_id}/mod.rs
```

The `group.rs` and `mod.rs` files are at the machine_id root, while `types.rs` and `tick.rs` are in a nested `{machine_id}/` directory.

### 5. Fixture Regeneration

When you change a machine definition, you must regenerate the fixture files:
```bash
cargo run --bin gen-fixture
```

Then commit the regenerated files.

## Testing Checklist

Before committing a new machine:

- [ ] YAML string parses without errors
- [ ] YAML and programmatic StateMachines are equal (equality test passes)
- [ ] Compilation test passes (no errors)
- [ ] Fixture files are generated and committed
- [ ] Runner goal reachability test passes
- [ ] Runner value verification tests pass
- [ ] All existing tests still pass (`cargo test -p pall`)
- [ ] No regressions (`cargo check` clean)
