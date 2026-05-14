# Plan: Reference Validation Pass

## Phase 1: Infrastructure — new error kinds and helper

### Step 1.1: Add new CompileErrorKind variants
- **File**: `src/compiler/error.rs`
- **Action**: Add `UnknownVariableReference` and `InvalidAssignmentTarget` to the `CompileErrorKind` enum, with Display implementations.

### Step 1.2: Create reference_validation.rs with helper function
- **File**: `src/compiler/reference_validation.rs` (new)
- **Action**: Create file with `is_reserved_internal_name(name: &str) -> bool` helper returning `true` for "state" only.

### Step 1.3: Register module in compiler mod.rs
- **File**: `src/compiler/mod.rs`
- **Action**: Add `pub mod reference_validation;` declaration.

## Phase 2: Implement the reference validation pass

### Step 2.1: Implement unknown reference check for expressions
- **File**: `src/compiler/reference_validation.rs`
- **Action**: Add `check_reference_exists(name: &str, scope: &VariableScope) -> bool` that checks scope AND reserved names.
- **Action**: Add `check_expression_refs(expr: &Expression, scope: &VariableScope, machine_id: &str, context_name: &str, context_type: &str, errors: &mut Vec<CompileError>)` that walks the expression tree and reports unknown references.

### Step 2.2: Implement invalid assignment target check
- **File**: `src/compiler/reference_validation.rs`
- **Action**: Add `is_valid_assignment_target(target: &str, variables: &HashMap<String, Variable>) -> bool` that checks if target is a variable name (not signal, timer, constant, or input).
- **Action**: Add `validate_assignment_target(stmt: &FullStatement, variables: &HashMap<String, Variable>, machine_id: &str, state_name: &str, context: &str, errors: &mut Vec<CompileError>)` that reports `InvalidAssignmentTarget` if target is not a variable.

### Step 2.3: Implement the main validate_references function
- **File**: `src/compiler/reference_validation.rs`
- **Action**: Add `validate_references(machine: &StateMachine) -> Vec<CompileError>` that walks all expressions (signal, timer, transitions, actions) calling `check_expression_refs` for each, and validates assignment targets.

## Phase 3: Integrate into compiler pipeline

### Step 3.1: Wire new pass into compile()
- **File**: `src/compiler/mod.rs`
- **Action**: Add call to `validate_references(machines)` in the `compile()` method. Place it as a new Phase 2a, between type inference and original validation (or as a sub-phase of validation). The pass should run early — before type inference, since it's a structural check. Actually, since type inference already builds scope, place it right after type inference errors are checked.

### Step 3.2: Add tests for new pass
- **File**: `src/compiler/reference_validation.rs` (test module)
- **Action**: Add tests:
  - Valid reference passes
  - Unknown variable reference in expression
  - Unknown variable reference in assignment target
  - Signal as assignment target (invalid)
  - Timer as assignment target (invalid)
  - Input as assignment target (invalid)
  - Constant as assignment target (invalid)
  - "state" reference does NOT produce error (reserved)
  - Unknown reference in transition when
  - Unknown reference in action do
  - Multiple errors reported at once

## Phase 4: Clean up existing duplicate/error-prone code

### Step 4.1: Remove duplicate unknown reference error from typecheck
- **File**: `src/compiler/typecheck.rs`
- **Action**: In `infer_reference()`, remove the error push for unknown references. The new reference validation pass provides richer, contextual errors. This avoids duplicate error messages.

### Step 4.2: Verified type_validation.rs needs no changes
- **File**: `src/compiler/type_validation.rs`
- **Result**: No changes needed. The existing `get_target_type` already returns `None` for unknown targets, and `validate_assignment` skips validation when `target_type` is `None`. This is correct behavior — the new reference validation pass handles both unknown references and invalid assignment targets.

## Phase 5: Final verification

### Step 5.1: Run all existing tests ✅
- **Result**: 103 library tests pass. No regressions.

### Step 5.2: Run the full compilation pipeline on all test machines ✅
- **Result**: 21 creator tests pass (all compilation + YAML roundtrip tests). 34 runner tests pass (all runtime tests). All test machines compile successfully with the new validation pass integrated.
