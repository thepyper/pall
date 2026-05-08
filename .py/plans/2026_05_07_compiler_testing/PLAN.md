# PLAN: Compiler Testing Framework — Phase 1 (counter_test)

## Overview

Transform the creator/runner into an end-to-end test framework:
- **Creator**: handles YAML parsing + programmatic construction equality tests and compilation tests
- **Runner**: handles runtime tick execution and behavior verification

Phase 1 focuses exclusively on counter_test to validate the entire architecture.

---

## File Structure (Target)

```
src/bin/creator/
├── tests/
│   ├── mod.rs                  — test module root (empty, makes tests/ a test crate)
│   └── counter_test.rs         — YAML string + programmatic builder + equality test + compilation test
├── src/
│   └── main.rs                 — stub (fn main() {})

src/bin/runner/
├── src/
│   ├── main.rs                 — stub (fn main() {})
│   ├── stubs.rs                — include! macros, re-exports Persistent/init/tick
│   └── tests/                  — [cfg(test)] modules (same crate as main.rs)
│       ├── mod.rs              — declares test submodules
│       ├── helper.rs           — shared runtime test utilities
│       └── counter_test.rs     — goal reachability + variable value tests
└── generated/
    └── counter_test/
        ├── types.rs            — written by creator test
        ├── tick.rs             — written by creator test
        ├── mod.rs              — written by creator test
        └── group.rs            — written by creator test
```

**Key design decision**: Runner test files are in `src/bin/runner/src/tests/` (source directory), NOT `src/bin/runner/tests/` (integration test directory). This allows them to access `stubs` module via `use crate::stubs::*;` because they're part of the same crate. The `#[cfg(test)] mod tests;` in `main.rs` ensures they're only compiled during tests.

---

## Phase 1A: Creator — YAML String + Programmatic Dual Definition

### Step 1.A.1: Create creator test module directory

- Create `src/bin/creator/tests/` directory
- Create `src/bin/creator/tests/mod.rs` — empty module file to make it a test crate

### Step 1.A.2: Create `src/bin/creator/tests/counter_test.rs`

Create file `src/bin/creator/tests/counter_test.rs` with:

1. **Imports**:
```rust
use pall::compiler::{Compiler, FileSet, RustBackend};
use pall::machine::{
    Action, FullExpression, FullStatement, State, StateMachine, Transition,
    Type, Value, Variable, IntegerFmt, IntegerValue,
};
use std::collections::HashMap;
```

2. **YAML string constant**:
```rust
const YAML_COUNTER_TEST: &str = r#"
StateMachine:
  id: counter_test
  initial: initial
  variables:
    counter:
      type: U32
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
```

3. **Programmatic builder function**:
```rust
fn build_counter_programmatic() -> StateMachine {
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

    let goal_state = State {
        actions: vec![],
        transitions: vec![],
    };
    states.insert("goal".to_string(), goal_state);

    let mut variables = HashMap::new();
    variables.insert(
        "counter".to_string(),
        Variable {
            r#type: Type::U32,
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
```

### Step 1.A.3: Verify YAML string is valid

- The YAML string must follow the `machine_spec.md` format exactly
- Root key: `StateMachine:` (matching serde struct field)
- `do` field in transitions uses `"do"` (serde renames `r#do`)
- `when` uses `null` for always-true
- Actions contain `do` with statement strings
- Verify by running `serde_yaml::from_str::<StateMachine>(YAML_COUNTER_TEST)` in a test

---

## Phase 1B: Creator — Equality Comparison Helper

### Step 1.B.1: Implement semantic StateMachine equality function

Create `fn compare_state_machines(a: &StateMachine, b: &StateMachine) -> Result<(), String>` in `counter_test.rs`:

The function compares:
- `a.id == b.id`
- `a.initial == b.initial`
- `a.states` content equality: same keys, same values (order independent)
- `a.variables` content equality: same keys, same values
- `a.inputs`, `a.signals`, `a.timers`, `a.constants` content equality

For each `HashMap<String, T>` field:
1. Check `a_field.len() == b_field.len()`
2. For each key in `a_field`, check `b_field.get(key) == Some(&a_field[key])`
3. Return descriptive error on mismatch

For `State` comparison (order matters within vectors):
- `a.actions == b.actions` (vector element-by-element)
- `a.transitions == b.transitions` (vector element-by-element)

For `Transition` comparison:
- `a.when == b.when` (FullExpression comparison: raw + expression)
- `a.r#do == b.r#do` (FullStatement vector comparison)
- `a.target == b.target`

For `FullExpression` comparison:
- `a.raw == b.raw`
- `a.expression == b.expression` (Expression enum, derives PartialEq)

For `FullStatement` comparison:
- `a.raw == b.raw`
- `a.statement == b.statement` (Statement struct, derives PartialEq)

For `Variable` comparison: all fields
For `Action` comparison: `when`, `r#do`

Returns `Ok(())` if equal, `Err(desc: String)` with descriptive error if not.

### Step 1.B.2: Verify helper compiles

- Run `cargo check -p creator` to ensure the comparison function compiles
- Fix any issues before proceeding

---

## Phase 1C: Creator — Equality Test

### Step 1.C.1: Write equality test function

Create `#[test] fn test_counter_test_yaml_vs_programmatic()` in `counter_test.rs`:

```rust
#[test]
fn test_counter_test_yaml_vs_programmatic() {
    let yaml_sm: StateMachine = serde_yaml::from_str(YAML_COUNTER_TEST)
        .expect("YAML should parse");
    let prog_sm = build_counter_programmatic();
    compare_state_machines(&yaml_sm, &prog_sm)
        .expect("YAML and programmatic StateMachines should be equal");
    println!("✓ YAML and programmatic StateMachines are equal");
}
```

### Step 1.C.2: Run equality test

- Execute: `cargo test -p creator test_counter_test_yaml_vs_programmatic`
- Verify test passes
- If it fails, debug the mismatch (likely HashMap ordering or serde field naming)

---

## Phase 1D: Creator — Compilation Test

### Step 1.D.1: Write compilation test function

Create `#[test] fn test_counter_test_compilation()` in `counter_test.rs`:

```rust
#[test]
fn test_counter_test_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_COUNTER_TEST).unwrap();
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);
    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    println!("✓ Compilation succeeded");
}
```

### Step 1.D.2: Run compilation test

- Execute: `cargo test -p creator test_counter_test_compilation`
- Verify test passes

---

## Phase 1E: Creator — Write Generated Files to Runner's Directory

### Step 1.E.1: Modify compilation test to write output files

In `test_counter_test_compilation()`, after successful compilation:

```rust
#[test]
fn test_counter_test_compilation() {
    let sm = serde_yaml::from_str::<StateMachine>(YAML_COUNTER_TEST).unwrap();
    let rust_backend = RustBackend::new();
    let compiler = Compiler::new(rust_backend);
    let result = compiler.compile(&[sm]);
    assert!(result.is_ok(), "compilation should succeed: {:?}", result.err());
    
    let files = result.unwrap();
    
    // Write to runner's generated directory
    let output_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| "..".to_string());
    let output_dir = std::path::PathBuf::from(&output_dir)
        .join("src/bin/runner/generated/counter_test");
    
    std::fs::create_dir_all(&output_dir).expect("failed to create output dir");
    
    for (name, content) in &files {
        let file_path = output_dir.join(name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).unwrap_or_else(|e| {
            panic!("failed to write {}: {}", file_path.display(), e);
        });
        println!("  Written: {} ({} bytes)", file_path.display(), content.len());
    }
    
    println!("✓ Compilation succeeded, {} files written", files.len());
}
```

### Step 1.E.2: Run and verify files are written

- Execute: `cargo test -p creator test_counter_test_compilation`
- Verify files written to `src/bin/runner/generated/counter_test/`:
  - `counter_test_types.rs`
  - `counter_test_tick.rs`
  - `mod.rs`
  - `group.rs`
- Confirm file contents look correct (brief inspection)

---

## Phase 1F: Creator — Clean Up Main Binary

### Step 1.F.1: Replace creator/src/main.rs with stub

Replace `src/bin/creator/src/main.rs` with:
```rust
fn main() {}
```

### Step 1.F.2: Verify creator works as test-only crate

- Run: `cargo test -p creator`
- Should compile and run all tests (no main function execution needed)

---

## Phase 1G: Runner — Test Helper Module (Source Module)

### Step 1.G.1: Create runner test modules directory

Create `src/bin/runner/src/tests/` directory (source directory, NOT integration test directory).

### Step 1.G.2: Create `src/bin/runner/src/tests/mod.rs`

```rust
mod helper;
mod counter_test;
```

### Step 1.G.3: Update `main.rs` to include test modules

Replace `src/bin/runner/src/main.rs` with:
```rust
mod stubs;

#[cfg(test)]
mod tests;

fn main() {}
```

### Step 1.G.4: Create `src/bin/runner/src/tests/helper.rs`

Create shared test utilities:

```rust
use crate::stubs::*;

/// Run ticks until the machine's state matches the goal predicate, or max ticks exceeded.
pub fn run_until<F: Fn(&Persistent) -> bool>(
    max_ticks: u32,
    delta_ms: u64,
    goal_check: F,
) -> Result<u32, String> {
    let mut state = init();
    let mut ticks: u32 = 0;
    
    loop {
        if goal_check(&state) {
            return Ok(ticks);
        }
        if ticks >= max_ticks {
            return Err(format!(
                "Goal not reached after {} ticks. Final state: {}",
                max_ticks,
                state.state.as_str()
            ));
        }
        
        let tick_info = TickInfo { delta_ms };
        state = tick(&state, &tick_info).map_err(|e| e.message)?;
        ticks += 1;
    }
}

/// Run a fixed number of ticks, returning the final state.
pub fn run_for(num_ticks: u32, delta_ms: u64) -> Persistent {
    let mut state = init();
    let mut ticks: u32 = 0;
    
    while ticks < num_ticks {
        let tick_info = TickInfo { delta_ms };
        state = tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }
    
    state
}
```

### Step 1.G.5: Verify helper compiles

- Run: `cargo check --bin runner --tests`
- Ensure the `#[cfg(test)] mod tests;` in main.rs compiles correctly
- Verify `use crate::stubs::*;` in helper.rs resolves correctly

---

## Phase 1H: Runner — Include Generated Code (stubs.rs)

### Step 1.H.1: Verify existing stubs.rs include! paths

The existing `stubs.rs` already has:
```rust
mod counter_test_types {
    include!("../generated/counter_test/types.rs");
}
mod counter_test_tick {
    include!("../generated/counter_test/tick.rs");
}
mod group {
    include!("../generated/group.rs");
}
```

These paths are relative to `stubs.rs` at `src/bin/runner/src/stubs.rs`.
- `../generated/counter_test/types.rs` → `src/bin/runner/generated/counter_test/types.rs` ✓

No changes needed — verify paths are correct.

### Step 1.H.2: Verify re-exports

Current re-exports in stubs.rs:
```rust
pub use counter_test_types::Persistent;
pub use counter_test_types::State;
pub use counter_test_tick::{init, tick};
```

These make `Persistent`, `State`, `init()`, `tick()` available via `use crate::stubs::*;` in test modules. ✓

---

## Phase 1I: Runner — Goal Reachability Test

### Step 1.I.1: Create `src/bin/runner/src/tests/counter_test.rs`

```rust
use crate::stubs::*;
use super::helper::*;

/// Test that counter_test reaches the goal state within expected ticks.
#[test]
fn test_counter_test_reaches_goal() {
    // expected: ~12 ticks (initial→counting on tick 1, counter increments each tick,
    // counter >= 10 fires transition around tick 11-12)
    let max_ticks = 20;
    let result = run_until(max_ticks, 1000, |state| {
        state.state.as_str() == "goal"
    });
    
    let ticks = result.expect("machine should reach goal within max ticks");
    println!("✓ Reached goal at tick {}", ticks);
    assert!(ticks >= 1, "should have taken at least one tick");
    assert!(ticks <= 20, "expected ~12 ticks, got {}", ticks);
}
```

### Step 1.I.2: Run goal reachability test

- Execute: `cargo test -p runner test_counter_test_reaches_goal`
- Verify test passes
- Confirm ticks count is in expected range (~12 ticks)

---

## Phase 1J: Runner — Variable Value Check Test

### Step 1.J.1: Add variable value test to counter_test.rs

```rust
/// Test counter value when goal is reached.
#[test]
fn test_counter_test_goal_counter_value() {
    // Run until goal, checking counter value at that point
    let max_ticks = 20;
    let mut ticks: u32 = 0;
    let mut state = init();
    let delta_ms = 1000u64;
    
    loop {
        if state.state.as_str() == "goal" {
            println!("✓ Goal reached at tick {}, counter = {}", ticks, state.counter);
            // Counter should be exactly 10 when transition counter >= 10 fires
            // (action counter += 1 executes first, then transition checks)
            assert_eq!(state.counter, 10, "counter should be 10 when goal is reached");
            break;
        }
        if ticks >= max_ticks {
            panic!("Goal not reached after {} ticks. Final state: {}", max_ticks, state.state.as_str());
        }
        let tick_info = TickInfo { delta_ms };
        state = tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }
}

/// Test counter value after a fixed number of ticks.
#[test]
fn test_counter_test_counter_at_tick() {
    let state = run_for(5, 1000);
    // After 5 ticks:
    // tick 0 (initial state) → tick 1 (tick executes: initial→counting, counter still 0)
    // tick 2 (counting: counter += 1, counter = 1)
    // tick 3 (counter = 2)
    // tick 4 (counter = 3)
    // tick 5 (counter = 4)
    println!("✓ After 5 ticks, counter = {}", state.counter);
    assert_eq!(state.counter, 4, "counter should be 4 after 5 ticks");
}
```

### Step 1.J.2: Run variable value tests

- Execute: `cargo test -p runner test_counter_test_goal_counter_value`
- Execute: `cargo test -p runner test_counter_test_counter_at_tick`
- Verify both tests pass
- Confirm values match expectations

---

## Phase 1K: Cleanup — Runner Main Binary

### Step 1.K.1: Replace runner/src/main.rs with stub

Replace `src/bin/runner/src/main.rs` with:
```rust
mod stubs;

#[cfg(test)]
mod tests;

fn main() {}
```

### Step 1.K.2: Verify runner compiles as test-only crate

- Run: `cargo check --bin runner --tests`
- Ensure no compilation errors
- Verify no unused code warnings from dead main() logic (main() is now empty)

---

## Phase 1L: Final Verification

### Step 1.L.1: Run all creator tests

- Execute: `cargo test -p creator`
- Verify all tests pass:
  - `test_counter_test_yaml_vs_programmatic` — ✓ equality
  - `test_counter_test_compilation` — ✓ codegen + file write

### Step 1.L.2: Run all runner tests

- Execute: `cargo test -p runner`
- Verify all tests pass:
  - `test_counter_test_reaches_goal` — ✓ goal reachability
  - `test_counter_test_goal_counter_value` — ✓ goal counter value
  - `test_counter_test_counter_at_tick` — ✓ counter at fixed tick

### Step 1.L.3: Verify file structure

Check all expected files exist:
- `src/bin/creator/tests/mod.rs` — creator test module root
- `src/bin/creator/tests/counter_test.rs` — creator tests
- `src/bin/runner/src/tests/mod.rs` — runner test module root
- `src/bin/runner/src/tests/helper.rs` — runner test helper
- `src/bin/runner/src/tests/counter_test.rs` — runner tests
- `src/bin/runner/generated/counter_test/types.rs` — generated
- `src/bin/runner/generated/counter_test/tick.rs` — generated
- `src/bin/runner/generated/counter_test/mod.rs` — generated
- `src/bin/runner/generated/counter_test/group.rs` — generated

### Step 1.L.4: Verify no regressions

- Run: `cargo check` on the full project
- Run: `cargo test` (all packages)
- Ensure no new warnings or errors
- Verify existing tests in `src/machine/parser/` and `src/compiler/validation.rs` still pass

---

## Phase 2 Preview (Not Yet Started)

After Phase 1 is COMPLETELY PERFECT:

### Phase 2A: Arithmetic expression test machine
- Machine testing +, -, *, /, %, ==, != on variables
- YAML + programmatic dual definition in `tests/arithmetic_expr.rs`

### Phase 2B: Bitwise/logical operator test machine
- Machine testing &, |, ^, !, &&, ||, ^^
- YAML + programmatic dual definition

### Phase 2C: Multi-type variable test machine
- Machine with Bool, U8, I8, U16, I16, U32, I32, U64, I64, F32, F64, String

### Phase 2D: Signal test machine
- Signals computed from expressions

### Phase 2E: Timer test machine
- Timers accumulating with/without conditions

### Phase 2F: Multi-machine link test
- Machine group with 2+ machines, links propagating values

### Phase 2G: Transition ordering test
- Machine with "deceiving" transitions (same condition, multiple)
- Only first fires; others lead to error state

### Phase 2H: Action with condition test
- Actions with when conditions

---

## Completion Criteria

- [ ] `test_counter_test_yaml_vs_programmatic` passes (equality test)
- [ ] `test_counter_test_compilation` passes (codegen test + file write)
- [ ] `test_counter_test_reaches_goal` passes (runtime goal test)
- [ ] `test_counter_test_goal_counter_value` passes (runtime counter check)
- [ ] `test_counter_test_counter_at_tick` passes (runtime counter check)
- [ ] `cargo test -p creator` runs all creator tests successfully
- [ ] `cargo test -p runner` runs all runner tests successfully
- [ ] File structure: one file per group in creator/tests and runner/src/tests/
- [ ] Generated files written to runner/generated by creator compilation test
- [ ] No regressions in existing tests
- [ ] Phase 1 COMPLETE before beginning Phase 2
