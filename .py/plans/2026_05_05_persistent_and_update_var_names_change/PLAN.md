# Plan: Persistent and Update Variable Names Change

## Problem

Variable names `state` and `update` for the Persistent and Update structs in generated Rust code are hardcoded in templates (`tick.hbs`, `group.hbs`) and in codegen functions (`expr_to_rust`, `stmt_to_rust`). This produces verbose generated code and makes it hard to change these names.

## Objective

Introduce 4 compile-time constants in `RustBackend` that control the variable names used in generated code:
- `STATE_NAME` = `"x"` (Persistent parameter in per-machine tick)
- `UPDATE_NAME` = `"y"` (Update local variable in per-machine tick)
- `STATE_GROUP_NAME` = `"xs"` (Persistent parameter in group tick)
- `UPDATE_GROUP_NAME` = `"ys"` (Update local variable in group tick)

Struct names (`Persistent`, `Update`, `GroupPersistent`, `GroupUpdate`) remain unchanged.

## Files Changed

- `src/compiler/codegen.rs`
- `src/compiler/backend/rust/mod.rs`
- `src/compiler/backend/rust/templates/tick.hbs`
- `src/compiler/backend/rust/templates/group.hbs`

---

## Plan Steps

### Step 1: Add `CodegenContext` struct to codegen.rs

**File:** `src/compiler/codegen.rs`

Add a new struct at the top of the file (after the imports):

```rust
/// Context passed to code generation helpers for variable naming.
pub struct CodegenContext {
    /// Variable name for the Persistent struct parameter (e.g. "x").
    pub state_var: String,
    /// Variable name for the Update struct local (e.g. "y").
    pub update_var: String,
}

impl CodegenContext {
    pub fn new(state_var: &str, update_var: &str) -> Self {
        Self {
            state_var: state_var.to_string(),
            update_var: update_var.to_string(),
        }
    }
}
```

---

### Step 2: Add 4 compile-time constants to RustBackend

**File:** `src/compiler/backend/rust/mod.rs`

Add constants to the `RustBackend` struct:

```rust
/// Rust backend: compiles state machines into Rust code using Handlebars templates.
pub struct RustBackend {
    handlebars: Handlebars<'static>,
    /// Variable name for the Persistent struct parameter in per-machine tick().
    pub const STATE_NAME: &'static str = "x",
    /// Variable name for the Update struct local in per-machine tick().
    pub const UPDATE_NAME: &'static str = "y",
    /// Variable name for the Persistent struct parameter in group tick().
    pub const STATE_GROUP_NAME: &'static str = "xs",
    /// Variable name for the Update struct local in group tick().
    pub const UPDATE_GROUP_NAME: &'static str = "ys",
}
```

---

### Step 3: Update `build_types_data` — accept context param, add name fields

**File:** `src/compiler/codegen.rs`

Change function signature:
```rust
pub fn build_types_data(
    machine: &crate::machine::StateMachine,
    context: &CodegenContext,
) -> serde_json::Value {
```

Add two fields to the return JSON:
```rust
serde_json::json!({
    // ... existing fields ...
    "state_var": context.state_var.clone(),
    "update_var": context.update_var.clone(),
})
```

---

### Step 4: Update `build_tick_data` — accept context param, add name fields

**File:** `src/compiler/codegen.rs`

Change function signature:
```rust
pub fn build_tick_data(
    machine: &crate::machine::StateMachine,
    context: &CodegenContext,
) -> Result<serde_json::Value, Vec<CompileError>> {
```

Create a context variable at the start (after building `field_list`):
```rust
let context = CodegenContext::new(&context.state_var, &context.update_var);
```

Update the return JSON to include variable names:
```rust
Ok(serde_json::json!({
    // ... existing fields ...
    "state_var": context.state_var.clone(),
    "update_var": context.update_var.clone(),
}))
```

---

### Step 5: Update `build_mod_data` — no changes needed

**File:** `src/compiler/codegen.rs`

No changes. `mod.hbs` does not use variable names (struct names unchanged).

---

### Step 6: Update `build_group_data` — accept context param, add name fields

**File:** `src/compiler/codegen.rs`

Change function signature:
```rust
pub fn build_group_data(
    machines: &[crate::machine::StateMachine],
    context: &CodegenContext,
) -> serde_json::Value {
```

Add fields to return JSON:
```rust
serde_json::json!({
    // ... existing fields ...
    "state_group_var": context.state_var.clone(),
    "update_group_var": context.update_var.clone(),
})
```

---

### Step 7: Update `expr_to_rust` — use context variable name

**File:** `src/compiler/codegen.rs`

Change function signature:
```rust
pub fn expr_to_rust(
    expr: &Expression,
    ctx: &CodegenContext,
) -> Result<String, CompileError>
```

In the `Expression::Reference` arm, change from:
```rust
Ok(format!("state.{ident}"))
```
to:
```rust
Ok(format!("{}.{}", ctx.state_var, ident))
```

---

### Step 8: Update `condition_to_rust` — pass context to expr_to_rust

**File:** `src/compiler/codegen.rs`

Change signature:
```rust
pub fn condition_to_rust(
    expr: &FullExpression,
    ctx: &CodegenContext,
) -> Result<String, CompileError>
```

Change body:
```rust
expr_to_rust(&expr.expression, ctx)
```

---

### Step 9: Update `stmt_to_rust` — use context variable name

**File:** `src/compiler/codegen.rs`

Change signature:
```rust
pub fn stmt_to_rust(
    stmt: &FullStatement,
    ctx: &CodegenContext,
) -> Result<String, CompileError>
```

In `expr_to_rust` call: change `persistent_fields` → `ctx`

In all 13 match arms, change `update.{target}` → `{ctx.update_var}.{target}`:

```rust
crate::machine::AssignmentOperator::Assign => {
    format!("{}.{target} = Some({expr_code});", ctx.update_var)
}
crate::machine::AssignmentOperator::AddAssign => {
    format!("{}.{target} = Some({target} + {expr_code});", ctx.update_var)
}
// ... 11 more arms, same pattern ...
```

---

### Step 10: Update calls inside `build_tick_data` — use context

**File:** `src/compiler/codegen.rs`

After creating the local `context` variable (from `CodegenContext::new`), replace all calls:

- `condition_to_rust(expr, &field_list)` → `condition_to_rust(expr, &context)` (4 occurrences)
- `stmt_to_rust(stmt, &field_list)` → `stmt_to_rust(stmt, &context)` (6 occurrences)
- `expr_to_rust(&sig.expr, &field_list)` → `expr_to_rust(&sig.expr, &context)` (1 occurrence)
- `expr_to_rust(expr, &field_list)` → `expr_to_rust(expr, &context)` (1 occurrence)

---

### Step 11: Update `RustBackend::compile()` — pass constants as context

**File:** `src/compiler/backend/rust/mod.rs`

Add import:
```rust
use super::super::codegen::CodegenContext;
```

In `RustBackend::compile()`, create `CodegenContext` instances before calling each build function:

For per-machine types:
```rust
let types_ctx = CodegenContext::new(Self::STATE_NAME, Self::UPDATE_NAME);
let data = codegen::build_types_data(machine, &types_ctx);
```

For per-machine tick:
```rust
let tick_ctx = CodegenContext::new(Self::STATE_NAME, Self::UPDATE_NAME);
let data = codegen::build_tick_data(machine, &tick_ctx)?;
```

For group:
```rust
let group_ctx = CodegenContext::new(Self::STATE_GROUP_NAME, Self::UPDATE_GROUP_NAME);
let group_data = codegen::build_group_data(machines, &group_ctx);
```

For mod (no context needed):
```rust
let mod_data = codegen::build_mod_data(machines);
```

---

### Step 12: Update `tick.hbs` — function signatures, local variables

**File:** `src/compiler/backend/rust/templates/tick.hbs`

**IMPORTANT:** Type names (`Persistent`, `Update`) remain unchanged. Only variable names change.

1. **Tick function signature** (parameter name only):
   Before: `pub fn tick(state: &Persistent, tick_info: &TickInfo) -> Result<Update, TickError>`
   After: `pub fn tick({{{state_var}}}: &Persistent, tick_info: &TickInfo) -> Result<Update, TickError>`

2. **Update local variable:**
   Before: `let mut update = Update::default();`
   After: `let mut {{{update_var}}} = Update::default();`

3. **Init function comment and struct instantiation:**
   Before:
   ```
   /// Create initial Persistent state for machine: {{{machine_id}}}
   pub fn init() -> Persistent {
       Persistent {
   ```
   After:
   ```
   /// Create initial {{{state_var}}} state for machine: {{{machine_id}}}
   pub fn init() -> Persistent {
       Persistent {
   ```
   (Note: `Persistent` is a type name — stays unchanged. The `init()` function returns type `Persistent`.)

---

### Step 13: Update `tick.hbs` — replace `state.` with `{{x}}.` references

**File:** `src/compiler/backend/rust/templates/tick.hbs`

Replace all occurrences of `state.` (the Persistent parameter access) with `{{{state_var}}}.`:

- `match state.state` → `match {{{state_var}}}.state`
- `state.{{{name}}}: default()` (inputs in init) → `{{{state_var}}}.{{{name}}}: default()`
- `state.{{{name}}}: {{{default_value}}}` (variables in init) → `{{{state_var}}}.{{{name}}}: {{{default_value}}}`
- `state.{{{name}}}: default()` (signals in init) → `{{{state_var}}}.{{{name}}}: default()`
- `state.{{{name}}}: 0 as {{{rust_type}}}` (timers in init) → `{{{state_var}}}.{{{name}}}: 0 as {{{rust_type}}}`
- `state.{{{name}}}` (timer condition: `state.{name} + ...`) → `{{{state_var}}}.{{{name}}}`
- `state.{{{name}}}` (timer no-condition: `state.{name} + ...`) → `{{{state_var}}}.{{{name}}}`

---

### Step 14: Update `tick.hbs` — replace `update.` with `{{y}}.` references

**File:** `src/compiler/backend/rust/templates/tick.hbs`

Replace all occurrences of `update.` with `{{{update_var}}}.`:

- `update.state = Some(State::{{{target_variant}}})` → `{{{update_var}}}.state = Some(State::{{{target_variant}}})`
- `update.{{{name}}} = Some(val)` (signals) → `{{{update_var}}}.{{{name}}} = Some(val)`
- `update.{{{name}}} = Some(state.{{{name}}} + ...)` (timers when) → `{{{update_var}}}.{{{name}}} = Some({{{state_var}}}.{{{name}}} + ...)`
- `update.{{{name}}} = Some(0 as ...)` (timers else) → `{{{update_var}}}.{{{name}}} = Some(0 as ...)`
- `update.{{{name}}} = Some(state.{{{name}}} + ...)` (timers no-condition) → `{{{update_var}}}.{{{name}}} = Some({{{state_var}}}.{{{name}}} + ...)`
  (Note: Signal `state.` references go through `expr_to_rust` (Step 7). Timer accumulation code is in the template, handled here and in Step 13.)
- `Ok(update)` → `Ok({{{update_var}}})` (end of tick function)

---

### Step 15: Update `group.hbs` — import

**File:** `src/compiler/backend/rust/templates/group.hbs`

**IMPORTANT:** Type names (`Persistent`, `Update`) remain unchanged. Only variable names change.

Before: `use {{{id}}}_types::{Persistent, Update};`
After: `use {{{id}}}_types::{Persistent, Update};` (unchanged)

---

### Step 16: Update `group.hbs` — function signature and local variables

**File:** `src/compiler/backend/rust/templates/group.hbs`

**IMPORTANT:** Type names (`GroupPersistent`, `GroupUpdate`, `Update`) remain unchanged.

Function signature (parameter name only):
Before:
```rust
pub fn group_tick(
    group: &GroupPersistent,
    tick_info: &TickInfo,
) -> Result<GroupUpdate, TickError>
```
After:
```rust
pub fn group_tick(
    {{{state_group_var}}}: &GroupPersistent,
    tick_info: &TickInfo,
) -> Result<GroupUpdate, TickError>
```

Local variable:
Before: `let mut update = GroupUpdate {`
After: `let mut {{{update_group_var}}} = GroupUpdate {`

---

### Step 17: Update `group.hbs` — link assignment block

**File:** `src/compiler/backend/rust/templates/group.hbs`

Before:
```rust
        let val = &group.{{{source_machine}}}.{{{source_var}}};
        let mut g = &group.{{{target_machine}}};
        let mut u = &mut update.{{{target_machine}}};
        u.{{{target_var}}} = Some(val.clone());
```

After:
```rust
        let val = &{{{state_group_var}}}.{{{source_machine}}}.{{{source_var}}};
        let mut g = &{{{state_group_var}}}.{{{target_machine}}};
        let mut u = &mut {{{update_group_var}}}.{{{target_machine}}};
        u.{{{target_var}}} = Some(val.clone());
```

---

### Step 18: Update `group.hbs` — per-machine tick block

**File:** `src/compiler/backend/rust/templates/group.hbs`

Before:
```rust
    {
        let state = &group.{{{id}}};
        let result = tick(state, tick_info)?;
        update.{{{id}}} = result;
    }
```

After:
```rust
    {
        let {{{state_var}}} = &{{{state_group_var}}}.{{{id}}};
        let result = tick({{{state_var}}}, tick_info)?;
        {{{update_group_var}}}.{{{id}}} = result;
    }
```

Also change the final `Ok(update)` → `Ok({{{update_group_var}}})` in the group_tick function.

Note: `{{{state_var}}}` is `"x"` (same name used in tick.hbs), passed to the tick function as the first argument.

---

### Step 19: Build and verify compilation

**Command:** `cargo build`

Verify that:
- Generated code compiles without errors
- `tick.rs` uses `x` for Persistent parameter and `y` for Update local
- `group.rs` uses `xs` for GroupPersistent parameter and `ys` for GroupUpdate local
- `tick.rs` tick function: `pub fn tick(x: &Persistent, tick_info: &TickInfo) -> Result<Update, TickError>`
- `group.rs` group_tick: `pub fn group_tick(xs: &GroupPersistent, tick_info: &TickInfo) -> Result<GroupUpdate, TickError>`
- No compiler warnings

---

### Step 20: Run tests

**Command:** `cargo test`

Verify all existing tests pass.

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| `persistent_fields` parameter in `expr_to_rust`/`stmt_to_rust` is unused | It was a placeholder for validation that was never implemented. Replacing with `CodegenContext` is clean. |
| `x` is a very short name, might clash with user variables | `x` is the **parameter name** for the Persistent struct, not a field name — user variables are fields inside Persistent, so no clash |
| `state_name` field in Persistent not affected | `state_name` is a struct field name, not the parameter name — correctly unchanged |
| `expr_to_rust` generates `x.{ident}` — need to verify expression resolution works | Since `x` is the first parameter of the tick function (of type `&Persistent`), `x.counter`, `x.error_flag`, etc. all resolve correctly |
| Template variables `state_var` / `update_var` / `state_group_var` / `update_group_var` must exist in JSON data | Codegen builds the JSON with these fields (Steps 3, 4, 6), passed from RustBackend constants (Step 11) |
| `persistent_fields` parameter removed from `expr_to_rust`/`stmt_to_rust`/`condition_to_rust` | All call sites updated in Steps 8-10 to use `CodegenContext` instead |

