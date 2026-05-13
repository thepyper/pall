# Interview: Reference Validation Pass

## Objective

Implement a new validation pass that ensures every variable reference in expressions and statements corresponds to an actually-defined variable in the machine. The pass should produce well-explaining, contextual error messages, consistent with other validation passes in the compiler.

## Key Decisions

### What the pass validates

1. **Unknown variable references**: When a `Reference` expression (e.g., `counter > 5`, `x = y + z`) references a name that does not exist as any of: variable, signal, timer, constant, or input → report `UnknownVariableReference` error.

2. **Invalid assignment targets**: When the target of an assignment statement is not a variable → report `InvalidAssignmentTarget` error. Only variables can be assignment targets. Signals, timers, constants, and inputs are read-only.

3. **Reserved name exception**: References to "state" (the internally defined state variable) are valid — they should NOT trigger an unknown reference error. This should use a helper function to allow future reserved names to be added easily.

### Error kinds (distinct, non-overlapping)

- **`UnknownVariableReference`**: A name is referenced but does not exist as any machine field (variable, signal, timer, constant, input), and is not a reserved internal name.
- **`InvalidAssignmentTarget`**: An assignment target references something that is not a variable (e.g., a signal, timer, constant, or input).

### Error message format

Consistent with existing validation errors:
```
unknown variable reference 'foo' in machine 'X', state 'Y', action Z: 'foo' is not defined
unknown variable reference 'foo' in machine 'X', state 'Y', transition Z: 'foo' is not defined
invalid assignment target 'bar' in machine 'X', state 'Y', action Z: 'bar' is not a variable
invalid assignment target 'bar' in machine 'X', state 'Y', transition Z: 'bar' is not a variable
```

### Existing behavior retained

- `"when": null` in YAML correctly maps to `Option::None` (no condition = always true). This is correct behavior — no changes needed.
- Type inference in `typecheck.rs` already catches unknown references generically. The new pass will provide **context-rich** errors.
- `"when": true` and `when` omitted are both valid ways to have always-true transitions/actions.

### Architecture

- **New file**: `src/compiler/reference_validation.rs` — dedicated pass for reference validation.
- **Existing file changes**:
  - `error.rs` — add `UnknownVariableReference` and `InvalidAssignmentTarget` variants.
  - `validation.rs` — call the new reference validation pass.
  - `type_validation.rs` — update `get_target_type` to reject non-variable assignment targets (or rely on the new pass).
- **Helper function**: `is_reserved_internal_name(name: &str) -> bool` in the new file, currently returning `true` only for `"state"`.

### Scope of validation

The pass must check references in:
- Signal expressions
- Timer `when` expressions
- Transition `when` conditions and `do` statements
- Action `when` conditions and `do` statements
- Assignment targets in all of the above
