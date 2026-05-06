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

## Phase 2: Refactor `Update` structure — remove `Option<>`

### Step 2.1
Update `types.hbs` template: change Update struct to have direct field types (no `Option<>`)
- Variables: `pub {{{name}}}: {{rust_type}},`
- Signals: `pub {{{name}}}: {{rust_type}},`
- Timers: `pub {{{name}}}: {{rust_type}},`
- State: `pub state: State,`
- Remove `Default` derive from Update
- Remove `Clone` derive? No, keep Clone so we can copy Update values

### Step 2.2
Update `types.hbs` template: add `Update::from_persistent(x: &Persistent) -> Self` impl
- Copies all fields from Persistent except inputs
- For state, copies `x.state` (not `x.state_name`)
- For variables, signals, timers: copy the field value directly

### Step 2.3
Update `types.hbs` template: add `Persistent::apply_update(&mut self, update: Update)` method
- Copies all fields from Update back into self
- For state: set `self.state = update.state` and `self.state_name = update.state.as_str().to_string()` — wait, we're removing state_name! So: `self.state = update.state` and update state_name if we keep it... NO, remove state_name entirely.
- For variables, signals, timers: `self.{field} = update.{field}`

### Step 2.4
Regenerate code with `cargo run --bin creator` to update `src/bin/runner/generated/`

### Step 2.5
Verify compilation: `cargo check -p pall` — should pass

---

## Phase 3: Remove `state_name` from Persistent

### Step 3.1
Update `types.hbs` template: remove `state_name: String` field from `Persistent` struct

### Step 3.2
Update `types.hbs` template: remove `state_name` from `init()` function

### Step 3.3
Update `types.hbs` template: in `Update::from_persistent`, remove `state_name` copy

### Step 3.4
Update `types.hbs` template: in `Persistent::apply_update`, remove `state_name` handling

### Step 3.5
Update `tick.hbs` template: remove any `state_name` references (should be none after tick.hbs changes in Phase 5)

### Step 3.6
Update `group.hbs` template: remove any `state_name` references

### Step 3.7
Update `src/bin/runner/src/stubs.rs`: remove `state_name` from `apply_update` function

### Step 3.8
Regenerate code with `cargo run --bin creator`

### Step 3.9
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
- Output: also build a list of statement targets (always `y.{field}`) — inputs cannot be targets

### Step 4.2
Update `codegen.rs`: modify `expr_to_rust()` to use the precalculated field access map
- For `Expression::Reference(r)`: look up `r.target` in the access map and return the precalculated string
- No longer need `CodegenContext.state_var` for reference resolution

### Step 4.3
Update `codegen.rs`: modify `stmt_to_rust()` to use the precalculated access map for expression sources
- Target is always `y.{field}` (inputs can't be targets)
- Expression sources use the precalculated strings from the access map

### Step 4.4
Update `tick.hbs` template: remove `state_var` and `update_var` from the template data (replaced by field access map)
- Or keep them for backward compatibility but deprecate

### Step 4.5
Update `tick.hbs` template: change `let mut y = Update::default();` to `let mut y = Update::from_persistent(x);`

### Step 4.6
Update `codegen.rs`: update `build_types_data()` to include field access map in the JSON (needed for types template if it uses field accesses)

### Step 4.7
Update `codegen.rs`: update `build_group_data()` for any field access changes

### Step 4.8
Regenerate code with `cargo run --bin creator`

### Step 4.9
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
Update `tick.hbs` template: remove `return Ok(y)` from inside each state branch
- The `return Ok(y)` was causing signals/timers to be skipped
- Instead, let the function fall through to the signal/timer section
- Return `Ok(y)` only at the very end, after signals and timers

### Step 5.3
Update `tick.hbs` template: handle transition state assignment without return
- Transitions set `y.state = State::Target` (no `Some()`, no return)
- The state change is reflected in the Update struct for the caller to see
- But signals and timers still execute for the current state

### Step 5.4
Update `tick.hbs` template: verify signal calculation section still uses `y.{field}` (precalculated map)
- Signals assign to `y.{field}` (now direct, no `Some()`)
- Remove `Some(...)` wrapping from signal values

### Step 5.5
Update `tick.hbs` template: verify timer accumulation section uses `y.{field}`
- Timers accumulate to `y.{field}` (now direct, no `Some()`)
- Remove `Some(...)` wrapping from timer values

### Step 5.6
Update `tick.hbs` template: verify actions section uses precalculated field accesses
- Actions' `when` clauses use precalculated access (e.g., `y.state.as_str() == "foo"`)
- Actions' statement sources use precalculated access (e.g., `y.counter`)
- Actions' statement targets use `y.{field} = ...`

### Step 5.7
Regenerate code with `cargo run --bin creator`

### Step 5.8
Verify the generated `tick.rs` — check:
- No `return Ok(y)` inside state branches
- Transitions use `if/else-if/else` chain
- `y` is initialized with `Update::from_persistent(x)`
- No `Option<>` in Update assignments

### Step 5.9
Verify compilation: `cargo check -p pall`

---

## Phase 6: Fix `group.hbs` — collect from individual ticks

### Step 6.1
Update `group.hbs` template: remove `GroupUpdate` initialization with `Update::default()`
- Instead, collect each machine's tick result into a local variable
- Build `GroupUpdate` at the end

### Step 6.2
Update `group.hbs` template: rewrite tick loop to individual calls
- For each machine: `let {id}_result = {id}_tick::tick(&{xs}.{id}, tick_info)?;`
- After all ticks: `Ok(GroupUpdate { {id1}: {id1}_result, {id2}: {id2}_result, ... })`

### Step 6.3
Update `group.hbs` template: handle link propagation
- Phase 1: propagate links from source machines to target machines' persistent state
- Since links go into inputs (which are in Persistent but not in Update), we need to modify the source machine's persistent state before calling tick
- Use `let mut xs.{id} = xs.{id}.clone();` then modify inputs, then call tick
- Or: clone the whole GroupPersistent, modify inputs, then tick

### Step 6.4
Regenerate code with `cargo run --bin creator`

### Step 6.5
Verify compilation: `cargo check -p pall`

---

## Phase 7: Update `types.hbs` — complete Update struct changes

### Step 7.1
Verify `types.hbs` has no remaining `Option<>` in Update struct (done in Phase 2)

### Step 7.2
Verify `types.hbs` has no `Default` derive on Update (done in Phase 2)

### Step 7.3
Verify `types.hbs` `Persistent` struct has no `state_name` field (done in Phase 3)

### Step 7.4
Verify `types.hbs` `Persistent::apply_update` method is present and correct (done in Phase 2)

### Step 7.5
Verify `types.hbs` `Update::from_persistent` method is present and correct (done in Phase 2)

### Step 7.6
Regenerate code with `cargo run --bin creator`

### Step 7.7
Verify compilation: `cargo check -p pall`

---

## Phase 8: Update downstream code (runner, creator)

### Step 8.1
Update `src/bin/runner/src/stubs.rs`: remove `apply_update` function (now `Persistent::apply_update`)

### Step 8.2
Update `src/bin/runner/src/stubs.rs`: remove `state_name` references from any remaining code

### Step 8.3
Update `src/bin/runner/src/main.rs`: change `apply_update(&mut state, &update)` to `state.apply_update(update)`

### Step 8.4
Update `src/bin/runner/src/main.rs`: remove any `state_name` references

### Step 8.5
Update `src/bin/runner/src/main.rs`: verify tests still compile and run correctly
- `test_counter_reaches_goal`
- `test_goal_reached_within_max_ticks`
- `test_generated_code_compiles`

### Step 8.6
Verify compilation: `cargo check -p pall`

---

## Phase 9: Regenerate code and run full test suite

### Step 9.1
Run `cargo run --bin creator` to regenerate all generated code

### Step 9.2
Run `cargo check -p pall` — verify no compilation errors

### Step 9.3
Run `cargo test -p pall` — verify all tests pass

### Step 9.4
Run `cargo clippy -p pall` — check for warnings

### Step 9.5
Review generated code manually:
- `src/bin/runner/generated/counter_test/tick.rs` — no return in state branches, if/else chain, from_persistent
- `src/bin/runner/generated/counter_test/types.rs` — Update has direct types, from_persistent, apply_update
- `src/bin/runner/generated/group.rs` — collects from individual ticks

---

## Phase 10: Final verification

### Step 10.1
Run `cargo test -p pall -- --nocapture` — verify all test output is correct

### Step 10.2
Run `cargo run --bin runner` — verify runtime behavior is correct
- Counter reaches goal state
- Expected number of ticks

### Step 10.3
Run `cargo build --release -p pall` — verify release build succeeds

### Step 10.4
Verify no regressions in existing 43 lib tests
