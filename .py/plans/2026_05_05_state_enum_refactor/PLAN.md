# Plan: State Enum Refactor

## Problem

State in generated Rust code is represented as `String`, requiring runtime string matching and offering no compile-time safety. This refactor replaces `String` with a generated `State` enum.

## Scope

**IN:**
- Generate `State` enum in `types.rs`
- Change `Persistent.state` from `String` to `State`
- Add `state_name: String` field to `Persistent` for expression resolution
- Update `Update.state` to `Option<State>`
- Update `tick.hbs` to match enum variants directly
- Add `state` reserved variable validation
- Export `State` from `mod.rs`
- Update `main.rs` sample code

**OUT:**
- Changes to `group.hbs` (no changes needed)
- Changes to machine-level types.rs/types.rs (the parser-side types — unchanged)
- Changes to TickErrorKind::UnknownState removal (kept for backward compat, just unreachable)

## Approach

Micro-stepped changes, each touching a minimal set of files with a single concern.

---

## Plan Steps

### Step 1: Add `ReservedVariableName` error kind
**Files:** `src/compiler/error.rs`
- Add `ReservedVariableName` variant to `CompileErrorKind`
- Add display impl for the new variant ("reserved variable name")

### Step 2: Add `state` reserved name validation
**Files:** `src/compiler/validation.rs`
- Add a new validation function `validate_reserved_variables` that checks each machine's `variables` map for a key named `"state"`
- Add a new validation function `validate_state_reference` that checks each machine's `states` map for keys named `"state"` (state name collision)
- Call these new validators from `validate_machines`
- Add a test for reserved `state` variable detection

### Step 3: Update `build_types_data` to include state list
**Files:** `src/compiler/codegen.rs`
- Add a `states` array to the JSON data: list of `{"name": "StateVariant", "raw_name": "raw_name"}` for each state
- Build PascalCase variant name from the state name (e.g., `"initial"` → `"Initial"`, `"running"` → `"Running"`)

### Step 4: Update `types.hbs` — Add `State` enum
**Files:** `src/compiler/backend/rust/templates/types.hbs`
- Generate `pub enum State { Initial, Running, Error, ... }` with derived traits: `Serialize, Deserialize, Debug, Clone, Copy, PartialEq`
- Implement `Display` trait for `as_str()` returning lowercase string names
- Implement `TryFrom<String>` and `TryFrom<&str>` for infallible conversion back (returns error on unknown)
- Use definition order from the states list

### Step 5: Update `types.hbs` — Change `Persistent.state` type and add `state_name`
**Files:** `src/compiler/backend/rust/templates/types.hbs`
- Change `pub state: String` to `pub state: State` in `Persistent` struct
- Add `pub state_name: String` field to `Persistent` struct (for expression resolution of `state` as a string)
- Change `pub state: Option<String>` to `pub state: Option<State>` in `Update` struct

### Step 6: Update `tick.hbs` — Change match to use enum variants
**Files:** `src/compiler/backend/rust/templates/tick.hbs`
- Change `match state.state.as_str()` to `match state.state`
- Change match arms from `"state_name"` → `State::StateName` (PascalCase)
- Remove the `_` catch-all branch (unknown state is now impossible)

### Step 7: Update `tick.hbs` — Change transition `update.state` assignment
**Files:** `src/compiler/backend/rust/templates/tick.hbs`
- Change `update.state = Some("target".to_string())` to `update.state = Some(State::Target)` (PascalCase)

### Step 8: Update `tick.hbs` — Change `init()` to use enum
**Files:** `src/compiler/backend/rust/templates/tick.hbs`
- Change `state: "{{initial}}".to_string()` to `state: State::Initial` (PascalCase)
- Add `state_name: "{{initial}}".to_string()` to the init struct

### Step 9: Update `codegen.rs` — Pass state names for expression resolution
**Files:** `src/compiler/codegen.rs`
- In `build_tick_data`, include a mapping of lowercase state names to PascalCase enum variants in the JSON data
- This enables the tick template to generate correct variant names

### Step 10: Update `mod.hbs` — Export `State` enum
**Files:** `src/compiler/backend/rust/templates/mod.hbs`
- Add `pub use {{{id}}}_types::State;` for each machine

### Step 11: Update `main.rs` — Use `State` enum in sample
**Files:** `src/main.rs`
- Remove `use machine::*` if needed, use explicit imports
- No code changes needed for `main.rs` — the sample machine is defined at the data-model level (StateMachine), not the generated level

### Step 12: Build and verify compilation
**Command:** `cargo build`
- Verify generated code compiles
- Check that `State` enum is correctly generated
- Verify no warnings or errors

### Step 13: Run tests
**Command:** `cargo test`
- Verify all existing tests pass
- Verify new validation test passes

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| `state` reference in expressions resolves to wrong thing | Codegen maps `state` reference to `state_name` field in generated code |
| State name collision with user-defined variable | Validation rejects `state` as a variable name before codegen |
| `state` as a state name itself | Validation rejects `state` as a state name before codegen |
| `TickErrorKind::UnknownState` becomes dead code | It's kept for backward compatibility; just unreachable |
| `TryFrom` error type mismatch | Use a simple `enum StateParseError { Unknown(String) }` or the existing pattern |
