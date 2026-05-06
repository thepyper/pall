# Pall Machine Specification

## Overview

Pall is a state machine compiler that generates Rust code from machine definitions. A machine is a deterministic, synchronous state machine with states, variables, inputs, signals, and timers. Machines are compiled into Rust modules containing persistent state structs, update structs, and tick functions.

## Machine Format (YAML)

### Root: `StateMachine`

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
| `do` | `statement[]` | No | `[]` | Actions performed during transition |
| `target` | `string` | Yes | — | Name of the state to transition to |

Transitions are evaluated in order. The first matching transition wins. When a transition fires, it returns immediately with the `Update`, setting the target state.

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
| `link` | `link` | No | `null` | Link from another machine's output |
| `output` | `bool` | No | `false` | Whether this input is also an output |

In multi-machine (group) mode, link propagation occurs before per-machine ticks.

### Signal

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | `Type` | Yes | — | Signal type |
| `output` | `bool` | No | `false` | Whether this signal is exposed as an output |
| `expr` | `expression` | Yes | — | Expression to compute the signal value |

Signals are computed after all state/transition logic and assigned to the `Update`.

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

### Value

| Variant | Fields | Description |
|---------|--------|-------------|
| `Integer` | `value: i64`, `fmt: IntegerFmt` | Integer literal |
| `Float` | `value: f64`, `fmt: FloatFmt` | Float literal |
| `String` | `value: string`, `fmt: StringFmt` | String literal |

#### IntegerFmt

| Variant | Description | Example |
|---------|-------------|---------|
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
- `state_name` — the current state as a string
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
| `\|=` | Bitwise OR Assign | `target = target \| expression` |
| `^=` | Bitwise XOR Assign | `target = target ^ expression` |
| `&&=` | Logical AND Assign | `target = target && expression` |
| `\|\|=` | Logical OR Assign | `target = target \|\| expression` |
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
- Statements produce an `Update` entry: `update.target = Some(value)`
- Only variables can appear in the target position

## Link Format

### Syntax

```
source_id.output_name
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

## Functional Semantics

### Tick

The `tick` function executes one synchronous step of the state machine:

```rust
pub fn tick(state: &Persistent, tick_info: &TickInfo) -> Result<Update, TickError>;
```

- **Deterministic**: Same input always produces same output.
- **Synchronous**: No async, no parallelism, no concurrency.
- **Returns `Update`**: Changes must be applied to `Persistent` by the caller.

### Tick Execution Order

1. **Match** on current `state`
2. **Execute actions** in declaration order (condition checked first)
3. **Execute transitions** in declaration order:
   - Actions in the current state execute first (step 2)
   - Then transitions are checked in order
   - First matching transition fires: applies its `do` statements, sets `Update.state`, returns immediately
4. **Compute signals** (after all state/transition logic)
5. **Accumulate timers**
6. **Return** `Update` (if no transition fired)

### Init

The `init` function creates the initial persistent state:

```rust
pub fn init() -> Persistent;
```

- Sets `state` to the machine's initial state (via `TryFrom<&str>`)
- Sets `state_name` to the initial state's lowercase string name
- Initializes variables to their `initial` values (or type defaults)
- Sets inputs to `default()`, signals to `default()`, timers to `0`

### Update Application

The caller must merge `Update` into `Persistent`:

```rust
fn apply_update(state: &mut Persistent, update: &Update) {
    if let Some(v) = update.counter {
        state.counter = v;
    }
    if let Some(s) = update.state {
        state.state = s;
        state.state_name = s.as_str().to_string();
    }
    // ... repeat for each variable, signal, timer
}
```

Only `Some` values in `Update` are applied; `None` values are ignored.

### Group Tick (Multi-Machine)

The `group_tick` function coordinates multiple machines:

```rust
pub fn group_tick(state: &GroupPersistent, tick_info: &TickInfo) -> Result<GroupUpdate, TickError>;
```

Execution phases:
1. **Phase 1 — Link Propagation**: Copy output values from source machines to target inputs
2. **Phase 2 — Per-Machine Tick**: Call `tick()` for each machine, collecting `Update` into `GroupUpdate`

### Error Handling

Failures propagate as `TickError` with a message string. The generated tick function returns `Result<Update, TickError>` — the caller handles errors via `?` or pattern matching.

### Timer Semantics

- When `when` condition is true: `timer += delta_ms`
- When `when` is `null` (always): `timer += delta_ms`
- Otherwise: `timer = 0` (reset)

### State String Names

Each `State` enum variant has an `as_str()` method returning the lowercase machine-defined name:
- `State::Goal.as_str()` → `"goal"`
- `State::Initial.as_str()` → `"initial"`
