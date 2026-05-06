# PLAN: Generate Code, Build Micro-Runtime, and Test End-to-End

## Phase 0: Bug Fix — Generated Code Module Imports

### Step 0.1: Fix tick.hbs template — types module reference
- The template uses `use super::types::{Persistent, Update, State};` but the actual module is named `{machine_id}_types`
- In `build_tick_data` (codegen.rs), add `"types_module": format!("{}\types", machine.id)` to the returned JSON
- In `tick.hbs`, replace `use super::types::{Persistent, Update, State};` with `use super::{{{types_module}}}::{Persistent, Update, State};`

### Step 0.2: Fix group.hbs template — types module references
- The `group.hbs` template uses `use {id}_types::{Persistent, Update};`
- This is correct as-is (matches the module name declared in mod.rs)
- Verify no other references need fixing

### Step 0.3: Verify fix compiles
- After fixing tick.hbs, run `cargo check` to ensure the compiler itself still works

---

## Phase 1: Crate Preparation

### Step 1.1: Add chrono dependency to Cargo.toml
- Add `chrono = "0.4"` to `[dependencies]` in `Cargo.toml`
- This enables realistic delta_ms generation in the micro-runtime

### Step 1.2: Add binary targets to Cargo.toml
- Add `[[bin]]` sections for both `creator` (pointing to `src/bin/creator/main.rs`) and `runner` (pointing to `src/bin/runner/main.rs`)
- Keep the existing default binary (`src/main.rs`) intact

---

## Phase 2: Create Creator Binary (Code Generator)

### Step 2.1: Create creator directory structure
- Create `src/bin/creator/src/` directory
- Create `src/bin/creator/Cargo.toml` (depends on the parent pall library)

### Step 2.2: Write src/bin/creator/Cargo.toml
- Package name: `creator`
- Dependencies: `pall` (path: `../../`)
- Set `path = "src/main.rs"`

### Step 2.3: Write src/bin/creator/src/main.rs — Imports
- Import `pall::machine::StateMachine` and all required types (FullExpression, FullStatement, Variable, Constant, Value, IntegerValue, IntegerFmt, etc.)
- Import `pall::compiler::{Compiler, RustBackend}`
- Import `std::fs`, `std::path::PathBuf` for file I/O

### Step 2.4: Write src/bin/creator/src/main.rs — Build the example machine
- Create `fn build_counter_machine() -> StateMachine`:
  - Machine id: `"counter_test"`
  - Initial state: `"initial"`
  - Variables: `counter` (U32, initial 0, fmt: IntegerFmt::Dec)
  - States:
    - `"initial"`: always-true transition to `"counting"` (when: None, do: [], target: "counting")
    - `"counting"`:
      - Action: `counter += 1` (when: None, do: [FullStatement::parse("counter += 1")])
      - Transition: when `counter >= 10` to `"goal"` (when: FullExpression::parse("counter >= 10"), do: [], target: "goal")
    - `"goal"`: no transitions (dead end)

### Step 2.5: Write src/bin/creator/src/main.rs — Compile
- Create `Compiler::new(RustBackend::new())`
- Call `compiler.compile(&[machine])` → `Result<FileSet, Vec<CompileError>>`
- On error: print compilation errors to stderr and exit with code 1

### Step 2.6: Write src/bin/creator/src/main.rs — Write output files
- Output directory: `src/bin/runner/generated/` (relative to project root)
- Create the directory if it doesn't exist
- Write each file from `FileSet` to the output directory:
  - `counter_test_types.rs`
  - `counter_test_tick.rs`
  - `group.rs`
  - `mod.rs`
- Print which files were written and their paths

---

## Phase 3: Create Runner Binary (Micro-Runtime)

### Step 3.1: Create runner directory structure
- Create `src/bin/runner/src/` directory
- Create `src/bin/runner/Cargo.toml`
- Create `src/bin/runner/generated/` directory (the creator writes here)

### Step 3.2: Write src/bin/runner/Cargo.toml
- Package name: `runner`
- Dependencies: `chrono`
- Set `path = "src/main.rs"`

### Step 3.3: Write src/bin/runner/src/lib.rs — Stub modules
Create stubs so generated code imports resolve:

```rust
// error stub
pub mod error {
    use std::fmt;
    pub struct TickError { pub message: String }
    impl fmt::Display for TickError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.message) } }
    impl std::error::Error for TickError {}
}

// TickInfo stub
pub struct TickInfo { pub delta_ms: u64 }
```

### Step 3.4: Write src/bin/runner/src/lib.rs — Include macros
Add module includes for generated files:

```rust
mod counter_test_types { include!("generated/counter_test_types.rs"); }
mod counter_test_tick { include!("generated/counter_test_tick.rs"); }
mod group { include!("generated/group.rs"); }
mod mod_stub { include!("generated/mod.rs"); }
```

Note: `mod.rs` is a reserved filename — include it as `mod_stub` or rename the generated file to `_mod.rs` or `mod_files.rs`.

### Step 3.5: Write src/bin/runner/src/lib.rs — Re-exports
Re-export commonly used items from the included modules for convenience:
- `pub use counter_test_types::{Persistent, Update, State};`
- `pub use counter_test_tick::{tick, init};`

### Step 3.6: Write src/bin/runner/src/main.rs — Imports and entry
- Import items from the lib module
- Main function: call `run_and_test()`, print results

### Step 3.7: Write src/bin/runner/src/main.rs — TickInfo creation
- Create `fn make_tick_info() -> TickInfo { TickInfo { delta_ms: 1000 } }`
- Fixed 1 second per tick (deterministic for testing)

### Step 3.8: Write src/bin/runner/src/main.rs — Update application helper
- `fn apply_update(state: &mut Persistent, update: &Update)` — merges Update into Persistent:
  - For each variable: if `update.counter == Some(v)`, set `state.counter = v`
  - For each signal: if `update.signal_name == Some(v)`, set `state.signal_name = v`
  - Update `state.state_name = state.state.as_str().to_string()`

### Step 3.9: Write src/bin/runner/src/main.rs — Tick loop
- `fn run_machine() -> Result<u32, String>`:
  1. Create `Persistent` via `counter_test_tick::init()`
  2. Loop from tick 0 to 99 (max 100):
     a. Build `TickInfo { delta_ms: 1000 }`
     b. Call `counter_test_tick::tick(&state, &tick_info)` → `Result<Update, TickError>`
     c. Apply update via `apply_update(&mut state, &update)`
     d. Check if `state.state.as_str() == "goal"`
     e. If yes, return `tick + 1` (number of ticks taken)
  3. If loop completes without reaching goal, `Err("goal not reached".to_string())`

### Step 3.10: Write src/bin/runner/src/main.rs — Run and test
- `fn run_and_test()`:
  1. Print "Starting micro-runtime..."
  2. Call `run_machine()`
  3. On success: print ticks taken, final counter value, "Goal reached!"
  4. On failure: print error, exit with code 1

---

## Phase 4: Write Tests

### Step 4.1: Write #[test] — Single machine reaches goal
- `#[test] fn test_counter_reaches_goal()`:
  - Call `run_machine()`
  - Assert result is `Ok(ticks)` where ticks > 0
  - Print ticks taken

### Step 4.2: Write #[test] — Goal reached within expected ticks
- `#[test] fn test_goal_reached_within_max_ticks()`:
  - Call `run_machine()`
  - Assert result is `Ok(ticks)` where ticks >= 11 and ticks <= 11 (counter 0→10 needs 10 increments + 1 check = 11 ticks)

### Step 4.3: Write #[test] — Counter value is correct
- `#[test] fn test_counter_final_value()`:
  - Build machine, run tick loop
  - Assert final counter value equals 10

---

## Phase 5: Write Machine Specification

### Step 5.1: Write machine_spec.md — Overview section
- Introduction: what pall is, what the machine format represents
- High-level structure: StateMachine as root, states with actions/transitions
- Use cases: PLC-like state machines, protocol implementations, etc.

### Step 5.2: Write machine_spec.md — YAML format specification
- Document every field of `StateMachine`: id, initial, states, inputs, signals, timers, variables, constants
- Document `State`: actions (Vec), transitions (Vec)
- Document `Transition`: when (Optional<expression>), do (Vec<statement>), target (String)
- Document `Action`: when (Optional<expression>), do (Vec<statement>)
- Document `Variable`: type (Type), initial (Optional<Value>), output (bool)
- Document `Input`: type (Type), link (Optional<Link>)
- Document `Signal`: type (Type), expr (Expression)
- Document `Timer`: type (Type), when (Optional<Expression>)
- Document `Constant`: type (Type), value (Value), output (bool)
- Document `Type` enum: Bool, U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, String
- Document `Value` variants: Integer (i64 + fmt), Float (f64 + fmt), String (str + fmt)

### Step 5.3: Write machine_spec.md — Expression format
- Literals:
  - Integers: decimal (`42`), hex (`0xff`), octal (`0o17`), binary (`0b1010`)
  - Floats: decimal (`3.14`), scientific (`1.5e+2`)
  - Strings: double-quoted (`"hello\n"`), single-quoted (`'hello'`), escape sequences (`\n`, `\t`, `\\`, `\"`, `\'`, `\r`)
- References: variable names (`counter`, `state_name`), output references
- Binary operators with precedence:
  1. `*` `/` `%` (multiplicative)
  2. `+` `-` (additive)
  3. `<` `<=` `>` `>=` (comparison)
  4. `==` `!=` (equality)
  5. `&` (bitwise AND)
  6. `^` `^^` (bitwise/bitwise XOR)
  7. `|` (bitwise OR)
  8. `&&` (logical AND)
  9. `||` (logical OR)
- Unary operators: `-` (negate), `!` (logical NOT), `~` (bitwise NOT)
- Parentheses for grouping: `(a + b) * c`

### Step 5.4: Write machine_spec.md — Statement format
- Syntax: `target operator expression`
- Example: `counter += 1`, `result = a + b`
- Assignment operators:
  - `=` (assign), `+=` (add assign), `-=`, `*=`, `/=`, `%=`
  - `&=` (bitwise AND assign), `|=`, `^=`, `^^=`
  - `&&=` (logical AND assign), `||=`, `^^=`
- Target must reference a variable in the machine

### Step 5.5: Write machine_spec.md — Link format
- Syntax: `source_id.output_name`
- Used in `Input` → `link` field
- Example: `link: "sensor_module.temperature"`
- Links propagate values from one machine's output to another machine's input
- Processed before per-machine tick in group mode

### Step 5.6: Write machine_spec.md — Functional semantics
- **Tick**: Deterministic, synchronous, single-threaded state machine step
  - Takes a `Persistent` (current state) and `TickInfo` (timing)
  - Returns an `Update` (changes to apply)
  - State transitions cause immediate return with Update
- **Init**: Creates initial `Persistent`
  - Sets `state` to the machine's initial state
  - Sets `state_name` to the initial state's string name
  - Initializes variables to their default/initial values
  - Inputs set to `default()`, signals to `default()`, timers to 0
- **State evaluation order**: States are visited via match on current state
- **Actions**: Executed before transitions in each state; conditionally based on `when` clause
- **Transitions**: Evaluated in declaration order; first matching transition wins; sets `state` in Update and returns immediately
- **Signals**: Computed after all state/transition logic; assigned to Update
- **Timers**: Accumulate `delta_ms` when `when` condition is true; reset to 0 otherwise
- **Group tick**: Multi-machine coordination
  - Phase 1: Propagate links from outputs to inputs
  - Phase 2: Call tick for each machine
- **Update application**: Consumer must merge Update into Persistent (overwrite Some values, keep others)
- No parallelism, no async, no error recovery — failures propagate as `TickError`

---

## Phase 6: Verification

### Step 6.1: Verify parent crate compiles
- Run `cargo check` in `pall/` root
- Verify no errors or new warnings (existing warnings are acceptable)

### Step 6.2: Verify existing tests still pass
- Run `cargo test` in `pall/` root
- Assert all 43 original tests pass

### Step 6.3: Verify creator compiles
- Run `cargo check --bin creator`
- Verify no errors

### Step 6.4: Run creator
- Run `cargo run --bin creator`
- Verify: generated files appear in `src/bin/runner/generated/`
- Verify: expected files are created (counter_test_types.rs, counter_test_tick.rs, group.rs, mod.rs)

### Step 6.5: Verify runner compiles
- Run `cargo check --bin runner`
- Verify no errors (runner includes generated code, so it depends on creator having run first)

### Step 6.6: Run runner as binary
- Run `cargo run --bin runner`
- Verify: tick loop runs, goal state is reached, output shows success
- Verify: prints ticks taken and final counter value

### Step 6.7: Run runner tests
- Run `cargo test --bin runner`
- Assert all tests pass

### Step 6.8: Verify machine_spec.md is complete
- Read through the document
- Verify all sections from Phase 5 are present, accurate, and well-structured
- Check that examples in the spec match the actual behavior of the compiler
