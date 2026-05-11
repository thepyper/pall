# Test Machines List

Machine groups to add for comprehensive testing of all StateMachine features.
Simple → Complex ordering. One group = one file pair (creator + runner).

---

## Group 1: Simple State Transitions ✅ COMPLETO

### 1a. traffic_light ✅ COMPLETED
- **Purpose**: Basic state cycle with named transitions
- **Features**: 3+ states, always-true transitions, state name checks
- **States**: red → yellow → green → red (cycle)
- **Tests**: Reaches "red" state (cycle verified), 3 states visited in order

### 1b. binary_counter ✅ COMPLETED
- **Purpose**: Counter with binary representation (two bits)
- **Features**: I64 variable, increment, transition on condition
- **States**: idle → counting (counter increments, transitions when counter >= 4)
- **Tests**: Goal reached at counter=4, correct tick count

### 1c. conditional_action ✅ COMPLETED
- **Purpose**: Actions with `when` conditions
- **Features**: Action that only executes conditionally, state depends on action outcome
- **States**: setup → work (action increments when counter < 5) → done
- **Tests**: Counter reaches expected value based on action condition

---

## Group 2: Arithmetic Expressions ✅ COMPLETO

### 2a. arithmetic_ops ✅ COMPLETED
- **Purpose**: Test all arithmetic operators (+, -, *, /, %)
- **Features**: Variables updated with different operators
- **States**: start → compute (various arithmetic operations) → done
- **Tests**: Final variable values match expected arithmetic results

### 2b. assignment_ops ✅ COMPLETED
- **Purpose**: Test all assignment operators (=, +=, -=, *=, /=, %=)
- **Features**: Each operator tested on separate variable
- **States**: start → assign_ops → done
- **Tests**: Each variable has correct value after assignment

---

## Group 3: Logical and Bitwise Operators ✅ COMPLETO

### 3a. logic_ops ✅ COMPLETED
- **Purpose**: Test logical operators (&&, ||, ^^, !)
- **Features**: Conditions combining multiple logical operators
- **States**: start → check (logical expressions) → done
- **Tests**: Correct decision based on logical expression evaluation

### 3b. bitwise_ops ✅ COMPLETED
- **Purpose**: Test bitwise operators (&, |, ^, ~)
- **Features**: Bit manipulation on integer variables
- **States**: start → manipulate (bitwise operations) → done
- **Tests**: Final bit patterns match expected results

### 3c. expression_precedence ✅ COMPLETED
- **Purpose**: Test operator precedence and parentheses
- **Features**: Complex expressions with mixed operators, parentheses override
- **States**: start → calculate → done
- **Tests**: Final value matches expected result of precedence-resolved expression

---

## Group 4: Type Casting ✅ COMPLETO

### 4a. type_casting ✅ COMPLETED
- **Purpose**: Test implicit type casting in expressions and assignments
- **Features**:
  - Common type resolution (U8 + U16 → U16, I8 + U16 → I32)
  - Unsigned priority in tie-breaking
  - Int-to-float casting limits
  - Literal type inference (I64 default, U64 for overflow)
  - Assignment widening (U8 → U32 ✅) vs narrowing (U32 → U8 ❌)
  - Truthiness (C++-style: 0=false, non-zero=true)
  - Operator-type compatibility (arithmetic requires numeric, bitwise requires integer)
  - Bool restrictions (no arithmetic, only logical/equality)
- **States**: start → cast_ops → mixed_ops → truthiness → done
- **Tests**:
  - U8 + U16 generates `(a as u16) + b`
  - I8 + U16 generates `(a as i32) + (b as i32)`
  - I32 + U32 uses unsigned priority (U32)
  - I16 → F32 allowed, I64 → F64 NOT allowed
  - Literal 42 has type I64
  - Assignment widening accepted, narrowing rejected
  - Truthiness: numeric non-zero → true, zero → false
  - Bool + numeric arithmetic → error

---

## Group 5: Type System ⬜

### 5a. type_system
- **Purpose**: Test all variable types (Bool, U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, String)
- **Features**: All types with correct values
- **States**: start → type_test → done
- **Tests**: All type variables have correct final values

---

## Group 6: Signals and Timers ⬜

### 6a. signals_and_timers
- **Purpose**: Test signals (computed expressions) and timers
- **Features**: Signal from expression, timer accumulation with/without condition
- **States**: start → compute → done
- **Tests**: Signal values correct, timer values correct

---

## Group 7: Constants and Inputs ⬜

### 7a. constants_and_inputs
- **Purpose**: Test constants (pub const) and inputs
- **Features**: Machine with constants, inputs that are read-only
- **States**: start → use_constants → done
- **Tests**: Constants used correctly in expressions, inputs not modified

---

## Group 8: Multi-Machine Groups with Links ⬜

### 8a. linked_machines
- **Purpose**: Two machines linked together
- **Features**: Input → Link propagation, multi-machine group tick
- **Machines**: counter + controller (link counter.output → controller.input)
- **States**: Each machine has its own state machine
- **Tests**: Link propagates values correctly between machines

---

## Group 9: Edge Cases and Complex Features ⬜

### 9a. edge_cases
- **Purpose**: Edge cases: empty states, multiple transitions same condition, nested expressions
- **Features**: Deceiving transitions (same condition, only first fires)
- **States**: Various edge case scenarios
- **Tests**: Only expected transitions fire, no error states reached

---

## Test Count Summary

| Group | Machines | Creator Tests | Runner Tests | Total |
|-------|----------|---------------|--------------|-------|
| 1. State Transitions | 3 | 6 | 9 | 15 |
| 2. Arithmetic | 2 | 4 | 6 | 10 |
| 3. Logic & Bitwise | 3 | 6 | 9 | 15 |
| 4. Type Casting | 1 | 2 | 3 | 5 |
| 5. Type System | 0 | — | — | — |
| 6. Signals & Timers | 0 | — | — | — |
| 7. Constants & Inputs | 0 | — | — | — |
| 8. Multi-Machine | 0 | — | — | — |
| 9. Edge Cases | 0 | — | — | — |
| **Totals** | **9** | **18** | **27** | **45** |

Plus: 43 lib tests (parser + validation)
**Total tests: 88**

---

## Execution Order

Execute groups in order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9
Each group: creator file + runner file → test → commit → next group.
