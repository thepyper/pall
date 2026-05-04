# INTERVIEW: Compiler Module with Rust Backend

## Objective

Create a `compiler` module that compiles a **set of StateMachines** into a Rust module (a set of files). The generated module is integrated by the user into their application. The first backend is Rust. The core generated functionality is a `tick` function that drives state machine execution with proper ordering: links → per-machine tick (state evaluation → actions → transitions → signals).

---

## Architecture Overview

```
compiler/
  mod.rs               — Compiler struct, Backend trait, FileSet, CompileError, TickError
  validation.rs        — Validation phase (collect errors, validate machines)
  backend/
    mod.rs             — Backend registration
    rust/
      mod.rs           — RustBackend implementation
      templates/
        mod.hbs        — Handlebars template for mod.rs
        types.hbs      — Handlebars template for types.rs (Persistent, Update)
        tick.hbs       — Handlebars template for tick.rs (tick + init functions)
```

---

## Design Decisions

### 1. Compilation Input

- Input: `Vec<StateMachine>`
- Starts simple; directory loading can be added later
- Each StateMachine must have a unique `id`
- Links reference machines by ID; all referenced machines must be in the Vec

### 2. Backend Trait

```rust
pub trait Backend: Sync {
    fn compile(&self, machines: &[StateMachine]) -> Result<FileSet, CompileError>;
}
```

- `FileSet` = `HashMap<String, String>` (filename → file content)
- Returns `CompileError` for any compilation issue

### 3. Error Handling — CompileError

- `message: String` — human-readable description
- `line: Option<usize>` — source location
- `column: Option<usize>` — source location
- `kind: CompileErrorKind` — enum for programmatic matching

CompileErrorKind variants (initial):
- `UnreachableTransition` — transition after always-true
- `MissingStateReference` — transition target state doesn't exist
- `InvalidLink` — link source or target doesn't exist
- `InvalidTimerType` — timer type is not numeric
- `InvalidSignalExpr` — signal expression can't cast to declared type
- `DuplicateMachineId` — two machines with same ID

### 4. Error Handling — TickError

- Runtime error type for the generated `tick` function
- `message: String`
- `kind: TickErrorKind` — extensible enum
- Minimal now; more variants can be added later (e.g., division by zero, unknown state)

### 5. FileSet

- `HashMap<String, String>` — filename keys, content values
- Keys are relative paths within the generated module directory

### 6. Handlebars for Code Generation

- Use the `handlebars` crate (latest version, 5.x)
- Templates embedded via `include_str!` at compile time
- Templates stored in `compiler/backend/rust/templates/`
- Templates handle boilerplate; backend provides data + expression-to-Rust conversions

### 7. Tick Function Signature (Generated)

```rust
pub fn tick(state: &Persistent, tick_info: &TickInfo) -> Result<Update, TickError>
```

- Takes persistent state and tick info
- Returns update with changed values
- Returns Result for error handling

### 8. TickInfo

```rust
pub struct TickInfo {
    pub delta_ms: u64,
}
```

- Only one field: `delta_ms` for timer accumulation
- Inputs will be injected later via exposed methods on Persistent

### 9. Persistent Type (Per Machine)

Holds all persistent state for one machine:

```rust
pub struct Persistent {
    pub state: String,                           // current state name
    pub <input_name>: <type>,                    // each input
    pub <variable_name>: <type>,                 // each variable
    pub <signal_name>: <type>,                   // each signal
    pub <timer_name>: <type>,                    // each timer
    pub <constant_name>: <type>,                 // each constant (optional)
}
```

- Fields named directly after the variable/signal/timer/input names
- Rust keyword conflicts handled with `r#` prefix
- All fields derive `Serialize, Deserialize, Debug, Clone`
- Default values: 0 for numeric, `String::new()` for String, `false` for Bool
- `init() -> Persistent` function generated to create initial state

### 10. Update Type (Per Machine)

Contains only values that can change during a tick:

```rust
pub struct Update {
    pub <variable_name>: Option<<type>>,         // None = unchanged
    pub <signal_name>: Option<<type>>,           // None = unchanged
    pub <timer_name>: Option<<type>>,            // None = unchanged
    pub state: Option<String>,                   // None = no state change
}
```

- Fields are `Option<T>` — `None` means no change this tick
- Used only internally (not exposed to user)

### 11. Group Types (For Multiple Machines)

- `GroupPersistent` — holds all machine `Persistent` structs (one per machine)
- `GroupUpdate` — holds all machine `Update` structs (one per machine)
- Group tick function: propagates links first, then ticks each machine
- Machine modules: one file per machine in the generated directory
- Generated `mod.rs` re-exports all machine types and group types

### 12. Group Tick Flow

1. **Link propagation phase**: for each link, read source output → write to target input
2. **Per-machine tick phase**: iterate machines in Vec order, call each machine's `tick()`
3. **Collect updates**: gather all machine Updates into GroupUpdate

### 13. Per-Machine Tick Flow

1. **Evaluate state**: switch on `state.state` to determine which state's code to execute
2. **Execute actions**: for each action in the current state:
   - If `when` is Some: evaluate condition; if truthy (cast to bool), execute `do` statements
   - If `when` is None: always execute `do` statements
   - Actions execute in YAML order
3. **Execute transitions**: for each transition in order:
   - If `when` is Some: evaluate condition; if truthy (cast to bool):
     - Execute `do` statements
     - Set `update.state` to transition target
     - Return immediately (only one transition fires)
   - If `when` is None: always-true transition; same as above
4. **Calculate signals**: evaluate each signal's expression, store in Update
5. **Return Update**: with all changes since the last tick

### 14. Outputs Refactoring (Machine Module)

**Before:** `Output` struct with `type: Type`, stored in `outputs` HashMap

**After:**
- `Output` struct is **removed**
- `outputs` HashMap is **removed** from `StateMachine`
- `output: bool` flag added to:
  - `Input` (in `connections.rs`)
  - `Variable` (in `variables.rs`)
  - `Signal` (in `variables.rs`)
  - `Constant` (in `variables.rs`)
- Flag indicates the variable has an external view (can be linked to as a source)
- Timer does NOT get this flag

### 15. Signal Refactoring

- Rename `when` field to `expr` in `Signal` struct
- `Signal.expr` is the expression that produces the signal's value at each tick
- The signal's `type` field declares the expected output type
- The expression should be able to cast to the declared type

### 16. Timer Behavior (PLC TON-like)

- Timer value = elapsed milliseconds
- When `when` condition is true: timer adds `delta_ms` to its value
- When `when` condition is false: timer resets to 0
- Timer type must be numeric (integer or float)
  - Warning if float type (not integer)
  - Error if not numeric (Bool, String)
- No threshold — timer is just a value accumulator
- Can be used in expressions: `timer.my_timer > 5000`

### 17. Expression Evaluation in Generated Code

- Expressions are converted **directly to Rust code strings** during compilation
- `Reference("my_var")` → `state.my_var`
- `Binary(a, Add, b)` → `a + b` (with type coercion as needed)
- `Value(Integer(42))` → `42i64`
- Type casting rules (which types can cast to which) TBD
- Constants are referenced as Rust `const` variables in generated code

### 18. Constants

- Constants have `type: Type` and `value: Value`
- Generated as Rust `const` variables with the constant's name as identifier
- Not stored in Persistent — embedded as compile-time constants
- Used in expression code generation (referenced by name)

### 19. Action Execution Details

- Actions execute in YAML-defined order within the current state
- `when: Option<FullExpression>` — condition to guard execution
  - `Some(expr)`: evaluate expr, cast to bool (C/C++ style: 0 = false, non-zero = true)
  - `None`: always execute
- `do: Vec<FullStatement>` — assignments to execute
- Actions execute **before** transitions (they can modify variables that transitions read)

### 20. Transition Execution Details

- Transitions execute in YAML-defined order
- **First** transition with a true condition wins — state changes and tick returns
- `when: Option<FullExpression>` — condition
  - `None` = always-true transition (can be the last resort)
  - `Some(expr)`: evaluate, cast to bool
- `do: Vec<FullStatement>` — assignments before state change
- `target: String` — state name to transition to
- Can reference variables modified by actions in the same tick

### 21. "Evaluate State"

- Means: a `match` statement on `state.state` (the current state name string)
- Each state has its own block of action execution + transition evaluation
- If state name doesn't match any known state → TickError

### 22. Link Resolution

- `Link` struct unchanged: `id: String` (source machine), `output: String` (source output name)
- Link's `output` name refers to a variable flagged with `output: true`
- During link propagation phase, the compiler resolves each link to (machine_id, var_name) pairs
- Generated code reads the source value and writes it to the target input

### 23. Validation Phase

- Runs **before** code generation
- Collects **all** errors (doesn't stop at first)
- Validates:
  - Duplicate machine IDs
  - Unreachable transitions (after always-true)
  - Missing state references (transition targets)
  - Link references (source and target exist)
  - Timer types (must be numeric)
  - Signal expression type compatibility
- Returns combined error list if any issues found

### 24. Generated Module Structure

For N machines, generates:
```
generated_module/
  mod.rs              — re-exports all machine types + group types
  machine_a_types.rs  — Persistent + Update for machine A
  machine_a_tick.rs   — tick() + init() for machine A
  machine_b_types.rs  — Persistent + Update for machine B
  machine_b_tick.rs   — tick() + init() for machine B
  group.rs            — GroupPersistent, GroupUpdate, group_tick()
```

Each machine module `pub use`s its own types.

### 25. Default Values for Persistent Fields

- Numeric types (I8-I64, U8-U64): `0`
- Float types (F32, F64): `0.0`
- Bool: `false`
- String: `String::new()`
- If a variable has `initial: Some(Value)`, use that value instead of default

### 26. Rust Keyword Handling

- When a variable/signal/constant name conflicts with a Rust keyword, use `r#` prefix
- Example: a variable named `type` → field name `r#type: Type`

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/machine/connections.rs` | Add `output: bool` to Input; remove Output struct |
| `src/machine/variables.rs` | Add `output: bool` to Variable, Signal, Constant; rename Signal.when → Signal.expr |
| `src/machine/mod.rs` | Remove outputs HashMap from StateMachine; update re-exports |
| `src/machine/parser/` | Update tests if needed for renamed fields |
| `Cargo.toml` | Add `handlebars` dependency |
| **New files** | See compiler module structure above |

---

## Not Changed

- Expression enum structure
- Statement structure
- Action/Transition/State structures (unchanged — still use FullExpression, FullStatement)
- Link struct (unchanged — same `id.output` format)
- Timer struct (no `output` flag; `when` stays as-is)
- AssignmentOperator enum
- BinaryOperator / UnaryOperator enums
- Type enum (values)
- Value enum structure
- Parser module structure (grammar.pest, parser/*.rs)
- FullExpression / FullStatement structures
