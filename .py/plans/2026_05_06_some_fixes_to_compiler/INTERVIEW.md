# INTERVIEW: Some Fixes to Compiler

## Objective

Apply several fixes and refactorings to the pall state machine compiler to improve correctness, cleanliness, and code organization.

---

## Decision 1: Fix Transition Logic — if/else chain instead of `return` inside match

**Problem:** The current `tick()` function uses `match x.state { ... }` for state dispatch, but inside each state branch, transitions are sequential `if` blocks followed by `return Ok(y)`. This means:
- Multiple transitions could fire in sequence (they should be mutually exclusive and ordered)
- `return Ok(y)` skips signal/timer calculation that happens after the match block
- Only the first matching transition fires because of `return`, but the pattern is fragile

**Solution:**
- Keep `match x.state { ... }` for state dispatch (only the current state executes)
- **Inside each state branch**, transitions become an `if / else-if / else-if / else` chain
- Actions remain as separate `if when_clause { ... }` blocks (all can fire, in order)
- **No `return Ok(y)` inside the state branches** — signals and timers always execute after the match block
- If an always-true (fallback) transition exists → it becomes the `else` clause
- If no always-true transition → the chain ends, state stays unchanged

**Template changes (`tick.hbs`):**
- Actions: `{{#each actions}} if {{{when}}} { ... } {{/each}}`
- Transitions: `if {{{first.when}}} { ... } {{else-if subsequent}} {{else}} (if always-true) {{/if}}`

---

## Decision 2: Eliminate `Update` struct — use `Persistent` everywhere

**Problem:** `Update` with `Option<>` fields is unnecessary complexity. Inputs actually CAN change (they receive values from outputs via links). But they change at the group level, not individual machine level. Having a separate Update struct that's almost identical to Persistent (minus inputs) creates duplication.

**Solution:**
- **Remove `Update` struct entirely.** Use `Persistent` for both input (x) and output (y).
- `tick(x: &Persistent, tick_info: &TickInfo) -> Persistent` — returns a full Persistent
- Inside tick: `let mut y = x.clone()` — y has ALL fields including inputs
- All assignments: `y.field = value` — no `Some()`, no `Option<>`, no `from_persistent`
- Caller: `state = tick(&state, &tick_info)?` — direct assignment replaces `apply_update`
- In group tick: `let mut ys = xs.clone()`, then propagate links into `ys.{machine}.{input}`, then `ys.{machine} = machine_tick(&ys.{machine}, tick_info)?`
- `Persistent::apply_update` method is no longer needed
- `state_name` is no longer needed on Persistent — use `state.as_str()` directly

**Template changes (`types.hbs`):**
- Remove `Update` struct entirely
- Remove `state_name` field from `Persistent`
- Add `impl Persistent { fn apply_update(&mut self, update: &Persistent) }` — copies all fields except inputs from update into self (for callers who want selective update)
- Or better: just use direct assignment, no apply_update needed

**Runner simplification:**
```rust
// Before: let update = tick(&state, &tick_info)?; apply_update(&mut state, &update);
// After:  state = tick(&state, &tick_info)?;
```

---

## Decision 3: x vs y — both are `Persistent`, x is input, y is output

**Problem:** Currently `y` is default-initialized (`Update::default()`), and expressions/statements reference `x` for reads and `y` for writes. This causes issues when multiple statements reference the same variable:
- `foo = bar + baz; foo = foo + beh;` would become `y.foo = x.bar + x.baz; y.foo = x.foo + x.beh;` — second line reads stale `x.foo` instead of updated `y.foo`

**Solution:**
- Both x and y are `Persistent` type
- `let mut y = x.clone()` at tick start — y has ALL fields
- All reads come from `y`:
  - Variables: `y.field`
  - Signals: `y.field`
  - Timers: `y.field`
  - Constants: `y.field`
  - State (in expressions): `y.state.as_str()` (special string representation)
- Inputs always read from `x`: `x.field` (they're constant during tick, linked at group level)
- Statement targets always write to `y`: `y.field = ...`
- State cannot be a statement target (only transitions change state)
- After tick: `y.state` reflects any state change from transitions
- After tick: signals/timers are computed and stored in y
- Group level: `ys` is cloned from `xs`, links modify `ys` inputs, then individual ticks update `ys` machines

**Precalculated field access map:**
```json
{
  "field_accesses": {
    "state":      "y.state.as_str()",
    "inputs":     { "start": "x.start" },
    "variables":  { "counter": "y.counter" },
    "signals":    { "val": "y.val" },
    "timers":     { "elapsed": "y.elapsed" },
    "constants":  { "MAX": "y.MAX" }
  }
}
```

- `state` is always `y.state.as_str()` — treated as a string in expressions
- Inputs are the only category using `x` prefix
- All other categories use `y` prefix
- Statement targets are always `y.field`

**Template changes (`tick.hbs`):**
- `let mut y = x.clone();` — simple clone, not Update::default()

---

## Decision 4: Remove `state_name` from Persistent

**Problem:** `Persistent` has both `state: State` (enum) and `state_name: String`. Duplicated state information.

**Solution:**
- Remove `state_name` field from `Persistent` struct
- Always compute via `state.as_str()` — not costly (const fn), single source of truth
- Update all references in templates and generated code
- Remove from `init()`: no `state_name` assignment
- Update `runner/stubs.rs`: remove `state_name` usage

---

## Decision 5: Group tick — clone xs, propagate links into ys, tick each machine

**Problem:** `GroupUpdate` was initialized with `Update::default()` and modified in place.

**Solution:**
- `GroupPersistent` and `GroupUpdate` both use `Persistent` for each machine
- Group tick: `let mut ys = xs.clone()` — full clone with all fields
- Phase 1: propagate links — `ys.{machine}.{input} = xs.{source_machine}.{output}`
- Phase 2: individual ticks — `ys.{machine} = {machine}_tick::tick(&ys.{machine}, tick_info)?`
- Return `Ok(ys)` at the end
- No intermediate Update structs

**Template changes (`group.hbs`):**
- `ys` is cloned from `xs`
- Links modify `ys` directly (not through Update)
- Individual ticks replace machines in `ys`
- Return `Ok(ys)`

---

## Decision 6: Move `codegen.rs` to `compiler/backend/rust/`

**Problem:** `codegen.rs` lives in `compiler/` but contains Rust-specific code generation logic. It should be with the Rust backend.

**Solution:**
- Move `src/compiler/codegen.rs` → `src/compiler/backend/rust/codegen.rs`
- Update imports in `compiler/backend/rust/mod.rs` (remove parent prefix)
- Update `compiler/mod.rs` to export from new location if needed
- The `Backend` trait in `compiler/backend/mod.rs` stays as-is (it doesn't depend on codegen)

---

## Decision 7: Precalculated field access rendering (suggestion)

**Approach:** Build the field access mapping in `build_tick_data()` so that `expr_to_rust` and `stmt_to_rust` don't need to build `x.{field}` or `y.{field}` strings manually.

**Benefits:**
- Single place for the x/y decision
- Simpler `expr_to_rust` — just look up the access string
- Template rendering is cleaner — no need to pass context just for field resolution
- `state` special handling (`y.state.as_str()`) is handled uniformly

**Implementation:**
- In `build_tick_data()`, build a JSON array of field accesses
- Each entry: `{ "name": "counter", "access": "y.counter", "is_target": true }`
- Or use a nested JSON: `{ "state": "y.state.as_str()", "inputs": { "start": "x.start" }, "variables": { "counter": "y.counter" }, ... }`
- In `expr_to_rust`, look up the reference name and return the precalculated string
- In `stmt_to_rust`, the target is always `y.{field}`, the source uses the precalculated map

---

## Affected Files

| File | Action |
|------|--------|
| `src/compiler/codegen.rs` | **Move** → `src/compiler/backend/rust/codegen.rs` |
| `src/compiler/backend/rust/mod.rs` | Update imports, update field access building |
| `src/compiler/mod.rs` | Update import path for codegen if re-exported |
| `src/compiler/backend/rust/templates/tick.hbs` | Rewrite: if/else chain, x.clone(), no return, no state_name, no Update |
| `src/compiler/backend/rust/templates/types.hbs` | Rewrite: remove Update, remove state_name from Persistent, keep x/y as Persistent |
| `src/compiler/backend/rust/templates/group.hbs` | Rewrite: clone xs into ys, propagate links, individual ticks replace ys machines |
| `src/bin/creator/src/main.rs` | Simplify: no apply_update/from_persistent needed |
| `src/bin/runner/src/stubs.rs` | Remove ApplyUpdate trait, remove state_name, Update references |
| `src/bin/runner/src/main.rs` | Simplify: `state = tick(&state, &tick_info)?` |
| `src/bin/runner/generated/` | **Regenerate** after compiler changes |

---

## Test Strategy

- All existing 43 lib tests + 3 runner tests should pass after fixes
- Update any tests affected by the Update API change
- After code generation, regenerate and verify compilation + tests pass
- Verify the counter_test machine still works correctly (reaches goal in expected ticks)

---

## Not Changing

- Machine types (`machine/` module) — AST, types, parser unchanged
- Validation logic (`compiler/validation.rs`) — no changes to validation
- Existing 43 lib tests — should still pass (API changes to Update will need updates)
- `src/main.rs` demo binary — kept as-is
