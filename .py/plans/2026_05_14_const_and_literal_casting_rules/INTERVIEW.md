# Interview Summary: Const and Literal Casting Rules

## Objective

Make constants and literals more flexible in casting than variables and other typed values. The compiler knows the actual value of a constant or literal at compile time, so it can determine whether a cast is lossless based on the **value**, not just the **type**.

## Current Problem

- Integer literals always default to `I64` (or `U64` for values exceeding i64 range)
- `42` assigned to `U8` fails because `I64 → U8` is forbidden by the signed→unsigned casting rule
- The compiler rejects valid assignments where the actual value fits but the default type doesn't cast
- Constants have a `type` field in YAML that restricts their usage unnecessarily
- Code generation re-computes type information that was already determined during type inference/validation

## Decisions Made

### 1. Candidate Sets for Literals

Every literal gets a **candidate set** — all types that can hold the literal's value without loss:

| Literal | Candidate Set |
|---------|--------------|
| `42` | `{U8, I8, U16, I16, U32, I32, U64, I64, F32, F64}` |
| `-42` | `{I8, I16, I32, I64, F32, F64}` (signed only — can't fit in unsigned) |
| `-10000` | `{I16, I32, I64, F32, F64}` |
| `3.14` | `{F32, F64}` |
| `true` | `{Bool}` |
| `true` in bool context | `{Bool}` |

### 2. Binary Operations: Candidate Set Intersection

For **every** binary operation, compute candidate sets for **both** operands, then find the best common type:

- **Both typed (variables):** Each set = `{single_type}`. Common type found via intersection (reduces to existing `find_common_type`).
- **Typed + literal:** Typed set = `{single_type}`, literal set = `{candidates}`. Intersection gives compatible types; smallest = common type.
- **Both literals:** Both sets are candidates; intersection gives best common type.

This is a uniform approach — no special cases for "typed + literal."

### 3. Unary Operators — Candidate Filtering

Unary operators filter the inner expression's candidate set:

| Operator | Filter (result candidate set) |
|----------|------------------------------|
| `-` (negate) | `{I8, I16, I32, I64, F32, F64}` — signed types only |
| `~` (bitnot) | `{U8, U16, U32, U64}` — unsigned integers only |
| `!` (logical not) | Always `{Bool}` — result is always boolean |

For `-42`: inner `42` gives `{all types}`, filter with signed types → `{I8, I16, I32, I64, F32, F64}`.

### 4. Assignment Validation

- **Typed target + typed expression:** Use existing `is_cast_lossless(type1, type2)` — unchanged.
- **Typed target + literal/const:** Check if target type is **in** the literal's/const's candidate set. If yes → OK. If no → error.
- **Constant assignment:** Same as literal — check if target type is in candidate set.
- **New error kind:** `ConstLiteralTypeMismatch` — distinct from `InvalidSignalExpr`. Message includes the candidate set.

### 5. Constants — Type Field Ignored (For Now)

- Keep the `type` field in YAML for backward compatibility.
- **Do not use it** — treat constants as typeless, derive their type from their value.
- Constants behave like literals in all casting contexts.
- Decision to remove `type` field entirely is deferred to a future plan.

### 6. Float Literals — No Float→Integer Casts

Float literals (`3.14`) have candidate set `{F32, F64}`. **No float → integer casting is ever allowed**, even for literals, because it would be lossy.

In binary operations with integer operands, `find_best_common_type({F32, F64}, {U8, ...})` returns **nothing** (no intersection), producing a type error. This is correct behavior.

### 7. Refactoring `int_to_int_lossless` — Match-Based, Not Bit-Based

The current `int_to_int_lossless` function uses bit-count arithmetic which is obscure. Replace with explicit type-matching:

```
match (from, to) {
    (same type) → true,
    // Unsigned → larger unsigned
    (U8, U16|U32|U64) → true,
    (U16, U32|U64) → true,
    (U32, U64) → true,
    // Unsigned → larger signed (target must hold all source values)
    (U8, I16|I32|I64) → true,
    (U16, I32|I64) → true,
    (U32, I64) → true,
    // Signed → larger signed
    (I8, I16|I32|I64) → true,
    (I16, I32|I64) → true,
    (I32, I64) → true,
    // Never: signed → unsigned
    (I*, U*) → false,
    // Catch all: same bit width different sign → false
    _ → false,
}
```

### 8. Value-Aware Casting

New function `is_cast_lossless_value(value, target_type)` for literals/constants:
- Check if the target type is in the value's candidate set (computed from value + operator constraints)
- This subsumes the type-based `is_cast_lossless` for values — it's more precise

### 9. Architecture: TypeEnv with ResolvedType (No Separate Resolve Pass)

**Current problem:** codegen.rs recomputes types via `find_expr_type()` — duplicating logic already done during type inference.

**Solution:** Extend `TypeEnv` to store `ResolvedType` (not just `Type`). Both validation and codegen access resolved types via `as_type()` and `to_candidates()`. No separate resolve pass needed.

```infer_all (TypeEnv<ResolvedType>) → validate_types (uses TypeEnv) → codegen (uses TypeEnv)```

**ID assignment**: Sequential ExpressionIds assigned during typecheck walk. Validation and codegen replay the walk in the SAME order to get matching IDs.

**Separation of concerns:**
- `typecheck_rules` — language-independent type logic (candidate sets, casting rules, common type resolution). No Rust-specific code.
- `typecheck` — type inference (assigning ExpressionIds, walking AST). Language-independent.
- `resolve` — resolve candidate sets to concrete types. Language-independent.
- `validation` — check type compatibility. Language-independent.
- `codegen.rs` (Rust backend) — **no type resolution**. Uses resolved types directly. Only generates Rust code.

### 10. Data Structures

**ResolvedType:**
- `Definite(Type)` — single resolved type (for references, operations)
- `Candidates(CandidateSet)` — candidate set (for literals, constants, unary ops)

**CandidateSet:**
- `Vec<Type>` — ordered list of candidate types, smallest first

**TypeEnv** (extended):
- `HashMap<ExpressionId, ResolvedType>` — was `HashMap<ExpressionId, Type>`, now stores ResolvedType
- Validation and codegen look up resolved types by replaying the AST walk with sequential IDs

### 11. Truthiness

No changes needed. Numeric types (including literals/constants) are already truthy per existing rules. The `is_truthy_type` function already handles this.

### 12. Backward Compatibility

- No concern for breaking existing YAML — this is a compiler being crafted, not a user-facing product.
- Existing machines using `42` assigned to `U8` will now **work** (previously failed).
- Existing machines using `3.14` assigned to `F64` will continue to work (F64 is in candidate set).
- API changes within the crate are acceptable.

### 13. Error Messages

New error kind `ConstLiteralTypeMismatch` with message format:
```
cannot assign literal value <value> to <target_type>: no compatible type found
  literal candidates: {candidate_set}
```

This gives users immediate insight into why a literal couldn't be assigned.

### 10. Deterministic HashMap Key Ordering

**CRITICAL**: All HashMap iterations that assign sequential ExpressionIds must be sorted by key.
This applies to typecheck, validation, and codegen — ALL THREE must iterate in the same order.

In typecheck (`infer_all_expressions`):
- `signals.keys().sorted()`
- `timers.keys().sorted()`
- `states.keys().sorted()`

In validation and codegen: same sorted iteration pattern.

This ensures ExpressionIds match across all three phases.

### 11. Unary Operator - (Negate) and ~ (BitNot)

- `-42` → candidate set filtered to signed types `{I8, I16, I32, I64, F32, F64}`
- `~5` → candidate set filtered to unsigned integers `{U8, U16, U32, U64}`
- These are **operator-level constraints**, applied after computing the inner value's candidate set

## Questions Answered

Q: Literal type inference?
A: Any literal gets a full candidate set. Binary ops compute sets for both operands and find common type via intersection.

Q: Constants — keep or remove type?
A: Keep the YAML field, ignore it. Remove decision deferred.

Q: Binary operation priority?
A: Uniform candidate set intersection for all cases. No special typed+literal case.

Q: Assignment with literals?
A: Works if target type is in the literal's candidate set.

Q: Truthiness?
A: No changes — numeric types are already truthy.

Q: Scope?
A: Both typecheck and codegen. Plus refactoring int_to_int_lossless.

Q: Constants type handling?
A: Keep field, ignore it. Assignment works if target in candidate set.

Q: Signedness preference?
A: Both signed and unsigned in candidate set. Context (other operand) decides.

Q: Float+integer?
A: No float→integer cast allowed. Candidate sets don't intersect → error.

Q: Unary operators?
A: `-` filters to signed, `~` filters to unsigned ints, `!` → Bool.

Q: Codegen recomputing types?
A: Not acceptable. TypeEnv stores ResolvedType. Codegen looks up resolved types via sequential ID walk, no computation.

Q: Language separation?
A: typecheck_rules = language-independent (candidate sets, casting rules, common type resolution). codegen = Rust-specific (generates Rust code only, uses resolved types from TypeEnv).

Q: int_to_int_lossless refactoring?
A: Replace bit-count logic with explicit match-based type table.

Q: Codegen type resolution?
A: TypeEnv contains all ResolvedTypes. Codegen looks up via expression walk with sequential IDs, no computation.
