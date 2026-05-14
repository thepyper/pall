# Plan: Const and Literal Casting Rules

## Phase 1: typecheck_rules Refactoring (language-independent)

### Step 1.1: Add `CandidateSet` struct and `ResolvedType` enum

**File:** `src/compiler/typecheck_rules.rs` (after imports)

- `CandidateSet(pub Vec<Type>)` — ordered list of candidate types (smallest first)
  - Methods: `best() -> Option<&Type>`, `contains(&Type) -> bool`
- `ResolvedType` enum:
  - `Definite(Type)` — single resolved type (references, operations)
  - `Candidates(CandidateSet)` — candidate set (literals, constants, unary ops)
  - Methods: `as_type() -> Option<&Type>`, `to_candidates() -> CandidateSet`

### Step 1.2: Add `candidate_types_for_value(value: i64) -> CandidateSet`

Returns all types that can hold the integer value:
- Positive values (e.g., 42): `{U8, I8, U16, I16, U32, I32, U64, I64, F32, F64}` (subject to range checks)
- Negative values (e.g., -42): `{I8, I16, I32, I64, F32, F64}` (unsigned types excluded)
- Uses `int_value_fits(value, Type) -> bool` helper that checks range/precision per type

### Step 1.3: Add `candidate_types_for_float_value` → `{F32, F64}`

### Step 1.4: Add `candidate_types_for_bool_value` → `{Bool}`

### Step 1.5: Add `candidate_types_for_unary(candidates, op) -> CandidateSet`

- `Negate` (-): filters to `{I8, I16, I32, I64, F32, F64}` (signed only)
- `BitNot` (~): filters to `{U8, U16, U32, U64}` (unsigned ints only)
- `Not` (!): always `{Bool}`

### Step 1.6: Add `find_common_type_sets(set_a, set_b) -> Option<Type>`

Finds the best common type between two candidate sets:
1. For each target type (in order: smallest first), check if ALL types in both sets can cast to it via `is_cast_lossless`
2. Return the first such type (smallest common type)
3. Tie-break: prefer unsigned for same bit count

### Step 1.7: Add `is_cast_lossless_value(value: &Value, target_type: &Type) -> bool`

For literals/constants:
- Integer: `int_value_fits(value, target_type)` (value-aware, not just type-based)
- Float: target must be F32 or F64 (never float→integer)
- Bool: target must be numeric
- String: target must be String

### Step 1.8: Add `candidate_types_for_constant(value: &Value) -> CandidateSet`

Dispatches to `candidate_types_for_value`, `candidate_types_for_float_value`, or `candidate_types_for_bool_value`.

### Step 1.9: Add `value_default_type(value: &Value) -> Type`

Returns the default type for a value: `I64` for int, `F64` for float, etc.

### Step 1.10: Refactor `int_to_int_lossless` to match-based

Replace bit-counting logic with explicit type matching:
```
(U8, U16|U32|U64) → true
(U16, U32|U64) → true
(U32, U64) → true
(U8, I16|I32|I64) → true     // unsigned→larger signed
(U16, I32|I64) → true
(U32, I64) → true
(I8, I16|I32|I64) → true
(I16, I32|I64) → true
(I32, I64) → true
(I*, U*) → false               // never signed→unsigned
_ → false
```

### Step 1.11: Refactor `find_common_numeric_type` to use `find_common_type_sets`

```rust
fn find_common_numeric_type(from: &Type, to: &Type) -> Option<Type> {
    find_common_type_sets(&CandidateSet(vec![from.clone()]), &CandidateSet(vec![to.clone()]))
}
```

### Step 1.12: Refactor `int_to_float_lossless` to match-based (simplify existing)

### Step 1.13: Update existing tests to pass with new code

All existing tests should still pass. API surface is unchanged for external consumers.

### Step 1.14: Ensure deterministic HashMap key ordering in all AST walks

**CRITICAL**: All HashMap iterations that assign sequential IDs must be sorted by key.

In `typecheck.rs` (`infer_all_expressions`):
- `machine.signals.iter()` → `machine.signals.keys().sorted().map(...)`
- `machine.timers.iter()` → `machine.timers.keys().sorted().map(...)`
- `machine.states.iter()` → `machine.states.keys().sorted().map(...)`

In `type_validation.rs` (same pattern):
- Same sorted iteration for all HashMaps

In `codegen.rs` (build_tick_data):
- Same sorted iteration for all HashMaps

This ensures ExpressionIds are identical across typecheck, validation, and codegen.

### Step 1.15: Add new tests for candidate set functions

- `test_candidate_types_for_value`: 42, -42, 0, edge cases
- `test_candidate_types_for_unary`: negate, bitnot, logical not
- `test_find_common_type_sets`: various combinations
- `test_is_cast_lossless_value`: 42→U8 (true), -42→U8 (false)
- `test_int_to_int_lossless_match`: verify match-based logic

---

## Phase 2: typecheck.rs — Emit Candidate Sets

### Step 2.1: Update `TypeEnv` type alias to map to `ResolvedType`

Change: `pub type TypeEnv = HashMap<ExpressionId, ResolvedType>;`

### Step 2.2: Update `TypeChecker::new` to accept constants

Add `constants: HashMap<String, Constant>` field to `TypeChecker`.

### Step 2.3: Update `infer_value` to emit `ResolvedType::Candidates`

For each literal value type, emit `ResolvedType::Candidates(candidate_types_for_*())`.

### Step 2.4: Update `infer_reference` to emit candidates for constants

If reference points to a constant → emit `ResolvedType::Candidates(candidate_types_for_constant(&constant.value))`.
Otherwise → emit `ResolvedType::Definite(scope.get(&ref_.target)?)`.

### Step 2.5: Update `infer_unary` to filter candidates

- `Not` → `ResolvedType::Definite(Type::Bool)`
- Others → `ResolvedType::Candidates(candidate_types_for_unary(&inner_candidates, op))`
- If filtered set is empty → error, return None

### Step 2.6: Update `infer_binary` to use `find_common_type_sets`

- Get `ResolvedType` for both operands → `to_candidates()` → `find_common_type_sets(set_a, set_b)`
- For logical operators (`LogicalAnd/Or/Xor`): check truthiness via `is_truthy_candidate_set`, result is `Bool`
- For other operators: `find_common_type_sets` handles typed+literal, literal+literal, typed+typed uniformly
- If no common type → error, return None

### Step 2.7: Add `is_truthy_candidate_set` helper

Returns true if all candidates in the set are truthy types.

### Step 2.8: Update all tests in typecheck.rs

Update `get()` calls to use `as_type()`. Update assertions.

---

## Phase 3: validate_types.rs — Use ResolvedType

### Step 3.1: Update `get_expression_type` to work with `ResolvedType`

Walk expressions and look up resolved types from TypeEnv using sequential ID counter.
**CRITICAL**: All HashMap iterations MUST be sorted by key for deterministic ID assignment.
Walk order: signals (sorted) → timers (sorted) → transitions (sorted) → actions (sorted).

```rust
fn get_expression_resolved_type(
    expr: &Expression,
    type_env: &TypeEnv,
    scope: &VariableScope,
    id_counter: &mut usize,
) -> Option<ResolvedType> {
    let id = *id_counter;
    *id_counter += 1;
    type_env.get(&id).cloned()
}
```

### Step 3.2: Update `validate_assignment` to use `ResolvedType`

For assignment `x = expr`:
- Get target type from scope (always definite)
- Get expression resolved type from TypeEnv
- If expression is `ResolvedType::Definite(t)`: check `is_cast_lossless(t, target_type)`
- If expression is `ResolvedType::Candidates(candidates)`: check `candidates.contains(target_type)`
- If check fails: push `ConstLiteralTypeMismatch` error with candidate set info

### Step 3.3: Update `validate_signal_expression` similarly

### Step 3.4: Add `ConstLiteralTypeMismatch` to `CompileErrorKind`

```rust
ConstLiteralTypeMismatch,
```

With display: `"cannot assign value to target (no compatible type)"`.

### Step 3.5: Update validation tests

Add tests for:
- `42` assigned to `U8` → passes (U8 in candidate set)
- `-42` assigned to `U8` → fails (U8 not in {-42 candidates})
- `3.14` assigned to `U8` → fails (float → int never allowed)

---

## Phase 4: codegen.rs — Use ResolvedType, No Re-computation

### Step 4.1: Update `expr_to_rust` signature to accept `ResolvedEnv`

```rust
pub fn expr_to_rust(
    expr: &Expression,
    resolved_env: &TypeEnv, // ExpressionId → ResolvedType
    expected_type: Option<&Type>,
    field_accesses: &FieldAccessMap,
) -> Result<String, String>
```

Replace internal type resolution (`find_expr_type`) with `resolved_env` lookup.

### Step 4.2: Update `expr_to_rust` for `Expression::Value`

- Get resolved type from `resolved_env` (via sequential ID, same approach as validation)
- If `ResolvedType::Candidates(candidates)` and `expected_type` is Some:
  - Pick best candidate that matches or casts to expected_type
  - Use that type for the Rust literal suffix
- If `ResolvedType::Definite(t)`: use `t` directly
- Generate Rust literal with correct type suffix

### Step 4.3: Update `expr_to_rust` for `Expression::Reference`

- Look up resolved type from `resolved_env`
- If candidate set → pick best type for casting
- If definite type → use directly
- Generate cast if needed

### Step 4.4: Update `expr_to_rust` for `Expression::Binary`

- Look up resolved types for both operands from `resolved_env`
- Find common type from resolved types (use `find_common_type_sets`)
- Cast each operand to common type
- Generate: `(left as common) op (right as common)`

### Step 4.5: Update `expr_to_rust` for `Expression::Unary`

- Look up resolved type for inner expression
- Apply operator, cast result to resolved type if needed
- For `Negate`: inner candidates filtered to signed
- For `BitNot`: inner candidates filtered to unsigned
- For `Not`: result is Bool

### Step 4.6: Update `stmt_to_rust`

- Get target type from scope
- Pass target type as `expected_type` to `expr_to_rust`
- For compound assignment operators (+=, -=, etc.):
  - Generate: `y.target = y.target + <expr_casted_to_common>`
  - Common type between target and expression (use `find_common_type_sets`)

### Step 4.7: Update `build_tick_data` to pass `TypeEnv` to expr_to_rust

All calls to `expr_to_rust` and `stmt_to_rust` in `build_tick_data` need the `TypeEnv` parameter.

### Step 4.8: Update `build_types_data` for constants

For each constant, use `candidate_types_for_constant(&constant.value).best()` to get
the best type, then pass it to `value_to_literal_typed`. This ensures constants get
correct type suffixes (e.g., `10u8` instead of `10i64`) based on their value, not
declared type.

### Step 4.9: Remove `find_expr_type` function

No longer needed — resolved types come from `TypeEnv`.

### Step 4.10: Update `condition_to_rust` to accept `TypeEnv`

Update signature to accept `type_env: &TypeEnv` (no `expected_type` — conditions use
Rust truthiness, no casting needed). Walk the expression, look up resolved types from
`type_env` to generate correct literal suffixes. The resolved type gives the best type
for the literal (e.g., U8 for `42` in `flag && 42`).

---

## Phase 5: Compiler Integration

### Step 5.1: No changes needed to Compiler.compile

The `infer_all` function already returns `Vec<(TypeEnv, Vec<CompileError>)>`.
`validate_types` receives `type_envs` from `infer_all`.
`backend.compile` receives `machines` — we'll pass the `TypeEnv` through the `RustBackend::compile` signature.

### Step 5.2: Update `RustBackend::compile` signature

Add `type_envs` parameter:
```rust
fn compile(&self, machines: &[StateMachine], type_envs: &[(TypeEnv, Vec<CompileError>)]) -> Result<FileSet, Vec<CompileError>>;
```

### Step 5.3: Update `Compiler.compile` to pass `type_envs` to backend

```rust
self.backend.compile(machines, &type_results)
```

### Step 5.4: Update `RustBackend::compile` implementation

In `backend/rust/mod.rs`, pass `type_envs` to `build_tick_data` and `build_types_data`.

---

## Phase 6: Tests

### Step 6.1: Update existing creator tests

- `type_casting.rs`: `42` → `U8` should now pass
- `arithmetic_ops.rs`: verify binary operations still work
- `type_system.rs`: verify type checking still works

### Step 6.2: Add new test for literal assignment

```rust
#[test]
fn test_literal_to_typed_assignment() {
    // 42 assigned to U8 should succeed
    // -42 assigned to U8 should fail
    // 3.14 assigned to U8 should fail
}
```

### Step 6.3: Add new test for binary literal operations

```rust
#[test]
fn test_binary_literal_common_type() {
    // U8 + 42 → common type between U8 and {all ints}
    // -42 + U8 → common type between {signed} and {unsigned}
}
```

### Step 6.4: Verify compilation and test results

```bash
cargo test
cargo check --bin creator
cargo check --bin runner
```

---

## Key Design Decisions

1. **No separate resolve pass** — `TypeEnv` stores `ResolvedType` directly. Codegen and validation access via `as_type()` and `to_candidates()`.

2. **Language-independent** — `typecheck_rules` contains all type logic. `codegen.rs` only generates Rust code, no type resolution.

3. **Uniform candidate set approach** — binary ops always compute sets for both operands. No special case for typed+literal.

4. **Constants ignore type field** — constants use value-based candidates, same as literals.

5. **Float literals never cast to integer** — {F32, F64} ∩ {integer types} = ∅ → error.

6. **Error messages include candidate set** — `ConstLiteralTypeMismatch` shows candidates for debugging.
