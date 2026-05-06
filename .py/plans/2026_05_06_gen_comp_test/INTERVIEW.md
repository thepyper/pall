# INTERVIEW: Generate Code, Build Micro-Runtime, and Test End-to-End

## Objective

Create an end-to-end stress test for the current pall compiler implementation by:
1. Restructuring the crate to support library + binary + tests
2. Building a micro-runtime binary that generates code from a state machine and executes it
3. Testing that generated code compiles and that the machine reaches a known goal state
4. Writing a specification document (`machine_spec.md`) for the machine YAML format

---

## Design Decisions

### 1. Crate Structure: Hybrid (Library + Binaries)

- **Keep the library as the main output** — the compiler + machine types remain the crate's primary purpose
- **Keep `src/main.rs` as-is** (the current demo binary that prints generated code)
- **Add a new binary** in `src/bin/micro_runtime.rs` — the micro-runtime that generates, includes, and runs test code
- **Add `src/bin/` directory** — a normal binary (not `examples/`), so it can have multiple files
- **Do NOT convert to `lib` only** — keep `bin` as the output; the library is internal

Rationale: Rust examples must be single-file. A multi-file test harness requires a proper binary crate.

### 2. Two-Binary Architecture

Two separate binaries for clean separation of concerns:

#### `src/bin/creator/` — Code Generator

```
src/bin/creator/
├── Cargo.toml
└── src/main.rs
```

**Responsibility:** Creates the machine definition and compiles it to Rust source code.

**Flow:**
1. Builds `StateMachine` programmatically
2. Calls `Compiler.compile(&[machine])` → `FileSet`
3. Writes generated files to `src/bin/runner/generated/` directory
4. Prints which files were written and their paths

#### `src/bin/runner/` — Micro-Runtime

```
src/bin/runner/
├── Cargo.toml
└── src/
    ├── main.rs     # includes generated code, runs tick loop, runs tests
    └── lib.rs      # stubs (TickInfo, TickError) + include! macros
```

**Responsibility:** Includes pre-generated code and executes it.

**Flow:**
1. `include!()`s the generated files from `generated/` directory
2. Provides stubs (`TickInfo`, `TickError`) at the right module level
3. Runs `tick()` in a loop until goal state or max 100 ticks, asserts result
4. `#[test]` functions verify compilation and correctness

### 3. Generated Code Integration

- Generated code is **included via `include!()`** at compile time into `lib.rs`
- The `lib.rs` provides **minimal stubs** so generated imports resolve:
  - `error::TickError` — a minimal error type
  - `TickInfo` — struct with `delta_ms: u64`
- Generated module structure mirrors what the compiler outputs:
  - `{id}_types` module (Persistent, Update, State)
  - `{id}_tick` module (tick, init)
  - `group` module (group_tick, GroupPersistent, GroupUpdate)

### 4. Example Machine Design

**One standalone machine for now:**

```
initial → counting → goal
```

- **initial state**: counter starts at 0, transitions to "counting" immediately (always-true fallback or condition)
- **counting state**: counter increments by 1 each tick (0 → 1 → ... → 10), transitions to "goal" when counter >= 10
- **goal state**: final state, machine stops

Expected behavior:
- Machine reaches "goal" state in ~11 ticks
- Test asserts goal state is reached and counter equals expected value

Machine construction in Rust (programmatic, no YAML for now):
```rust
StateMachine {
    id: "counter_test".to_string(),
    initial: Some("initial".to_string()),
    variables: {
        "counter": Variable {
            type: U32,
            initial: Some(Value::Integer(IntegerValue { value: 0, fmt: IntegerFmt::Dec })),
            output: false,
        }
    },
    states: {
        "initial": {
            transitions: [
                { when: None, do: [], target: "counting" },  // always
            ],
        },
        "counting": {
            transitions: [
                { when: FullExpression::parse("counter >= 10"), do: [], target: "goal" },
            ],
            actions: [
                { when: None, do: [FullStatement::parse("counter += 1")], },  // increment every tick
            ],
        },
        "goal": {
            transitions: [],  // no outgoing transitions
        },
    },
}
```

### 5. Runtime Behavior

- **Call `tick()` up to 100 times** (configurable max ticks)
- **Stop early** when machine reaches the goal state
- **Goal detection**: check `state.as_str() == "goal"` (string comparison)
- **Assertion**: goal state is reached, counter variable has expected value
- **Time**: use `chrono` crate to generate realistic `delta_ms` values (or fixed 1000ms per tick for simplicity)

### 6. delta_ms Handling

- The generated `tick()` function takes `tick_info: &TickInfo` with `delta_ms: u64`
- The micro-runtime creates `TickInfo` each tick
- Two options:
  - **Fixed**: all ticks use `delta_ms = 1000`
  - **Dynamic**: use `chrono::Utc::now()` to compute actual elapsed time
- Decision: start with **fixed `delta_ms = 1000`** (simpler, deterministic for testing)

### 7. Testing Approach

- **Integration tests via a binary crate**: `src/bin/micro_runtime/`
- Tests are `#[test]` functions compiled with `cargo test` on the binary
- Each test:
  1. Builds a `StateMachine` programmatically
  2. Calls `Compiler.compile()` → `FileSet`
  3. Writes generated code to temp dir
  4. Includes code via `include!()` into the lib module
  5. Runs the tick loop, asserts goal reached

### 8. Goal State Requirement

- **For tests only**: every test machine MUST have a "goal" state
- This is a **test requirement**, not a machine requirement
- The test checks `state.as_str() == "goal"` to verify the machine completed successfully

### 9. Machine Specification Document (`machine_spec.md`)

A complete specification covering:
- **YAML format**: full structure of state machines in YAML (StateMachine, State, Transition, Action, Variable, Input, etc.)
- **Expression format**: expression grammar (literals, references, binary/unary operators, precedence)
- **Statement format**: assignment syntax (target, operator, expression)
- **Link format**: `source.output` syntax for input linking
- **Functional semantics**: tick behavior, state transitions, signal/timer accumulation, init behavior

### 10. Dependencies

- **chrono** — to add for realistic time handling in the micro-runtime
- Current dependencies remain (serde, serde_yaml, pest, pest_derive, handlebars, serde_json)

---

## File Locations

| File/Directory | Action |
|----------------|--------|
| `src/bin/creator/` | **New** — code generator binary |
| `src/bin/runner/src/lib.rs` | **New** — stubs + `include!` macros for generated code |
| `src/bin/runner/src/main.rs` | **New** — includes generated code, runs tick loop, runs tests |
| `machine_spec.md` | **New** — complete machine YAML specification |
| `Cargo.toml` | **Modify** — add `chrono` dependency |
| `src/main.rs` | **Keep as-is** (current demo binary) |

---

## Not Changed

- Compiler core (`compiler/` module) — no changes to validation, codegen, or backend
- Machine types (`machine/` module) — no changes to types, parser, or AST
- Existing 43 tests — preserved and should still pass
- `main.rs` demo binary — kept as the existing code-printing example
- Template files (`templates/`) — no changes
