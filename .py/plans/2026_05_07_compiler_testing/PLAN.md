# PLAN: Compiler Testing Framework — Phase 1 (counter_test)

## Overview

Transform the creator/runner into an end-to-end test framework. Creator handles YAML parsing + programmatic construction equality tests and compilation tests. Runner handles runtime tick execution and behavior verification. Phase 1 focuses exclusively on counter_test to validate the entire architecture.

## Phase 1A: Creator — YAML String + Programmatic Dual Definition

### Step 1.A.1: Create creator test module directory

- Create `src/bin/creator/tests/` directory
- Create `src/bin/creator/tests/mod.rs` — empty module file to make it a test crate

### Step 1.A.2: Create `src/bin/creator/tests/counter_test.rs`

- Create file `src/bin/creator/tests/counter_test.rs`
- Define `const YAML_COUNTER_TEST: &str = r#"..."#` — YAML string following machine_spec.md format
- The YAML string defines the counter_test machine:
  - id: "counter_test"
  - initial: "initial"
  - variable: counter (U32, initial 0)
  - states: initial, counting, goal
  - initial → counting (always-true transition)
  - counting → action: counter += 1, transition: counter >= 10 → goal
  - goal: dead end
- Define `fn build_counter_programmatic() -> StateMachine` — same machine built in Rust code using `StateMachine`, `State`, `Transition`, `Action`, `Variable`, `FullExpression`, `FullStatement`, `Type`, `Value`, `IntegerValue`, `IntegerFmt`

### Step 1.A.3: Verify YAML string is valid

- The YAML string must use the exact format from `machine_spec.md`
- Root key: `StateMachine:` (matching serde struct field)
- States nested under `states:` key
- Transitions nested under `transitions:` key within each state
- Actions nested under `actions:` key within each state
- Variables nested under `variables:` key

## Phase 1B: Creator — Equality Comparison Helper

### Step 1.B.1: Implement semantic StateMachine equality function

- Create `fn compare_state_machines(a: &StateMachine, b: &StateMachine) -> Result<(), String>` in `src/bin/creator/tests/counter_test.rs` (or a shared helper module)
- The function compares:
  - `id` equality
  - `initial` equality
  - `states` content equality (same keys, same values — order independent)
  - `variables` content equality (same keys, same values)
  - `inputs` content equality
  - `signals` content equality
  - `timers` content equality
  - `constants` content equality
- For each HashMap field: check same len, then for each key, compare the value by key
- For `State` comparison: compare `actions` and `transitions` vectors element-by-element (order matters within vectors)
- For `Transition` comparison: compare `when`, `r#do`, `target`
- For `Action` comparison: compare `when`, `r#do`
- For `FullExpression` comparison: compare `raw` and `expression`
- For `FullStatement` comparison: compare `raw` and `statement`
- For `Variable`/`Input`/`Signal`/`Timer`/`Constant` comparison: compare all fields
- Returns `Ok(())` if equal, `Err(msg)` with descriptive error if not

### Step 1.B.2: Verify helper compiles

- Run `cargo check -p creator` to ensure the comparison function compiles
- Fix any issues before proceeding

## Phase 1C: Creator — Equality Test

### Step 1.C.1: Write equality test function

- Create `#[test] fn test_counter_test_yaml_vs_programmatic()` in `src/bin/creator/tests/counter_test.rs`
- Parse YAML: `let yaml_sm: StateMachine = serde_yaml::from_str(YAML_COUNTER_TEST).expect("yaml should parse")`
- Build programmatic: `let prog_sm = build_counter_programmatic()`
- Compare: `compare_state_machines(&yaml_sm, &prog_sm).expect("YAML and programmatic StateMachines should be equal")`
- Print success message on pass

### Step 1.C.2: Run equality test

- Execute `cargo test -p creator test_counter_test_yaml_vs_programmatic`
- Verify test passes
- If it fails, debug the mismatch (likely HashMap ordering or serde field naming)

## Phase 1D: Creator — Compilation Test

### Step 1.D.1: Write compilation test function

- Create `#[test] fn test_counter_test_compilation()` in `src/bin/creator/tests/counter_test.rs`
- Use one of the equal StateMachines (either one — they're equal):
  - `let sm = serde_yaml::from_str::<StateMachine>(YAML_COUNTER_TEST).unwrap();`
- Compile: `let rust_backend = RustBackend::new(); let compiler = Compiler::new(rust_backend); let result = compiler.compile(&[sm]);`
- Assert: `assert!(result.is_ok(), "compilation should succeed")`
- On success, extract the FileSet and count files
- Print which files were generated

### Step 1.D.2: Run compilation test

- Execute `cargo test -p creator test_counter_test_compilation`
- Verify test passes

## Phase 1E: Creator — Write Generated Files to Runner's Directory

### Step 1.E.1: Modify compilation test to write output files

- In `test_counter_test_compilation()`, after successful compilation:
  - Determine output directory: `src/bin/runner/generated/counter_test/`
  - Create the directory: `std::fs::create_dir_all(output_dir).unwrap();`
  - Write each file from the FileSet to the output directory
  - Print which files were written

### Step 1.E.2: Run and verify files are written

- Execute `cargo test -p creator test_counter_test_compilation`
- Verify that files are written to `src/bin/runner/generated/counter_test/`:
  - `counter_test_types.rs`
  - `counter_test_tick.rs`
  - `mod.rs`
  - `group.rs` (or at least the expected files)
- Confirm file contents look correct (brief inspection)

## Phase 1F: Creator — Remove Main Binary Code

### Step 1.F.1: Clean up creator/src/main.rs

- The main binary code that builds and runs counter_test programmatically is no longer needed
- Replace `src/bin/creator/src/main.rs` with a minimal placeholder or remove it
- If the `[[bin]]` target in Cargo.toml needs to remain (for other purposes), keep a stub `fn main() {}`
- Otherwise, remove the `[[bin]]` for creator from `Cargo.toml`

### Step 1.F.2: Verify creator still works as test-only crate

- Run `cargo test -p creator` — should compile and run tests
- No compilation errors

## Phase 1G: Runner — Test Helper Module

### Step 1.G.1: Create runner tests directory

- Create `src/bin/runner/tests/` directory
- Create `src/bin/runner/tests/mod.rs` — empty module file to make it a test crate

### Step 1.G.2: Create test helper module

- Create `src/bin/runner/tests/helper.rs`
- Implement shared test utilities:

```rust
use crate::stubs::*; // or wherever TickError and TickInfo come from
// Also need access to the specific machine's init(), tick(), Persistent type

pub struct TestResult {
    pub ticks_taken: u32,
    pub final_state: String,
    pub error: Option<String>,
}

/// Run ticks until the machine reaches a goal state or max ticks exceeded.
/// Returns (ticks_taken, final_state_string, error_or_none).
pub fn run_until_goal<F: Fn(&Persistent) -> bool>(
    init_fn: F,
    max_ticks: u32,
    delta_ms: u64,
    is_goal: fn(&Persistent) -> bool,
) -> TestResult {
    // ... implementation
}

/// Run a fixed number of ticks, returning the final Persistent state.
/// This is useful for checking variable values at specific tick counts.
pub fn run_for_ticks<F: Fn() -> Persistent>(
    init_fn: F,
    num_ticks: u32,
    delta_ms: u64,
    tick_fn: fn(&Persistent, &TickInfo) -> Result<Persistent, TickError>,
) -> Persistent {
    // ... implementation
}
```

- Keep the helper generic enough to be reused across groups, but concrete enough to compile (needs access to `Persistent`, `TickInfo`, `TickError`, `tick()`, `init()`)

### Step 1.G.3: Verify helper compiles

- Run `cargo check --bin runner --tests` to ensure the test module compiles

## Phase 1H: Runner — Include Generated Code for counter_test

### Step 1.H.1: Update runner's stubs.rs to include generated code

- The existing `stubs.rs` already includes counter_test code via `include!` macros:
  ```rust
  mod counter_test_types { include!("../generated/counter_test/types.rs"); }
  mod counter_test_tick { include!("../generated/counter_test/tick.rs"); }
  ```
- Verify these `include!` paths are correct (relative to stubs.rs location)
- If needed, adjust paths to ensure they resolve correctly during `cargo test`

### Step 1.H.2: Re-export counter_test symbols for test use

- In `stubs.rs` or a new module, re-export:
  - `Persistent` type
  - `init()` function
  - `tick()` function
- Ensure these are accessible from test files

## Phase 1I: Runner — Goal Reachability Test

### Step 1.I.1: Write goal reachability test

- Create `src/bin/runner/tests/counter_test.rs`
- Implement `#[test] fn test_counter_test_reaches_goal()`
- Use the test helper or inline logic:
  - Initialize state with `init()`
  - Run ticks in a loop
  - Check state reaches "goal"
  - Assert goal is reached within expected max ticks (e.g., 20 ticks — counter increments each tick, initial→counting on tick 1, counter reaches 10 around tick 11, transition fires on tick 12)
  - Print ticks taken on success
- Return error if goal not reached within max ticks

### Step 1.I.2: Run goal reachability test

- Execute `cargo test -p runner test_counter_test_reaches_goal`
- Verify test passes
- Confirm ticks count is in expected range (~12 ticks)

## Phase 1J: Runner — Variable Value Check Test

### Step 1.J.1: Write variable value test

- In `src/bin/runner/tests/counter_test.rs`, add:
- Implement `#[test] fn test_counter_test_variable_values()`
- Test logic:
  - Run machine for a known number of ticks (e.g., 5 ticks)
  - Check counter value at that point (after 5 ticks: initial→counting at tick 1, counter=1 at tick 2, counter=2 at tick 3... counter=5 at tick 6)
  - Alternatively: run until goal, then check counter value (should be 10 when transition fires)
  - Assert specific values match expectations
  - Print tick count and values on success

### Step 1.J.2: Run variable value test

- Execute `cargo test -p runner test_counter_test_variable_values`
- Verify test passes
- Confirm counter value matches expectation

## Phase 1K: Cleanup — Remove Old Runner Main Binary

### Step 1.K.1: Remove or simplify runner/src/main.rs

- The old `main.rs` with hardcoded counter_test logic is no longer needed
- Remove the file or replace with a minimal stub
- If `[[bin]] runner` target in Cargo.toml is still needed, keep a stub main

### Step 1.K.2: Verify runner compiles as test-only crate

- Run `cargo check --bin runner --tests`
- Ensure no unused code warnings that would indicate leftover dead code

## Phase 1L: Final Verification

### Step 1.L.1: Run all tests

- Execute `cargo test -p creator -p runner`
- Verify all tests pass:
  - `test_counter_test_yaml_vs_programmatic` (creator)
  - `test_counter_test_compilation` (creator)
  - `test_counter_test_reaches_goal` (runner)
  - `test_counter_test_variable_values` (runner)

### Step 1.L.2: Verify file structure

- Creator tests: `src/bin/creator/tests/mod.rs`, `src/bin/creator/tests/counter_test.rs`
- Runner tests: `src/bin/runner/tests/mod.rs`, `src/bin/runner/tests/counter_test.rs`, `src/bin/runner/tests/helper.rs`
- Generated files: `src/bin/runner/generated/counter_test/*.rs`

### Step 1.L.3: Verify no regressions

- Run `cargo check` on the full project
- Ensure no new warnings or errors
- Verify existing tests in `src/machine/parser/` and `src/compiler/validation.rs` still pass

## Phase 2 Preview (Not Yet Started)

After Phase 1 is completely perfect:

### Phase 2A: Add arithmetic expression test machine
- Machine testing +, -, *, /, %, ==, != on variables
- YAML + programmatic dual definition
- Equality test, compilation test, runtime tests

### Phase 2B: Add bitwise/logical operator test machine
- Machine testing &, |, ^, !, &&, ||, ^^
- YAML + programmatic dual definition

### Phase 2C: Add multi-type variable test machine
- Machine with variables of different types: Bool, U8, I32, F64, String
- YAML + programmatic dual definition

### Phase 2D: Add signal test machine
- Machine with signals computed from expressions
- YAML + programmatic dual definition

### Phase 2E: Add timer test machine
- Machine with timers accumulating with/without conditions
- YAML + programmatic dual definition

### Phase 2F: Add multi-machine link test
- Machine group with 2+ machines, links propagating values
- YAML + programmatic dual definition
- Runtime test verifies link propagation

### Phase 2G: Add transition ordering test
- Machine with "deceiving" transitions (same condition, multiple)
- Only first should fire; others lead to error state
- Runtime test verifies no error state reached

### Phase 2H: Add action with condition test
- Machine with actions that have when conditions
- YAML + programmatic dual definition

## Completion Criteria

- [ ] `test_counter_test_yaml_vs_programmatic` passes (equality test)
- [ ] `test_counter_test_compilation` passes (codegen test)
- [ ] `test_counter_test_reaches_goal` passes (runtime goal test)
- [ ] `test_counter_test_variable_values` passes (runtime variable test)
- [ ] All tests discovered by `cargo test -p creator -p runner`
- [ ] File structure: one file per group in creator/tests and runner/tests
- [ ] Generated files written to runner/generated by creator
- [ ] No regressions in existing tests
- [ ] Plan Phase 1 is COMPLETE before beginning Phase 2
