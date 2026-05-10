# Test Machines List

Machine groups to add for comprehensive testing of all StateMachine features.
Simple → Complex ordering. One group = one file pair (creator + runner).

---

## Group 1: Simple State Transitions

### 1a. traffic_light ✅ COMPLETED
- **Purpose**: Basic state cycle with named transitions
- **Features**: 3+ states, always-true transitions, state name checks
- **States**: red → yellow → green → red (cycle)
- **Tests**: Reaches "red" state (cycle verified), 3 states visited in order

### 1b. binary_counter ⬜
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

## Group 2: Arithmetic Expressions

### 2a. arithmetic_ops ⬜

---

## Group 2: Arithmetic Expressions

### 2a. arithmetic_ops
- **Purpose**: Test all arithmetic operators (+, -, *, /, %)
- **Features**: Variables updated with different operators
- **States**: start → compute (various arithmetic operations) → done
- **Tests**: Final variable values match expected arithmetic results

### 2b. assignment_ops
- **Purpose**: Test all assignment operators (=, +=, -=, *=, /=, %=)
- **Features**: Each operator tested on separate variable
- **States**: start → assign_ops → done
- **Tests**: Each variable has correct value after assignment

---

## Group 3: Logical and Bitwise Operators ✅ COMPLETED

### 3a. logic_ops
- **Purpose**: Test logical operators (&&, ||, ^^, !)
- **Features**: Conditions combining multiple logical operators
- **States**: start → check (logical expressions) → done
- **Tests**: Correct decision based on logical expression evaluation

### 3b. bitwise_ops
- **Purpose**: Test bitwise operators (&, |, ^, ~)
- **Features**: Bit manipulation on integer variables
- **States**: start → manipulate (bitwise operations) → done
- **Tests**: Final bit patterns match expected results

### 3c. expression_precedence
- **Purpose**: Test operator precedence and parentheses
- **Features**: Complex expressions with mixed operators, parentheses override
- **States**: start → calculate → done
- **Tests**: Final value matches expected result of precedence-resolved expression

---

## Group 4: Type System

### 4a. type_system
- **Purpose**: Test all variable types
- **Features**: Bool, U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, String
- **States**: start → type_test → done
- **Tests**: All type variables have correct final values

---

## Group 5: Signals and Timers

### 5a. signals_and_timers
- **Purpose**: Test signals (computed expressions) and timers
- **Features**: Signal from expression, timer accumulation with/without condition
- **States**: start → compute → done
- **Tests**: Signal values correct, timer values correct

---

## Group 6: Constants and Inputs

### 6a. constants_and_inputs
- **Purpose**: Test constants (pub const) and inputs
- **Features**: Machine with constants, inputs that are read-only
- **States**: start → use_constants → done
- **Tests**: Constants used correctly in expressions, inputs not modified

---

## Group 7: Multi-Machine Groups with Links

### 7a. linked_machines
- **Purpose**: Two machines linked together
- **Features**: Input → Link propagation, multi-machine group tick
- **Machines**: counter + controller (link counter.output → controller.input)
- **States**: Each machine has its own state machine
- **Tests**: Link propagates values correctly between machines

---

## Group 8: Edge Cases and Complex Features

### 8a. edge_cases
- **Purpose**: Edge cases: empty states, multiple transitions same condition, nested expressions
- **Features**: Deceiving transitions (same condition, only first fires)
- **States**: Various edge case scenarios
- **Tests**: Only expected transitions fire, no error states reached

---

## Execution Order

Execute groups in order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8
Each group: creator file + runner file → test → commit → next group.
