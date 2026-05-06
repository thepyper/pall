# PLAN: Generate Code, Build Micro-Runtime, and Test End-to-End

## Phase 1: Crate Preparation

### Step 1.1: Add chrono dependency to Cargo.toml
- Add `chrono = "0.4"` to `[dependencies]` in `Cargo.toml`
- This enables realistic delta_ms generation in the micro-runtime

### Step 1.2: Add binary target to Cargo.toml
- Add a `[[bin]]` section for `micro_runtime` pointing to `src/bin/micro_runtime/main.rs`
- Keep the existing default binary (src/main.rs) intact

---

## Phase 2: Create Micro-Runtime Binary Structure

### Step 2.1: Create directory structure
- Create `src/bin/micro_runtime/src/` directory
- Create `src/bin/micro_runtime/Cargo.toml` (depends on the parent pall library)

### Step 2.2: Write src/bin/micro_runtime/Cargo.toml
- Package name: `micro_runtime`
- Dependencies: `pall` (path: `../../`), `chrono`
- Set `path = "src/main.rs"`

### Step 2.3: Write src/bin/micro_runtime/src/lib.rs — Stub modules
- Create `pub mod error;` stub with `TickError` type (minimal impl to satisfy generated imports)
- Create `pub mod TickInfo;` stub with `pub delta_ms: u64`
- These stubs resolve the imports the generated code expects: `use super::error::TickError;` and `use super::TickInfo;`

### Step 2.4: Write src/bin/micro_runtime/src/lib.rs — Include macros
- Add `include_str!` / `include!` macros that embed the generated code files into the module
- Structure: `mod counter_test_types;`, `mod counter_test_tick;`, `mod group;`
- Use `include!("generated/counter_test_types.rs")` etc.

---

## Phase 3: Write Main — Code Generation

### Step 3.1: Write src/bin/micro_runtime/src/main.rs — Imports
- Import `pall::machine::StateMachine` and all required types
- Import `pall::compiler::{Compiler, RustBackend}`
- Import `std::fs` and `std::path::PathBuf` for temp dir management

### Step 3.2: Write src/bin/micro_runtime/src/main.rs — Build the example machine
- Create a function `build_counter_machine() -> StateMachine` that builds:
  - Machine id: `"counter_test"`
  - Initial state: `"initial"`
  - Variables: `counter` (U32, initial 0)
  - States:
    - `"initial"`: always-true transition to `"counting"` (no when, no do)
    - `"counting"`: action `counter += 1` (every tick), transition when `counter >= 10` to `"goal"`
    - `"goal"`: no transitions (dead end)
- Use `FullExpression::parse()` and `FullStatement::parse()` for condition/do clauses

### Step 3.3: Write src/bin/micro_runtime/src/main.rs — Compilation
- Create `Compiler::new(RustBackend::new())`
- Call `compiler.compile(&[machine])` → `Result<FileSet, Vec<CompileError>>`
- Handle errors by printing them and aborting

---

## Phase 4: Write Main — File I/O and Code Placement

### Step 4.1: Write src/bin/micro_runtime/src/main.rs — Temp directory
- Create a temp directory (e.g., `std::env::temp_dir().join("pall_generated")`)
- Ensure directory is clean (remove old files)
- Write each file from `FileSet` to the temp dir
- Log which files were written

### Step 4.2: Write src/bin/micro_runtime/src/main.rs — Print generated code
- For debugging: print the contents of each generated file to stdout
- Show the temp directory path for manual inspection

---

## Phase 5: Write Main — Runtime Execution

### Step 5.1: Write src/bin/micro_runtime/src/main.rs — TickInfo creation
- Create a function that builds `TickInfo { delta_ms: 1000 }` (fixed 1 second per tick)
- Optionally use `chrono::Utc::now()` for dynamic delta_ms (keep for future)

### Step 5.2: Write src/bin/micro_runtime/src/main.rs — Tick loop
- Create a function `run_machine() -> Result<(), String>`:
  1. Call `counter_test_tick::init()` → `Persistent`
  2. Loop up to 100 times:
     a. Build `TickInfo`
     b. Call `counter_test_tick::tick(&state, &tick_info)` → `Result<Update, TickError>`
     c. Apply `Update` to `Persistent` (merge: overwrite Some values, keep state_name updated)
     d. Check if `state.state.as_str() == "goal"`
     e. If goal reached, break and report success
  3. If loop exits without reaching goal, report failure

### Step 5.3: Write src/bin/micro_runtime/src/main.rs — Update application
- Helper function to merge `Update` into `Persistent`:
  - For each variable/signal/timer: if `Update` has `Some(val)`, set it in `Persistent`
  - Update `state_name` to match `state.state.as_str()`

---

## Phase 6: Write Tests

### Step 6.1: Write #[test] — Single machine reaches goal
- `#[test] fn test_counter_reaches_goal()`:
  - Build machine, compile, generate code
  - Run tick loop
  - Assert goal state reached
  - Assert counter variable equals 10

### Step 6.2: Write #[test] — Generated code compiles
- `#[test] fn test_generated_code_compiles()`:
  - Build machine, compile
  - Assert `FileSet` is Ok
  - Assert expected files are present (counter_test_types.rs, counter_test_tick.rs, group.rs, mod.rs)

### Step 6.3: Write #[test] — Max ticks not exceeded
- `#[test] fn test_goal_reached_within_max_ticks()`:
  - Same as goal test, but assert ticks < 100 (should be ~11)

---

## Phase 7: Write Machine Specification

### Step 7.1: Write machine_spec.md — Overview section
- Introduction: what pall is, what the machine format represents
- High-level structure: StateMachine as root, states with actions/transitions

### Step 7.2: Write machine_spec.md — YAML format specification
- Document every field of `StateMachine`: id, initial, states, inputs, signals, timers, variables, constants
- Document `State`: actions, transitions
- Document `Transition`: when, do, target
- Document `Action`: when, do
- Document `Variable`: type, initial, output
- Document `Input`, `Signal`, `Timer`, `Constant`

### Step 7.3: Write machine_spec.md — Expression format
- Literals: integers (dec, hex, oct, bin), floats, strings (double/single quote, escape sequences)
- References: variable names, state_name
- Binary operators: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&`, `|`, `^`, `&&`, `||`, `^^`
- Unary operators: `-`, `!`, `~`
- Precedence: multiplication > addition > comparison > bitwise > logical
- Parentheses for grouping

### Step 7.4: Write machine_spec.md — Statement format
- Syntax: `target operator expression`
- Assignment operators: `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `&&=`, `||=`, `^^=`
- Examples

### Step 7.5: Write machine_spec.md — Link format
- Syntax: `source_id.output_name`
- Used in Input link field
- Example: `link: "sensor_module.temperature"`

### Step 7.6: Write machine_spec.md — Functional semantics
- Tick: deterministic, single-threaded, state machine semantics
- Init: creates initial Persistent with initial state and default values
- State transitions: evaluated in order, first match wins, returns Update immediately
- Actions: executed before transitions in each state
- Signal computation: evaluated after state/transition logic, assigned to Update
- Timer accumulation: increments by delta_ms when condition met, resets to 0 otherwise
- Group tick: link propagation first, then per-machine tick
- No parallelism, no async

---

## Phase 8: Verification

### Step 8.1: Verify parent crate compiles
- Run `cargo check` in `pall/` root
- Verify no errors or new warnings

### Step 8.2: Verify existing tests still pass
- Run `cargo test` in `pall/` root
- Assert all 43 original tests pass

### Step 8.3: Verify micro_runtime compiles
- Run `cargo check -p micro_runtime` (or `cargo check --bin micro_runtime`)
- Verify no errors

### Step 8.4: Run micro_runtime tests
- Run `cargo test --bin micro_runtime`
- Assert all tests pass (goal reached, code compiles, ticks within limit)

### Step 8.5: Run micro_runtime as binary (manual smoke test)
- Run `cargo run --bin micro_runtime`
- Verify generated code prints, tick loop succeeds, goal is reached

### Step 8.6: Verify machine_spec.md is complete
- Read through the document
- Verify all sections from Phase 7 are present and accurate
