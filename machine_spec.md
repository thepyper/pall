# Pall Machine Specification

## Overview

Pall is a state machine compiler that generates Rust code from machine definitions. A machine is a deterministic, synchronous state machine with states, variables, inputs, signals, and timers. Machines are compiled into Rust modules containing persistent state structs and tick functions.

## Machine Format (YAML)

### Root Fields (YAML)

The root of the YAML document contains the machine fields directly — **no `StateMachine` wrapper key is used**. Serde deserializes struct fields at the document root.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `string` | Yes | — | Unique machine identifier (Rust identifier) |
| `initial` | `string` | No | `"initial"` | Name of the initial state |
| `states` | `map<string, State>` | Yes | — | State definitions |
| `inputs` | `map<string, Input>` | No | `{}` | External inputs |
| `signals` | `map<string, Signal>` | No | `{}` | Computed output signals |
| `timers` | `map<string, Timer>` | No | `{}` | Timers |
| `variables` | `map<string, Variable>` | No | `{}` | Persistent variables |
| `constants` | `map<string, Constant>` | No | `{}` | Compile-time constants |

**Example YAML:**
```yaml
id: my_machine
initial: start
variables:
  counter:
    type: I64
    initial: 0
states:
  start:
    transitions:
      - when: null
        do: []
        target: running
  running:
    actions:
      - when: null
        do:
          - counter += 1
    transitions:
      - when: counter >= 10
        do: []
        target: stopped
  stopped:
    actions: []
    transitions: []
```

### State

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `actions` | `Action[]` | No | `[]` | Actions executed in this state |
| `transitions` | `Transition[]` | No | `[]` | State transitions |

States are visited via a `match` on the current `state` enum variant. Actions and transitions are evaluated in declaration order.

### Transition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `when` | `expression \| null` | No | `null` | Condition for transition. `null` means always-true. |
| `do` | `statement[]` | No | `[]` | Statements executed during transition |
| `target` | `string` | Yes | — | Name of the state to transition to |

Transitions are evaluated in order. The first matching transition wins. When a transition fires, it applies its `do` statements and sets the target state immediately.

### Action

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `when` | `expression \| null` | No | `null` | Condition for action. `null` means always execute. |
| `do` | `statement[]` | No | `[]` | Statements executed when condition is true |

Actions are executed before transitions within each state.

### Variable

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | `Type` | Yes | — | Variable type |
| `initial` | `Value` | No | — | Initial value (used in `init()`) |
| `output` | `bool` | No | `false` | Whether this variable is exposed as an output |

### Input

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | `Type` | Yes | — | Input type |
| `link` | `Link` | No | `null` | Link from another machine's output |
| `output` | `bool` | No | `false` | Whether this input is also an output |

In multi-machine (group) mode, link propagation occurs before per-machine ticks.

### Signal

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | `Type` | Yes | — | Signal type |
| `output` | `bool` | No | `false` | Whether this signal is exposed as an output |
| `expr` | `Expression` | Yes | — | Expression to compute the signal value |

Signals are computed after all state/transition logic and assigned to the `Persistent` state.

### Timer

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | `Type` | Yes | — | Timer type (typically a numeric type) |
| `when` | `expression \| null` | No | `null` | Condition for timer accumulation. `null` means always accumulate. |

Timers accumulate `delta_ms` when `when` is true, and reset to 0 otherwise.

### Constant

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | `Type` | Yes | — | Constant type |
| `output` | `bool` | No | `false` | Whether this constant is exposed as an output |
| `value` | `Value` | Yes | — | Constant value |

Constants are generated as Rust `pub const` values in the types module.

## Type System

### Type Enum

| Variant | Rust Type | Description |
|---------|-----------|-------------|
| `Bool` | `bool` | Boolean |
| `U8` | `u8` | Unsigned 8-bit integer |
| `U16` | `u16` | Unsigned 16-bit integer |
| `U32` | `u32` | Unsigned 32-bit integer |
| `U64` | `u64` | Unsigned 64-bit integer |
| `I8` | `i8` | Signed 8-bit integer |
| `I16` | `i16` | Signed 16-bit integer |
| `I32` | `i32` | Signed 32-bit integer |
| `I64` | `i64` | Signed 64-bit integer |
| `F32` | `f32` | 32-bit float |
| `F64` | `f64` | 64-bit float |
| `String` | `String` | String type |

### Implicit Type Casting

The Pall compiler supports **implicit (automatic) type casting** as part of its type system. All casting is **lossless** — the compiler will reject any cast that would lose precision or change the semantic meaning of a value.

#### Casting Rules

Casting is only permitted to target types that **fully contain** the source type without precision or sign loss.

**Allowed casts:**

| From | To (permitted) |
|------|----------------|
| `Bool` | `U8`, `U16`, `U32`, `U64`, `I8`, `I16`, `I32`, `I64`, `F32`, `F64` |
| `U8` | `U16`, `U32`, `U64`, `I16`, `I32`, `I64` |
| `U16` | `U32`, `U64`, `I32`, `I64` |
| `U32` | `U64`, `I64` |
| `U64` | `I64` |
| `I8` | `I16`, `I32`, `I64` |
| `I16` | `I32`, `I64` |
| `I32` | `I64` |
| `F32` | `F64` |

**Never allowed:**

| Cast | Reason |
|------|--------|
| Any signed → unsigned | Sign conversion is not lossless (e.g., `I8 → U8`, `I32 → U32`) |
| Any float → integer | Precision loss (e.g., `F32 → U8`) |
| `I64`/`U64` → `F64` | Mantissa overflow (>52 bits cannot be represented exactly in f64) |
| Any type → `String` | No automatic string conversion |

#### Common Type Resolution

When two operands of different types are combined in a binary operation (`+`, `-`, `*`, `/`, `%`, `&`, `\|`, `^`, `==`, `!=`, `<`, `<=`, `>`, `>=`), the compiler **automatically** finds a common type that both operands can cast to, using this algorithm:

1. **Find all candidates** — types that both operands can losslessly cast to
2. **Select the smallest** — pick the candidate with the fewest bits
3. **Tie-break with unsigned priority** — if two candidates have the same bit count (e.g., `U32` and `I32`), prefer the unsigned one

**Examples:**

| Expression | Operand Types | Common Type | Generated Code |
|-----------|---------------|-------------|----------------|
| `a + b` | `U8 + U16` | `U16` | `(a as u16) + b` |
| `a + b` | `U8 + U32` | `U32` | `(a as u32) + b` |
| `a + b` | `I8 + U16` | `I32` | `(a as i32) + (b as i32)` |
| `a + b` | `U32 + I64` | `I64` | `(a as i64) + b` |
| `a + b` | `I32 + U32` | `U32` | `(a as u32) + b` *(unsigned priority)* |

**Note:** In `I32 + U32`, unsigned `U32` is preferred because both operands are unsigned-compatible and `U32` is smaller than `I64`.

#### Int-to-Float Casting Rules

Integer-to-float casting is **conditionally permitted** based on mantissa precision:

| Cast | Allowed? | Reason |
|------|----------|--------|
| `U8`/`I8` → `F32` | ✅ | 8 bits fit in f32's 24-bit mantissa |
| `U16`/`I16` → `F32` | ✅ | 16 bits fit in f32's 24-bit mantissa |
| `U32`/`I32` → `F64` | ✅ | 32 bits fit in f64's 52-bit mantissa |
| `U64`/`I64` → `F64` | ❌ | 64 bits exceed f64's 52-bit mantissa |

#### Literal Types

Literal values have implicit default types that influence common type resolution:

| Literal | Default Type | Notes |
|---------|-------------|-------|
| `42`, `-5`, `0` | `I64` | Signed 64-bit integer |
| `18446744073709551615` (i64 overflow) | `U64` | Unsigned 64-bit when value exceeds i64 range |
| `3.14`, `2.0E-3` | `F64` | 64-bit float |
| `true`, `false` | `Bool` | Boolean |
| `"hello"` | `String` | String |

**Example:** `counter: U8 = 100` — literal `100` has type `I64`, which casts to `U8` (since `I64 → U8` is **not** allowed due to signed→unsigned rule).

> **Important:** Literal signed values cannot be assigned to unsigned variables via implicit cast. Use explicit unsigned literals or variables of matching signedness.

#### Truthiness (C++-Style)

In contexts requiring a boolean value (`when` conditions, `&&`, `\|\|`, `^^`), Pall uses **C++-style truthiness**:

- `0` → `false`
- Non-zero → `true`

This applies to **both** `Bool` values and numeric types.

**Implications:**

| Expression | Valid? | Notes |
|-----------|--------|-------|
| `when: counter > 0` | ✅ | `>` returns `Bool` |
| `when: counter` | ✅ | `counter` (numeric) resolves to boolean via truthiness |
| `when: flag && counter` | ✅ | `flag` (Bool) AND `counter` (numeric, truthy) |
| `when: "hello"` | ❌ | `String` is not truthy |

**Key distinction:** Truthiness is **resolution**, not casting. The numeric value is checked for zero/non-zero at runtime, but no type conversion occurs in the AST.

#### Operator-Type Compatibility

Each operator class has specific type requirements:

| Operator Class | Operators | Operand Types | Result Type |
|---------------|-----------|---------------|-------------|
| **Arithmetic** | `+`, `-`, `*`, `/`, `%` | Both numeric | Common numeric type |
| **Bitwise** | `&`, `\|`, `^` | Both integer (not float, not Bool) | Common integer type |
| **Logical** | `&&`, `\|\|`, `^^` | Both truthy (Bool or numeric) | `Bool` |
| **Equality** | `==`, `!=` | Any type (cast to common if different) | `Bool` |
| **Ordering** | `<`, `<=`, `>`, `>=` | Both numeric | `Bool` |

**Prohibited combinations:**

| Operation | Reason |
|-----------|--------|
| `true + 5` | Bool not allowed in arithmetic |
| `true < 5` | Bool not allowed in ordering |
| `a & b` where `a:F32` | Bitwise requires integer |
| `a || "text"` | String not truthy |

#### Assignment Type Checking

When assigning an expression to a variable, the compiler verifies that the expression's type can be **losslessly cast** to the variable's type:

```yaml
variables:
  x:
    type: U8
  y:
    type: U32

states:
  initial:
    transitions:
      - target: final
        when: null
        do:
          - y = x    # ✅ OK: U8 → U32 (widening)
          - x = y    # ❌ ERROR: U32 → U8 (lossy cast)
```

#### Boolean Restrictions

`Bool` values have limited usage:

- **Allowed in:** `==`, `!=`, `&&`, `\|\|`, `^^`, `!`
- **Allowed in:** `when` conditions (both Bool and truthy numeric)
- **NOT allowed in:** `+`, `-`, `*`, `/`, `%`, `<`, `<=`, `>`, `>=`, `&`, `\|`, `^`

### Value

The `Value` enum represents literal values. In YAML, `Value` accepts **both**:

1. **Plain values** (simplified): A raw number or string is automatically wrapped.
   - `42` → `Integer(42, Dec)`
   - `3.14` → `Float(3.14, Decimal)`
   - `"hello"` → `String("hello", DoubleQuote)`

2. **Tagged values** (explicit):
```yaml
initial:
  Integer:
    value: 42
    fmt: Dec
```

| Variant | Fields | Description |
|---------|--------|-------------|
| `Integer` | `value: i64`, `fmt: IntegerFmt` | Integer literal (stored as i64 internally) |
| `Float` | `value: f64`, `fmt: FloatFmt` | Float literal |
| `String` | `value: string`, `fmt: StringFmt` | String literal |
| `Bool` | `value: bool` | Boolean literal |

### Type Inference

The Pall compiler performs **type inference** at compile time for all expressions. The type checker walks the AST, assigns unique IDs to each expression node, and builds a `TypeEnv` mapping expression IDs to their inferred types.

**Inference rules:**

| Expression | Type Inferred |
|-----------|---------------|
| Integer literal `42` | `I64` (or `U64` if value exceeds i64 range) |
| Float literal `3.14` | `F64` |
| Boolean `true`/`false` | `Bool` |
| String literal `"hello"` | `String` |
| Variable reference | Type declared in machine |
| `!expr` | `Bool` |
| `-expr` | Same as `expr` (must be numeric) |
| `~expr` | Same as `expr` (must be integer) |
| `a + b` | Common numeric type of `a` and `b` |
| `a == b` | `Bool` |
| `a && b` | `Bool` |

### Type Validation Errors

The compiler produces clear error messages for type violations:

```yaml
# Example: lossy cast error
variables:
  x:
    type: U8
states:
  initial:
    transitions:
      - target: final
        when: null
        do:
          - x = large_value  # ERROR: cannot assign U32 to U8 (lossy cast)
```

Error messages include:
- The operation context (state, machine)
- The source type and target type
- The reason for rejection (e.g., "lossy cast", "operator incompatible with types")

**Common errors:**

| Scenario | Error |
|----------|-------|
| `U32` → `U8` | "cannot assign U32 to U8 (lossy cast)" |
| `I8` → `U8` | "cannot assign I8 to U8 (lossy cast)" |
| `true + 5` | "operator Add incompatible with types Bool and I64" |
| `"hello" > 5` | "operator LessThan incompatible with types String and I64" |
| `when: "text"` | "when condition must be truthy, got String" |

#### IntegerFmt

| Variant | Description | YAML Example |
|---------|-------------|-------------|
| `Dec` | Decimal (base 10) | `42` |
| `Hex` | Hexadecimal (base 16) | `0xff` |
| `Oct` | Octal (base 8) | `0o17` |
| `Bin` | Binary (base 2) | `0b1010` |

#### FloatFmt

| Variant | Description | Example |
|---------|-------------|---------|
| `Decimal` | Standard decimal notation | `3.14` |
| `Scientific` | Scientific notation | `1.5e+2` |

#### StringFmt

| Variant | Description | Example |
|---------|-------------|---------|
| `DoubleQuote` | Double-quoted string | `"hello"` |
| `SingleQuote` | Single-quoted string | `'hello'` |

## Expression Format

### Literals

**Integers:**
- Decimal: `42`
- Hexadecimal: `0xff`, `0xFF`, `0xDEADBEEF`
- Octal: `0o17`
- Binary: `0b1010`

**Floats:**
- Decimal: `3.14`, `0.0`
- Scientific: `1.5e+2`, `2.0E-3`

**Strings:**
- Double-quoted: `"hello world"`, `"line1\nline2"`
- Single-quoted: `'hello'`, `'single \'quote\''`
- Escape sequences: `\\` (backslash), `\"` (double quote), `\'` (single quote), `\n` (newline), `\t` (tab), `\r` (carriage return)

### References

References access machine fields by name:
- `counter` — variable named `counter`
- `state_name` — the current state as a string (via `.as_str()`)
- Any variable, signal, timer, or constant name

### Binary Operators

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `+` | Add | `a + b` | Addition |
| `-` | Subtract | `a - b` | Subtraction |
| `*` | Multiply | `a * b` | Multiplication |
| `/` | Divide | `a / b` | Integer or float division |
| `%` | Modulo | `a % b` | Modulo |
| `==` | Equal | `a == b` | Equality |
| `!=` | Not Equal | `a != b` | Inequality |
| `<` | Less Than | `a < b` | Less than |
| `<=` | Less Equal | `a <= b` | Less than or equal |
| `>` | Greater Than | `a > b` | Greater than |
| `>=` | Greater Equal | `a >= b` | Greater than or equal |
| `&` | Bitwise AND | `a & b` | Bitwise AND |
| `\|` | Bitwise OR | `a \| b` | Bitwise OR |
| `^` | Bitwise XOR | `a ^ b` | Bitwise XOR |
| `&&` | Logical AND | `a && b` | Logical AND (short-circuit) |
| `\|\|` | Logical OR | `a \|\| b` | Logical OR (short-circuit) |
| `^^` | Logical XOR | `a ^^ b` | Logical XOR (exclusive OR) |

### Unary Operators

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `-` | Negate | `-a` | Negation (numeric) |
| `!` | Logical NOT | `!a` | Logical NOT (boolean) |
| `~` | Bitwise NOT | `~a` | Bitwise NOT (integer) |

### Precedence (lowest to highest)

```
||                    — Logical OR
&&                    — Logical AND
==  !=                — Equality
<  <=  >  >=          — Comparison
&                     — Bitwise AND
^                     — Bitwise XOR
|                     — Bitwise OR
+  -                  — Additive
*  /  %               — Multiplicative
```

Parentheses `( )` override precedence.

### Expression Examples

```
counter > 10
0xff + 3.14 == a
!error_flag && counter < 100
(a + b) * (c - d)
~flags & 0x01
x ^^ y
```

## Statement Format

### Syntax

```
target operator expression
```

### Assignment Operators

| Operator | Name | Equivalent To |
|----------|------|---------------|
| `=` | Assign | `target = expression` |
| `+=` | Add Assign | `target = target + expression` |
| `-=` | Subtract Assign | `target = target - expression` |
| `*=` | Multiply Assign | `target = target * expression` |
| `/=` | Divide Assign | `target = target / expression` |
| `%=` | Modulo Assign | `target = target % expression` |
| `&=` | Bitwise AND Assign | `target = target & expression` |
| `\|=` | Bitwise OR Assign | `target = target | expression` |
| `^=` | Bitwise XOR Assign | `target = target ^ expression` |
| `&&=` | Logical AND Assign | `target = target && expression` |
| `\|\|=` | Logical OR Assign | `target = target || expression` |
| `^^=` | Logical XOR Assign | `target = target ^^ expression` |

### Statement Examples

```
counter += 1
result = a + b
flags &= 0x0F
enabled = !disabled
```

### Notes

- The `target` must reference a variable in the machine
- Only variables can appear in the target position
- Input fields cannot be targets (they are read-only)

## Link Format

### Link Struct

| Field | Type | Description |
|-------|------|-------------|
| `id` | `string` | Source machine ID |
| `output` | `string` | Source machine's output field name |

### Syntax in YAML

```yaml
link: "source_machine.output_field"
```

Links reference another machine's output variable and propagate its value to an input.

### Examples

```yaml
inputs:
  temperature:
    type: F64
    link: "sensor_module.temperature"
  enabled:
    type: Bool
    link: "controller_module.enable"
```

In multi-machine (group) mode, link propagation occurs in Phase 1 of `group_tick`, before any per-machine tick.

## Generated Code Structure

### Output Files

The compiler generates one file set per machine group:

```
src/bin/runner/generated/{machine_id}/
├── types.rs    — State enum, Persistent struct, constants
├── tick.rs     — tick() function and init() function
├── group.rs    — group_tick() function (for multi-machine groups)
└── mod.rs      — Module declarations
```

The `types.rs` file contains:
- `State` enum with PascalCase variants
- `Persistent` struct with all fields
- `pub const` definitions for constants

The `tick.rs` file contains:
- `pub fn tick(state: &Persistent, tick_info: &TickInfo) -> Result<Persistent, TickError>`
- `pub fn init() -> Persistent`

### State Enum Naming

State names are converted to PascalCase for Rust enum variants. The conversion follows these rules:
- `_` is treated as a word separator
- Each word is capitalized

Examples:
- `"initial"` → `State::Initial`
- `"goal_state"` → `State::GoalState`
- `"counting"` → `State::Counting`
- `"error"` → `State::Error`

Each variant has an `as_str()` method returning the original lowercase name.

### Tick Return Type

The tick function returns `Result<Persistent, TickError>` — the **full state** is cloned and returned. There is no separate `Update` struct; all changes are reflected in the returned `Persistent` state.

### Tick Execution Order

1. **Match** on current `state`
2. **Execute actions** in declaration order (condition checked first, then statements)
3. **Execute transitions** in declaration order:
   - Actions in the current state execute first (step 2)
   - Transitions are checked in order
   - First matching transition fires: applies its `do` statements, sets the state field, returns immediately
4. **Compute signals** (after all state/transition logic)
5. **Accumulate timers**
6. **Return** `Persistent` (full cloned state)

### Init Function

The `init()` function creates the initial persistent state:

```rust
pub fn init() -> Persistent {
    Persistent {
        state: State::Initial,
        counter: 0i64,
        // ... other fields
    }
}
```

- Sets `state` to the machine's initial state
- Initializes variables to their `initial` values (or type defaults: `0`, `false`, `default()`)
- Sets inputs to `default()`, signals to `default()`, timers to `0`

## Error Handling

Failures propagate as `TickError` with a message string. The generated tick function returns `Result<Persistent, TickError>` — the caller handles errors via `?` or pattern matching.

## Timer Semantics

- When `when` condition is true: `timer += delta_ms`
- When `when` is `null` (always): `timer += delta_ms`
- Otherwise: `timer = 0` (reset)

## State String Names

Each `State` enum variant has an `as_str()` method returning the lowercase machine-defined name:
- `State::Goal.as_str()` → `"goal"`
- `State::Initial.as_str()` → `"initial"`
- `State::GoalState.as_str()` → `"goal_state"`
