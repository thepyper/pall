# PLAN: Compiler Module with Rust Backend

## Phase 0: Machine Module Refactoring (Pre-requisite)

> These changes must be done first — they restructure the machine module before the compiler is built.

### Step 0.1: Add `output: bool` to Input struct
- **File:** `src/machine/connections.rs`
- **Action:** Add `pub output: bool` field to `Input` struct with `#[serde(default)]`
- **Detail:** Default `false` via `#[serde(default)]`. This flag marks the input as having an external view.

### Step 0.2: Add `output: bool` to Variable struct
- **File:** `src/machine/variables.rs`
- **Action:** Add `pub output: bool` field to `Variable` struct with `#[serde(default)]`
- **Detail:** Default `false`. Variables can be exposed externally if this is set.

### Step 0.3: Add `output: bool` and rename `when` to `expr` in Signal struct
- **File:** `src/machine/variables.rs`
- **Action:** Add `pub output: bool` field to `Signal` struct with `#[serde(default)]`; rename `when: Expression` to `expr: Expression`
- **Detail:** The signal's expression produces its value at each tick. The `expr` name is more accurate.

### Step 0.4: Add `output: bool` to Constant struct
- **File:** `src/machine/variables.rs`
- **Action:** Add `pub output: bool` field to `Constant` struct with `#[serde(default)]`
- **Detail:** Default `false`. Constants can be exposed externally (rare but possible).

### Step 0.5: Remove Output struct
- **File:** `src/machine/connections.rs`
- **Action:** Delete the entire `Output` struct definition
- **Detail:** Outputs are no longer a separate type — they're flagged variables.

### Step 0.6: Remove `outputs` HashMap from StateMachine
- **File:** `src/machine/mod.rs`
- **Action:** Remove `#[serde(default)] pub outputs: HashMap<String, Output>` from `StateMachine`
- **Detail:** The outputs concept is now handled by the `output: bool` flag on inputs/variables/constants/signals.

### Step 0.7: Update re-exports in machine/mod.rs
- **File:** `src/machine/mod.rs`
- **Action:** Remove `Output` from `pub use connections::` line
- **Detail:** No more `Output` to export.

### Step 0.8: Update test.rs for new machine structure
- **File:** `src/machine/test.rs`
- **Action:** Update all test YAML strings to remove `outputs` block and use new `output` flags
- **Detail:** Tests that reference `outputs` HashMap or `Output` struct need to be updated to the new structure.

### Step 0.9: Run cargo test to verify refactoring
- **Action:** `cargo test`
- **Detail:** Ensure all machine module tests pass with the new structure. No regressions.

---

## Phase 1: Compiler Module Skeleton

### Step 1.1: Create compiler/mod.rs with module declarations
- **File:** `src/compiler/mod.rs` (new)
- **Action:** Create module with declarations for `validation`, `backend`, `codegen`, `error`
- **Detail:** Skeleton file that declares sub-modules. Re-exports key types from sub-modules.

### Step 1.2: Create error.rs with CompileError type
- **File:** `src/compiler/error.rs` (new)
- **Action:** Define `CompileError` struct with `message: String`, `line: Option<usize>`, `column: Option<usize>`, `kind: CompileErrorKind`
- **Detail:** Implement `Display`, `std::error::Error`, `Debug` for CompileError. Include a `new()` constructor.

### Step 1.3: Define CompileErrorKind enum
- **File:** `src/compiler/error.rs`
- **Action:** Define `pub enum CompileErrorKind` with variants: `DuplicateMachineId`, `UnreachableTransition`, `MissingStateReference`, `InvalidLink`, `InvalidTimerType`, `InvalidSignalExpr`
- **Detail:** Each variant carries a descriptive label. New variants can be added as needed.

### Step 1.4: Create TickError type
- **File:** `src/compiler/error.rs`
- **Action:** Define `TickError` struct with `message: String`, `kind: TickErrorKind`
- **Detail:** `TickErrorKind` is an empty/extensible enum for now. TickError will be emitted by generated code.

### Step 1.5: Define TickErrorKind enum
- **File:** `src/compiler/error.rs`
- **Action:** Define `pub enum TickErrorKind` with minimal variant (e.g., `UnknownState`)
- **Detail:** Extensible — more variants added as runtime error scenarios are discovered.

### Step 1.6: Define FileSet type
- **File:** `src/compiler/mod.rs` or `src/compiler/error.rs`
- **Action:** Define `pub type FileSet = std::collections::HashMap<String, String>`
- **Detail:** Maps filename (key) to file content (value). Keys are relative paths.

### Step 1.7: Define Backend trait
- **File:** `src/compiler/mod.rs`
- **Action:** Define `pub trait Backend: Sync { fn compile(&self, machines: &[StateMachine]) -> Result<FileSet, CompileError>; }`
- **Detail:** The `Sync` bound allows the trait to be used in multi-threaded contexts.

### Step 1.8: Define TickInfo struct
- **File:** `src/compiler/mod.rs`
- **Action:** Define `pub struct TickInfo { pub delta_ms: u64 }`
- **Detail:** Simple struct with only `delta_ms`. Will be embedded in generated code.

### Step 1.9: Wire compiler module into main.rs
- **File:** `src/main.rs`
- **Action:** Add `mod compiler;` before `fn main()`
- **Detail:** Make the compiler module accessible from the crate root.

---

## Phase 2: Validation Module

### Step 2.1: Create validation.rs module skeleton
- **File:** `src/compiler/validation.rs` (new)
- **Action:** Create module with `pub fn validate_machines(machines: &[StateMachine]) -> Result<(), Vec<CompileError>>`
- **Detail:** Public API — takes a slice of StateMachines, returns a list of errors (empty = success).

### Step 2.2: Implement duplicate machine ID validation
- **File:** `src/compiler/validation.rs`
- **Action:** Collect all machine IDs; if any duplicate exists, emit `CompileErrorKind::DuplicateMachineId`
- **Detail:** Use a HashSet to track seen IDs. Report the first duplicate found.

### Step 2.3: Implement unreachable transition detection
- **File:** `src/compiler/validation.rs`
- **Action:** For each state in each machine, check if any transition has `when: None` (always-true); if so, flag all transitions after it as unreachable with `CompileErrorKind::UnreachableTransition`
- **Detail:** An always-true transition (`when: None`) causes the tick to return immediately, making subsequent transitions dead code.

### Step 2.4: Implement missing state reference validation
- **File:** `src/compiler/validation.rs`
- **Action:** For each transition, check that its `target` state exists in the machine's `states` HashMap
- **Detail:** Emit `CompileErrorKind::MissingStateReference` with the machine ID and target name if the state is missing.

### Step 2.5: Implement link reference validation
- **File:** `src/compiler/validation.rs`
- **Action:** For each input with a `link`, verify:
  1. The source machine (link.id) exists in the machine set
  2. The source variable (link.output) exists and has `output: true` on the source machine
  3. The target input exists on the current machine
- **Detail:** Emit `CompileErrorKind::InvalidLink` with details about what's missing.

### Step 2.6: Implement timer type validation
- **File:** `src/compiler/validation.rs`
- **Action:** For each timer, check that its `type` is numeric (U8-U64, I8-I64, F32, F64). Emit error if Bool or String.
- **Detail:** Numeric check: any `Type` variant except `Bool` and `String`. For float types (F32/F64), emit a warning (not error) since they work but aren't integer counters.

### Step 2.7: Implement signal expression type validation
- **File:** `src/compiler/validation.rs`
- **Action:** For each signal, validate that its `expr` expression is compatible with the signal's declared `type`
- **Detail:** For now, basic check: the expression must produce a value (not be a Reference to a type-incompatible variable). Detailed casting rules can be added later.

### Step 2.8: Collect all errors (don't stop at first)
- **File:** `src/compiler/validation.rs`
- **Action:** Change return type to `Result<(), Vec<CompileError>>`. Each validation function appends to a shared error list.
- **Detail:** All validation functions run regardless of errors. If errors exist, return `Err(errors)`.

### Step 2.9: Call validation from Backend::compile()
- **File:** `src/compiler/mod.rs`
- **Action:** In the Backend trait's compile() implementation (or in a Compiler wrapper), call validation first. Return validation errors if any.
- **Detail:** Validation is the first step of compilation. Code generation only proceeds if validation succeeds.

---

## Phase 3: Handlebars Setup

### Step 3.1: Add handlebars dependency to Cargo.toml
- **File:** `Cargo.toml`
- **Action:** Add `handlebars = "5"` to `[dependencies]`
- **Detail:** Version 5.x is the latest major version.

### Step 3.2: Create templates directory
- **Action:** Create `src/compiler/backend/rust/templates/` directory
- **Detail:** Templates directory for Handlebars template files.

### Step 3.3: Create mod.hbs template
- **File:** `src/compiler/backend/rust/templates/mod.hbs` (new)
- **Action:** Create Handlebars template for the generated mod.rs file
- **Detail:** Template iterates over machine names, declares each module, and re-exports Persistent/Update/tick/init from each. Also declares the group module.

### Step 3.4: Create types.hbs template
- **File:** `src/compiler/backend/rust/templates/types.hbs` (new)
- **Action:** Create Handlebars template for the generated types.rs file
- **Detail:** Template generates:
  - Imports (serde, TickInfo, etc.)
  - `const` declarations for each constant
  - `Persistent` struct with fields for state, inputs, variables, signals, timers
  - `Update` struct with `Option<T>` fields for variables, signals, timers, and state
- **Data context expected:** machine object with fields: name, initial, constants, inputs, variables, signals, timers, states

### Step 3.5: Create tick.hbs template
- **File:** `src/compiler/backend/rust/templates/tick.hbs` (new)
- **Action:** Create Handlebars template for the generated tick.rs file
- **Detail:** Template generates:
  - Imports
  - `tick()` function with match on state, action execution, transition execution, signal calculation
  - `init()` function that initializes Persistent with default/initial values
- **Data context expected:** machine object with all state/action/transition/signal data, plus pre-rendered Rust code strings for expressions and statements

### Step 3.6: Add include_str! to embed templates
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Use `const MOD_TEMPLATE: &str = include_str!("templates/mod.hbs");` etc.
- **Detail:** Templates are loaded as string constants at compile time.

### Step 3.7: Set up Handlebars registration in RustBackend
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Create a `Handlebars` instance and register the template strings
- **Detail:** The backend creates a Handlebars renderer, adds templates, and can render them with data context.

---

## Phase 4: Rust Backend Implementation

### Step 4.1: Create backend/mod.rs
- **File:** `src/compiler/backend/mod.rs` (new)
- **Action:** Declare `pub mod rust;` and optionally re-export the RustBackend
- **Detail:** Simple module that makes the Rust backend accessible.

### Step 4.2: Create RustBackend struct
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Define `pub struct RustBackend` (empty or with minimal config)
- **Detail:** Implements the `Backend` trait.

### Step 4.3: Implement RustBackend::compile() skeleton
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Implement `compile(&self, machines: &[StateMachine]) -> Result<FileSet, CompileError>` that calls validation, then generates code
- **Detail:** Skeleton that returns `Ok(FileSet::new())` initially; flesh out in subsequent steps.

### Step 4.4: Generate Persistent struct fields (code gen helper)
- **File:** `src/compiler/codegen.rs` (new) or `src/compiler/backend/rust/mod.rs`
- **Action:** Write helper function that takes a Vec of Variable/Signal/Timer/Input definitions and produces Rust field definitions
- **Detail:** Maps Type enum to Rust types: U8→u8, I64→i64, F64→f64, String→String, Bool→bool. Handles r# prefix for keyword conflicts.

### Step 4.5: Generate Update struct fields (code gen helper)
- **File:** `src/compiler/codegen.rs`
- **Action:** Write helper function that produces Rust fields for Update (Option<T> type)
- **Detail:** Same type mapping as Persistent, but each field is `Option<T>`. Excludes inputs (they don't change via tick).

### Step 4.6: Implement expression-to-Rust-code conversion
- **File:** `src/compiler/codegen.rs`
- **Action:** Write `pub fn expr_to_rust(expr: &Expression, persistent: &Persistent) -> Result<String, CompileError>`
- **Detail:** Converts Expression AST to a Rust code string:
  - `Value(Integer(v))` → `v as rust_type`
  - `Value(Float(v))` → `v as f64`
  - `Value(String(s))` → `s.to_string()`
  - `Reference(target)` → `state.<field_name>`
  - `Binary(a, op, b)` → `rust_expr(a) <operator> rust_expr(b)`
  - `Unary(op, inner)` → `<operator> rust_expr(inner)`
  - `Parenthesis(inner)` → `(<rust_expr>)`

### Step 4.7: Implement statement-to-Rust-code conversion
- **File:** `src/compiler/codegen.rs`
- **Action:** Write `pub fn stmt_to_rust(stmt: &FullStatement, persistent: &Persistent) -> Result<String, CompileError>`
- **Detail:** Converts `target = expr` to `update.target = Some(<rust_expr>)`. Handles assignment operators:
  - `Assign` → `update.target = Some(<expr>)`
  - `AddAssign` → `update.target = Some(<current> + <expr>)`
  - Other operators follow similar pattern

### Step 4.8: Generate tick function body for each machine
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** For each machine, render tick.hbs template with data including:
  - State match arms, each with pre-rendered action and transition code
  - Signal expression code (pre-rendered via expr_to_rust)
- **Detail:** The template receives ready-to-insert Rust code strings, not raw AST nodes.

### Step 4.9: Generate init function body for each machine
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Generate init() that sets state to initial state name and initializes all fields with defaults or `initial` values
- **Detail:** Default values: 0 for numeric, 0.0 for float, false for bool, "" for string. Use variable's `initial` value if present.

### Step 4.10: Generate const declarations for constants
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** For each constant in the machine, generate `pub const <name>: <type> = <value>;`
- **Detail:** Value is the constant's `Value` converted to a Rust literal (e.g., `42u32`, `3.14f64`).

### Step 4.11: Generate machine-specific module files
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** For each machine, generate `types.rs` and `tick.rs` files with proper module-level code
- **Detail:** Each file is a self-contained Rust module. Add `#[allow(unused)]` or other attributes as needed.

### Step 4.12: Generate the mod.rs for the compiled module
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Generate mod.rs that declares all machine modules and group module, with pub use re-exports
- **Detail:** Template-driven. Each machine exposes Persistent, Update, tick, init.

---

## Phase 5: Group Types Generation

### Step 5.1: Implement GroupPersistent struct generation
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Generate `GroupPersistent` struct holding one field per machine's Persistent
- **Detail:** Fields named after machine IDs: `pub <machine_id>_state: <machine_id>_types::Persistent`

### Step 5.2: Implement GroupUpdate struct generation
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Generate `GroupUpdate` struct holding one field per machine's Update
- **Detail:** Fields named after machine IDs: `pub <machine_id>: <machine_id>_tick::Update`

### Step 5.3: Implement group tick function generation
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Generate `group_tick(group: &GroupPersistent, tick_info: &TickInfo) -> Result<GroupUpdate, TickError>`
- **Detail:** The function orchestrates link propagation and per-machine ticks.

### Step 5.4: Implement link propagation code generation
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** For each link, generate code that reads source output → writes to target input
- **Detail:** Pattern: `group.<source_machine>.<output_var> = ...` → `group.<target_machine>.<input_var> = ...`. Source is read from the group Persistent before any machine ticks.

### Step 5.5: Wire group generation into compile()
- **File:** `src/compiler/backend/rust/mod.rs`
- **Action:** Add group type generation to the compile() method, add files to FileSet
- **Detail:** Files: `mod.rs`, `<machine_id>_types.rs`, `<machine_id>_tick.rs`, `group.rs`

---

## Phase 6: Integration & Testing

### Step 6.1: Update main.rs to demonstrate compiler usage
- **File:** `src/main.rs`
- **Action:** Create a sample StateMachine, pass it to the compiler, print the generated files
- **Detail:** Use a simple inline StateMachine or load from a string. Print each file's content.

### Step 6.2: Run cargo build to verify compilation
- **Action:** `cargo build`
- **Detail:** Ensure the compiler crate compiles without errors.

### Step 6.3: Test validation errors
- **Action:** Create test machines with invalid configurations (duplicate IDs, unreachable transitions, bad links) and verify they produce CompileError
- **Detail:** Unit tests in `src/compiler/validation.rs`.

### Step 6.4: Test successful compilation of a simple machine
- **Action:** Compile a minimal machine and verify generated code compiles
- **Detail:** Write generated files to a temp directory, try to compile them as a standalone Rust crate.

### Step 6.5: Test multi-machine compilation with links
- **Action:** Compile two machines connected by a link and verify the group tick code is generated correctly
- **Detail:** Check that link propagation code exists in the group.rs output.
