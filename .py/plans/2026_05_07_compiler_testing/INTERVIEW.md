# INTERVIEW: Compiler Testing Framework

## Objective

Transform the `creator` and `runner` executables into a comprehensive end-to-end test framework for the Pall compiler. The creator should generate many machine definitions and machine groups from YAML strings; the runner should execute them, verify goal reachability, and check expected behaviors.

## Key Decisions

### 1. YAML String Definition

- Machines are defined as Rust const strings using raw multi-line strings: `const YAML_<NAME>: &str = r#"..."#;`
- YAML format follows the existing `machine_spec.md` specification exactly (the same format the parser/serde deserializer expects)
- Each machine group has one YAML string and one programmatic `StateMachine` construction

### 2. Dual-Definition Equality Test

Each test has TWO definitions of the same machine:
- **YAML path**: YAML string → serde deserialization → `StateMachine`
- **Programmatic path**: Rust code building `StateMachine` directly via `State`, `Transition`, `Variable`, etc.

The two `StateMachine` objects must be equal — this is the primary test of YAML parser correctness vs programmatic construction.

### 3. StateMachine Comparison

- `StateMachine` uses `HashMap` for `states`, `variables`, `inputs`, `signals`, `timers`, `constants`
- HashMap doesn't implement stable ordering, so `PartialEq` won't work for content comparison
- **Custom semantic equality function** is needed: compare sizes, then compare each key's value by key name

### 4. Generated Code Comparison

- NOT compared between YAML and programmatic paths. Code generation is deterministic, so if StateMachines are equal, generated code will also be equal.
- Only the `StateMachine` equality test is performed.

### 5. Creator / Runner Division — "Divide et Impera"

- **Creator** handles compiler-related tests:
  - YAML vs programmatic StateMachine equality test
  - Compilation test (codegen succeeds without errors)
  - Writes generated files to `src/bin/runner/generated/<group>/`
- **Runner** handles runtime-related tests:
  - Includes generated code via `include!` macros
  - Executes tick loops
  - Checks goal reachability
  - Checks variable values at specific ticks
- **Shared test helper module**: generic utilities (run_loop_until_goal, run_for_ticks, etc.) in runner, with per-group files containing machine-specific test logic
- **File-per-group structure**: one Rust file per machine group in `src/bin/runner/tests/`
- `mod.rs` in the tests directory to register all test modules

This division preserves the existing architecture:
- Creator is a binary that compiles machines → add equality and compilation tests here
- Runner is a binary that includes generated code → add runtime tick tests here
- No circular dependency issues

### 6. Test Coverage (Phased)

#### Phase 1 (current): counter_test only
- Single-machine group: counter_test
- Convert existing counter_test to YAML + programmatic format
- Verify all test types work end-to-end

#### Phase 2 (future): expand with new machines
All compiler features to be tested:
- Simple single-state machine
- Multi-state with transitions (counter_test pattern)
- Actions with conditions
- Variables of all types (Bool, U8, I8, U16, I16, U32, I32, U64, I64, F32, F64, String)
- Expressions: binary operators (+, -, *, /, %, ==, !=, <, <=, >, >=, &, |, ^, &&, ||), unary operators (-, !, ~)
- Expression precedence (parentheses override)
- Assignment operators (=, +=, -=, *=, /=, %=, &=, |=, ^=, &&=, ||=)
- Signals (computed from expressions)
- Timers (accumulation with/without when condition)
- Constants
- Inputs
- Multi-machine groups with links
- Complex/nested expressions
- Transition ordering (deceiving: multiple transitions with same condition — only first fires, others go to error state)

### 7. Test Execution

- `cargo test` is the test runner
- All test functions prefixed with `test_`
- Tests are discovered automatically via Rust test harness
- Results: pass/fail via cargo test output

### 8. Machine Group Definition

- A "group" = one or more machines that may be linked together
- Single-machine groups are the simplest case (like counter_test)
- Multi-machine groups used for link/propagation tests
- Each group = one file in `src/bin/runner/tests/`

### 9. End-to-End Verification

Priority order for verification:
1. **Goal reachability**: machine reaches expected goal state within max ticks (not too many more than expected)
2. **Variable value checks**: at specific ticks or upon reaching goal, verify expected values
   - Example: when counter_test reaches goal, counter should have a specific value
3. **Transition ordering**: use "deceiving transitions" (5 transitions with same condition in one state) — only first should fire; others cause transition to an "error" state
4. **Timer accumulation**: use fixed `delta_ms` (e.g., 1000) for deterministic testing

### 10. File Structure

```
src/bin/runner/tests/
├── mod.rs              — test module registry (include all test files)
├── counter_test.rs     — Phase 1: single-machine group (counter_test)
├── arithmetic_expr.rs  — Phase 2: arithmetic expressions
├── multi_machine_link.rs — Phase 2: multi-machine group with links
└── ...
```

### 11. Counter_test as Baseline

- Existing counter_test will be converted to the new format
- counter_test: initial → counting (counter+=1 per tick) → goal (when counter >= 10)
- Expected: reaches goal after approximately 12 ticks, final counter value around 10-12

### 12. Runner Main Binary

- The runner's `main.rs` is no longer needed (all logic moves to tests)
- Keep the runner binary target but it may be minimal or removed

### 13. Creator Organization

- One Rust file per machine group
- Clean code organization — one group = one file
- Each file contains: YAML string + programmatic construction + test functions

### 14. Phased Approach

- **Phase 1**: Get counter_test completely working in the new format
  - Creator side: YAML string + programmatic dual definition, equality test, compilation test
  - Runner side: include generated code, goal reachability test, variable value test
  - File structure validated (creator groups + runner tests)
  - `cargo test` integration working end-to-end
- **Phase 2**: Only after Phase 1 is "completely perfect", add new machines one by one

### 15. Creator Test Helper Module

- Generic utilities shared across all creator test files:
  - `check_state_machine_equality(yaml_sm, programmatic_sm)` — custom YAML vs programmatic comparison (semantic, order-independent)
  - `compile_and_write(machine, output_dir)` — compile machine and write generated files

### 16. Runner Test Helper Module

- Generic utilities shared across all runner test files:
  - `run_until_goal(state_init, max_ticks, delta_ms)` — execute ticks until goal or max ticks, return ticks taken
  - `run_for_ticks(state_init, num_ticks, delta_ms)` — execute N ticks, return final state
  - `check_goal_reached(state, expected_goal, max_ticks, delta_ms)` — generic goal check
  - `check_variable_at_tick(state, var_name, expected_value, tick_num)` — specific value check
