# Interview: State Enum Refactor

## Broad Objective

Replace the `String`-based state representation in the **generated Rust code** with a proper Rust `enum`, improving type safety. State names in the `.pall` definition should remain lowercase identifiers and be used as the string representation via conversion functions.

## Key Decisions

### 1. Generated File Location
- State enum is generated in **`types.rs`** (via `types.hbs` template)

### 2. Enum Variant Naming
- PascalCase for enum variants (e.g., `"initial"` → `Initial`)
- String representation keeps the original lowercase name (e.g., `state.as_str()` returns `"initial"`)

### 3. Derive Traits
- `Serialize`, `Deserialize`, `Debug`, `Clone`, `Copy`, `PartialEq`
- **Not** `Eq` or `Hash` (not needed for generated code)

### 4. Conversion to/from String
- `TryFrom<String>` and `TryFrom<&str>` → `Result<State, TryFromStateError>` (returns error for unrecognized strings)
- Infallible `as_str()` method → `&'static str` (returns canonical lowercase name)

### 5. Special `state` Variable
- `Persistent` struct has `state: State` (the enum)
- `Persistent` struct has `state_name: String` (the string representation, derived from `state.as_str()`)
- When `state` is referenced in expressions/statements, it resolves to `state_name` (string)
- `state` is **read-only** — only transitions can change it
- `Update.state` is `Option<State>` (enum)

### 6. Tick Error Handling
- Remove the `_` catch-all branch in `tick.hbs` — unknown state is now **impossible** at compile time
- `TickErrorKind::UnknownState` variant remains for backward compatibility but is no longer reachable

### 7. Reserved Variable
- `state` is a **reserved variable name**
- Compiler validates and errors if a user-defined variable is named `state`

### 8. Tick Template Match
- `match state.state.as_str() { ... }` becomes `match state.state { State::Initial => { ... } }`
- Matches enum variants directly, no string comparison needed

### 9. Assignment Error
- Attempting to assign to `state` (e.g., `state = "running"`) is invalid
- Only transitions can change state (enforced by Rust compile-time type system: `Update.state` is the only writable field)

### 10. Variable Naming Conflict
- `state` as a user-defined variable name is **rejected** with a compile error

### 11. Enum Variant Ordering
- Definition order (same order as states appear in the `.pall` definition)

### 12. Update.state
- `Update.state` is `Option<State>` (enum), replacing `Option<String>`

### 13. Eq / Hash
- Not derived (not useful for generated code)

## Files Affected (Summary)

- `src/compiler/backend/rust/templates/types.hbs` — Add enum, change field types
- `src/compiler/backend/rust/templates/tick.hbs` — Change match to enum, remove catch-all
- `src/compiler/backend/rust/templates/mod.hbs` — Export `State` enum
- `src/compiler/backend/rust/templates/group.hbs` — No changes needed
- `src/compiler/codegen.rs` — Add state list to data builders
- `src/compiler/validation.rs` — Add `state` reserved name check
- `src/compiler/error.rs` — Add `ReservedVariableName` error kind
- `src/main.rs` — Update sample to use `State` enum
