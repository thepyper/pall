# Interview: Persistent and Update Variable Names Change

## Broad Objective

Make the variable names used in generated Rust code for the `Persistent` and `Update` structs configurable via compile-time constants in `RustBackend`, instead of being hardcoded string literals scattered across templates and codegen code. This produces more concise generated code and allows easy name changes.

## Key Decisions

### 1. Struct Names vs Variable Names
- **Struct names** (`Persistent`, `Update`, `GroupPersistent`, `GroupUpdate`) remain **unchanged**
- **Variable names** (local bindings / field access prefixes in generated code) are what get configurable

### 2. Variable Name Constants
Four compile-time constants in `RustBackend`:

| Constant | Default Value | Used for |
|---|---|---|
| `STATE_NAME` | `"x"` | Persistent parameter name in per-machine `tick()` and `group.hbs` |
| `UPDATE_NAME` | `"y"` | Update local variable name in per-machine `tick()` |
| `STATE_GROUP_NAME` | `"xs"` | GroupPersistent parameter name in `group.hbs` (group of state machines) |
| `UPDATE_GROUP_NAME` | `"ys"` | GroupUpdate local variable name in `group.hbs` (group of updates) |

### 3. Constant Location
- Constants live in the `RustBackend` struct in `src/compiler/backend/rust/mod.rs`
- They are `pub const` values on the struct
- They are **compile-time constants** (hardcoded `const`), not runtime-configurable fields

### 4. Template Changes
- `tick.hbs`: Replace hardcoded `state` → `{{state_var}}`, `update` → `{{update_var}}`
- `group.hbs`: Replace hardcoded `group` → `{{state_group_var}}`, `update` → `{{update_group_var}}`
- Both templates receive the names as data context keys from codegen

### 5. Codegen Changes
- `expr_to_rust` in `codegen.rs`: generates `state.{ident}` → `x.{ident}`
- `stmt_to_rust` in `codegen.rs`: generates `update.{target}` → `y.{target}`
- These functions need to receive the variable names from the data context
- `build_types_data`, `build_tick_data`, and `build_group_data` need to pass these names

### 6. Scope of Changes
- Only `tick.hbs`, `group.hbs`, and `codegen.rs` are affected
- `types.hbs` and `mod.hbs` have NO changes (struct names unchanged)
- `group.hbs` also changes: the `let state = &group.xxx` binding → `let x = &xs.xxx`
- `types.hbs` struct definitions stay the same (struct names unchanged)

### 7. Backward Compatibility
- Not a concern — this is internal code generation, no external API consumers

### 8. Coherence Between tick and group Templates
- `tick.hbs` uses `x` for the Persistent parameter → `group.hbs` also uses `x` for consistency
- The `let state = &group.xxx` in `group.hbs` passes `x` (the same name) to `tick(x, ...)`
- Group uses `xs`/`ys` to signify "collection of" (many x, many y)

## Files Affected

| File | Changes |
|---|---|
| `src/compiler/backend/rust/mod.rs` | Add 4 `const` fields to `RustBackend`; use them in `compile()` data builders |
| `src/compiler/codegen.rs` | Add variable name fields to JSON data; update `expr_to_rust` and `stmt_to_rust` signatures |
| `src/compiler/backend/rust/templates/tick.hbs` | Replace `state` → `{{state_var}}`, `update` → `{{update_var}}` |
| `src/compiler/backend/rust/templates/group.hbs` | Replace `group` → `{{state_group_var}}`, `update` → `{{update_group_var}}` |
| `src/main.rs` | No changes (uses Compiler API, not generated code directly) |

## Summary of Generated Code Changes

**Before (tick.rs):**
```rust
pub fn tick(state: &Persistent, ...) -> Result<Update, ...> {
    let mut update = Update::default();
    // state.xxx, update.yyy
}
```

**After (tick.rs):**
```rust
pub fn tick(x: &Persistent, ...) -> Result<y, ...> {
    let mut y = y::default();
    // x.xxx, y.yyy
}
```

**Before (group.rs):**
```rust
pub fn group_tick(group: &GroupPersistent, ...) -> Result<GroupUpdate, ...> {
    let mut update = GroupUpdate { ... };
    let state = &group.xxx;
    tick(state, ...);
}
```

**After (group.rs):**
```rust
pub fn group_tick(xs: &GroupPersistent, ...) -> Result<ys, ...> {
    let mut ys = ys { ... };
    let x = &xs.xxx;
    tick(x, ...);
}
```
