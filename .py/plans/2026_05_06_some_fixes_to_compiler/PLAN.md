# PLAN: Some Fixes to Compiler

## Phase 1: Move `codegen.rs` to `compiler/backend/rust/`

### Step 1.1
Move `src/compiler/codegen.rs` to `src/compiler/backend/rust/codegen.rs`

### Step 1.2
Update `src/compiler/backend/rust/mod.rs` to import from `codegen` (same module, no parent prefix needed)
- Change `super::super::codegen` → `super::codegen` or `crate::compiler::backend::rust::codegen`
- Update `CodegenContext` import path accordingly

### Step 1.3
Update `src/compiler/mod.rs` — verify no re-export of `codegen` that would break; if it does, update the path

### Step 1.4
Verify compilation: `cargo check -p pall` — should pass with no new errors

---

## Phase 2: Remove `state_name` from Persistent

### Step 2.1
Update `types.hbs` template: remove `state_name: String` field from `Persistent` struct

### Step 2.2
Update `types.hbs` template: remove `state_name` from `init()` function

### Step 2.3
Update `tick.hbs` template: remove any `state_name` references

### Step 2.4
Update `group.hbs` template: remove any `state_name` references

### Step 2.5
Update `src/bin/runner/src/stubs.rs`: remove `state_name` from `apply_update` function

### Step 2.6
Regenerate code with `cargo run --bin creator`

### Step 2.7
Verify compilation: `cargo check -p pall`

---

## Phase 3: Eliminate Update struct — use Persistent everywhere

### Step 3.1
Update `types.hbs` template: remove the `Update` struct entirely

### Step 3.2
Update `types.hbs` template: change `tick()` signature from `-> Result<Update, TickError>` to `-> Result<Persistent, TickError>`

### Step 3.3
Update `types.hbs` template: change `init()` return type stays `Persistent`

### Step 3.4
Update `tick.hbs` template: change `let mut y = Update::default();` to `let mut y = x.clone();`

### Step 3.5
Update `tick.hbs` template: change all assignments from `y.field = Some(value);` to `y.field = value;`
- Remove all `Some()` wrappers from signal assignments
- Remove all `Some()` wrappers from timer assignments
- Remove all `Some()` wrappers from variable assignments
- Remove all `Some()` wrappers from state assignments in transitions

### Step 3.6
Update `tick.hbs` template: change `return Ok(y);` in transitions to `y.state = State::Target;` (fall through)
- No return inside state branches
- Transition sets `y.state` and falls through to signal/timer section
- Return `Ok(y)` only at the very end

### Step 3.7
Update `tick.hbs` template: ensure signal/timer section works with direct Persistent fields
- Signals: `y.{field} = expr;` (no Some)
- Timers: `y.{field} = y.{field} + delta_ms;` or `y.{field} = 0;` (no Some)

### Step 3.8
Update `group.hbs` template: change `Update::default()` to `xs.{id}.clone()` or restructure to clone xs first
- Rewrite: `let mut ys = xs.clone();`
- Phase 1 (links): `ys.{machine}.{input} = xs.{source}.{output};`
- Phase 2 (ticks): `ys.{machine} = {machine}_tick::tick(&ys.{machine}, tick_info)?;`
- Return `Ok(ys)`

### Step 3.9
Update `mod.hbs` template: remove `Update` re-export from each machine module

### Step 3.10
Update `src/bin/runner/src/stubs.rs`: remove `apply_update` function

### Step 3.11
Update `src/bin/runner/src/main.rs`: change `apply_update(&mut state, &update)` to `state = tick(&state, &tick_info)?`

### Step 3.12
Update `src/bin/creator/src/main.rs`: remove any Update references

### Step 3.13
Regenerate code with `cargo run --bin creator`

### Step 3.14
Verify compilation: `cargo check -p pall`

---

## Phase 4: Implement precalculated field access + x/y in expressions

### Step 4.1
Update `codegen.rs` (in `backend/rust/`): modify `build_tick_data()` to build the field access map
- State: access string `"y.state.as_str()"` (read-only, treated as string)
- Inputs: access strings `"x.{field}"` for each input (read-only)
- Variables: access strings `"y.{field}"` for each variable (read and write)
- Signals: access strings `"y.{field}"` for each signal (read and write)
- Timers: access strings `"y.{field}"` for each timer (read and write)
- Constants: access strings `"y.{field}"` for each constant (read-only)

### Step 4.2
Update `codegen.rs`: modify `expr_to_rust()` to use the precalculated field access map
- For `Expression::Reference(r)`: look up `r.target` in the access map and return the precalculated string
- No longer need `CodegenContext.state_var` for reference resolution

### Step 4.3
Update `codegen.rs`: modify `stmt_to_rust()` to use the precalculated access map for expression sources
- Target is always `y.{field}` (inputs can't be targets)
- Expression sources use the precalculated strings from the access map

### Step 4.4
Update `tick.hbs` template: simplify field references to use precalculated map (no manual `x.` or `y.` prefix in templates)

### Step 4.5
Update `codegen.rs`: update `build_types_data()` to include field access map

### Step 4.6
Update `codegen.rs`: update `build_group_data()` for any field access changes

### Step 4.7
Regenerate code with `cargo run --bin creator`

### Step 4.8
Verify compilation: `cargo check -p pall`

---

## Phase 5: Fix `tick.hbs` — if/else chain for transitions, no return

### Step 5.1
Update `tick.hbs` template: rewrite state transition logic from `match` to if/else chain
- Keep `match x.state { State::Foo => { ... }, ... }` for state dispatch
- Inside each state branch, replace sequential `if` transitions with `if / else-if / else-if / ... / else`
- Detect always-true transitions (where `when_rust_code` is empty string) — these become the `else` clause
- If no always-true transition, there is no `else` (state remains unchanged)

### Step 5.2
Update `tick.hbs` template: verify no `return` inside state branches
- The `return` was causing signals/timers to be skipped
- Let the function fall through to the signal/timer section
- Return `Ok(y)` only at the very end

### Step 5.3
Update `tick.hbs` template: handle transition state assignment
- Transitions set `y.state = State::Target` (direct assignment, no Some)
- Signals and timers still execute for the current state
- State change is reflected in the returned Persistent

### Step 5.4
Update `tick.hbs` template: verify signal/timer section uses direct field assignment (no Some)
- Signals: `y.{field} = expr;`
- Timers: `y.{field} = ...;`

### Step 5.5
Update `tick.hbs` template: verify actions use precalculated field accesses
- Actions' `when` clauses: `y.state.as_str() == "foo"` or `y.field == value`
- Actions' statement sources: `y.field`
- Actions' statement targets: `y.field = ...`

### Step 5.6
Regenerate code with `cargo run --bin creator`

### Step 5.7
Verify the generated `tick.rs` — check:
- No `return` inside state branches
- Transitions use `if/else-if/else` chain
- `y` is initialized with `x.clone()`
- All assignments are direct (no `Some()`)
- Signals/timers always execute

### Step 5.8
Verify compilation: `cargo check -p pall`

---

## Phase 6: Fix `group.hbs` — clone xs, propagate links, tick into ys

### Step 6.1
Update `group.hbs` template: rewrite to clone xs into ys
- `let mut ys = xs.clone();`

### Step 6.2
Update `group.hbs` template: rewrite link propagation
- For each link: `ys.{target_machine}.{target_var} = xs.{source_machine}.{source_var};`

### Step 6.3
Update `group.hbs` template: rewrite tick calls
- For each machine: `ys.{id} = {id}_tick::tick(&ys.{id}, tick_info)?;`

### Step 6.4
Update `group.hbs` template: return `Ok(ys)`

### Step 6.5
Update `mod.hbs` template: ensure `GroupPersistent` and `GroupUpdate` re-exports are correct
- Both should be `Persistent`-based

### Step 6.6
Regenerate code with `cargo run --bin creator`

### Step 6.7
Verify compilation: `cargo check -p pall`

---

## Phase 7: Update runner main.rs for new API

### Step 7.1
Update `src/bin/runner/src/main.rs`: change the tick loop to use new API
- `state = tick(&state, &tick_info)?;` instead of `let update = tick(...); apply_update(&mut state, &update);`

### Step 7.2
Update `src/bin/runner/src/main.rs`: remove state_name references

### Step 7.3
Update `src/bin/runner/src/stubs.rs`: remove apply_update, clean up imports

### Step 7.4
Verify tests still compile: `cargo test -p pall --no-run`

### Step 7.5
Verify compilation: `cargo check -p pall`

---

## Phase 8: Regenerate code and run full test suite

### Step 8.1
Run `cargo run --bin creator` to regenerate all generated code

### Step 8.2
Run `cargo check -p pall` — verify no compilation errors

### Step 8.3
Run `cargo test -p pall` — verify all tests pass

### Step 8.4
Run `cargo clippy -p pall` — check for warnings

### Step 8.5
Review generated code manually:
- `src/bin/runner/generated/counter_test/tick.rs` — if/else chain, no return, x.clone(), direct assignments
- `src/bin/runner/generated/counter_test/types.rs` — no Update struct, no state_name on Persistent
- `src/bin/runner/generated/group.rs` — clone xs, propagate links, individual ticks

---

## Phase 9: Final verification

### Step 9.1
Run `cargo test -p pall -- --nocapture` — verify all test output is correct

### Step 9.2
Run `cargo run --bin runner` — verify runtime behavior is correct
- Counter reaches goal state
- Expected number of ticks

### Step 9.3
Run `cargo build --release -p pall` — verify release build succeeds

### Step 9.4
Verify no regressions in existing 43 lib tests
