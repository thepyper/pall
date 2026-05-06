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

## Decision 2: Update structure — from `Option<>` to actual values

**Problem:** `Update` has `Option<FieldType>` for every field, requiring `Some(x)/None` checks everywhere.

**Solution:**
- `Update` fields become direct types (no `Option<>`): `FieldType` instead of `Option<FieldType>`
- `Update` contains all `Persistent` fields **except inputs** (inputs are constant, never updated)
- Add `impl Update { fn from_persistent(x: &Persistent) -> Self }` — copies all non-input fields (acts as clone minus inputs)
- Add `impl Persistent { fn apply_update(&mut self, update: Update) }` — copies Update values back into Persistent
- Users compare `x.field != y.field` to detect changes
- Remove `Default` derive from `Update` (no longer meaningful)

**Template changes (`types.hbs`):**
- Update struct: `pub {{{name}}}: {{rust_type}},` instead of `pub {{{name}}}: Option<{{rust_type}}>,`
- No `Default` derive on Update

---

## Decision 3: x vs y — all reads use y, inputs use x

**Problem:** Currently `y` is default-initialized (`Update::default()`), and expressions/statements reference `x` for reads and `y` for writes. This causes issues when multiple statements reference the same variable:
- `foo = bar + baz; foo = foo + beh;` would become `y.foo = x.bar + x.baz; y.foo = x.foo + x.beh;` — second line reads stale `x.foo` instead of updated `y.foo`

**Solution:**
- `y` initialized as `Update::from_persistent(x)` at tick start (copy of x for all non-input fields)
- All reads come from `y`:
  - Variables: `y.field`
  - Signals: `y.field`
  - Timers: `y.field`
  - Constants: `y.field`
  - State (in expressions): `y.state.as_str()` (special string representation)
- Inputs always read from `x`: `x.field` (they're constant, never in y)
- Statement targets always write to `y`: `y.field = ...`
- State cannot be a statement target (only transitions change state)

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

- `state` is always `y.state.as_str()` — treated as a string in expressions for comparisons like `state == "counting"`
- Inputs are the only category using `x` prefix
- All other categories use `y` prefix
- Statement targets (left-hand side) are always `y.field` regardless of category

**Template changes (`tick.hbs`):**
- `let mut y = Update::from_persistent(x);` instead of `let mut y = Update::default();`

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

## Decision 5: GroupUpdate — collect from individual ticks

**Problem:** `GroupUpdate` is initialized with `Update::default()` and then modified. This is unnecessary with the new `Update::from_persistent` pattern.

**Solution:**
- Call each machine's `tick()` individually: `let result1 = machine1::tick(xs.machine1, tick_info)?`
- Collect results into local variables
- Build `GroupUpdate` at end: `GroupUpdate { machine1: result1, machine2: result2, ... }`
- Link propagation stays in phase 1 but now propagates to the individual tick inputs directly

**Template changes (`group.hbs`):**
- No `GroupUpdate { ...: Update::default() }` initialization
- Individual tick calls with result variables
- Final `Ok(GroupUpdate { ... })` at the end

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
| `src/compiler/backend/rust/templates/tick.hbs` | Rewrite: if/else chain, from_persistent, no return, no state_name |
| `src/compiler/backend/rust/templates/types.hbs` | Rewrite: Update without Option<>, no state_name, add impl blocks |
| `src/compiler/backend/rust/templates/group.hbs` | Rewrite: collect from individual ticks, no default Init |
| `src/bin/creator/src/main.rs` | Update for API changes (apply_update, from_persistent) |
| `src/bin/runner/src/stubs.rs` | Update apply_update → Persistent::apply_update, remove state_name, Update no longer has Option |
| `src/bin/runner/src/main.rs` | Update for API changes, add tests |
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
