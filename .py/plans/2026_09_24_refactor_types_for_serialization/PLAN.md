# PLAN: Refactor Types for Serialization

## Overview

Refactor `Expression`, `Statement`, and `Link` to support high-fidelity YAML round-trip serialization with format preservation. The key changes:

- **New format enums**: `IntegerFmt`, `FloatFmt`, `StringFmt` in `types.rs`
- **Restructured `Value`**: Uses `IntegerValue`, `FloatValue`, `StringValue` structs carrying format info
- **New wrapper structs**: `FullExpression { raw, expression }` and `FullStatement { raw, statement }`
- **Custom Serialize/Deserialize**: Each type serializes as a YAML string
- **Parser integration**: Parser returns plain AST types; `Full*` types wrap them with `raw` strings

## Micro-Steps

---

### Step 1: Add Format Enums to `types.rs`

**File:** `src/machine/types.rs`

Add three new enums:

```rust
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum IntegerFmt {
    Dec,
    Hex,
    Oct,
    Bin,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum FloatFmt {
    Decimal,
    Scientific,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum StringFmt {
    DoubleQuote,
    SingleQuote,
}
```

**Verification:** `cargo check` compiles.

---

### Step 2: Restructure `Value` Enum in `types.rs`

**File:** `src/machine/types.rs`

Replace the existing `Value` enum:

```rust
pub struct IntegerValue {
    pub value: i64,
    pub fmt: IntegerFmt,
}

pub struct FloatValue {
    pub value: f64,
    pub fmt: FloatFmt,
}

pub struct StringValue {
    pub value: String,
    pub fmt: StringFmt,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Value {
    Integer(IntegerValue),
    Float(FloatValue),
    String(StringValue),
}
```

**Verification:** `cargo check` compiles. This will break any code that constructs `Value` variants directly — fix in next steps.

---

### Step 3: Update `parser/expression.rs` — Return Format-Enriched AST

**File:** `src/machine/parser/expression.rs`

Update the parser functions to produce `Value` variants with format info:

- `dec_integer`, `hex_integer`, `oct_integer`, `bin_integer` → `Value::Integer(IntegerValue { value, fmt: match base })`
- `float` → `Value::Float(FloatValue { value, fmt: FloatFmt::Decimal })` (scientific detection needs parsing the original string — check if exponent notation is present)
- `string_dq` → `Value::String(StringValue { value: unescaped_string, fmt: StringFmt::DoubleQuote })`
- `string_sq` → `Value::String(StringValue { value: unescaped_string, fmt: StringFmt::SingleQuote })`
- `identifier` → `Expression::Reference(Reference { target })` (no format info needed)

**Implementation details:**

For float detection, check if the original string contains `e` or `E` (case-insensitive) to decide `FloatFmt::Scientific` vs `FloatFmt::Decimal`.

For integer formats:
- `dec_integer` → `IntegerFmt::Dec`
- `hex_integer` → `IntegerFmt::Hex`
- `oct_integer` → `IntegerFmt::Oct`
- `bin_integer` → `IntegerFmt::Bin`

**Verification:** `cargo check` compiles. Run `cargo test` — existing parser tests should still pass (they check values, not format info).

---

### Step 4: Create `FullExpression` Struct in `expression.rs`

**File:** `src/machine/expression.rs`

Add the wrapper struct:

```rust
use super::parser::ParseError;

#[derive(Debug, Clone)]
pub struct FullExpression {
    pub raw: String,
    pub expression: Expression,
}

impl FullExpression {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let expression = parser::parse_expression(input)?;
        Ok(Self { raw: input.to_string(), expression })
    }
}
```

**Re-export in `mod.rs`:**

Update `src/machine/mod.rs` to export `FullExpression`.

**Verification:** `cargo check` compiles.

---

### Step 5: Create `FullStatement` Struct in `statement.rs`

**File:** `src/machine/statement.rs`

Add the wrapper struct:

```rust
use super::expression::Expression;
use super::parser::ParseError;

#[derive(Debug, Clone)]
pub struct FullStatement {
    pub raw: String,
    pub statement: Statement,
}

impl FullStatement {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let statement = parser::parse_statement(input)?;
        Ok(Self { raw: input.to_string(), statement })
    }
}
```

The `Statement` struct remains **unchanged** — it keeps `expression: Expression` (not `FullExpression`).

**Re-export in `mod.rs`:**

Update `src/machine/mod.rs` to export `FullStatement`.

**Verification:** `cargo check` compiles.

---

### Step 6: Implement `Serialize` for `FullExpression`

**File:** `src/machine/expression.rs`

Implement `Serialize` so that `FullExpression` outputs the raw string:

```rust
use serde::{Serialize, Serializer};

impl Serialize for FullExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&self.raw)
    }
}
```

**Verification:** `cargo check` compiles.

---

### Step 7: Implement `Deserialize` for `FullExpression`

**File:** `src/machine/expression.rs`

Implement `Deserialize` so that `FullExpression` parses YAML strings:

```rust
use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for FullExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = String::deserialize(deserializer)?;
        FullExpression::parse(&raw)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
```

**Verification:** `cargo check` compiles.

---

### Step 8: Implement `Serialize` for `FullStatement`

**File:** `src/machine/statement.rs`

Implement `Serialize` so that `FullStatement` outputs a formatted string:

```rust
impl Serialize for FullStatement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let operator_str = match self.statement.operator {
            AssignmentOperator::Assign            => "=",
            AssignmentOperator::AddAssign         => "+=",
            AssignmentOperator::SubAssign         => "-=",
            AssignmentOperator::MulAssign         => "*=",
            AssignmentOperator::DivAssign         => "/=",
            AssignmentOperator::ModAssign         => "%=",
            AssignmentOperator::AndAssign         => "&=",
            AssignmentOperator::OrAssign          => "|=",
            AssignmentOperator::XorAssign         => "^=",
            AssignmentOperator::LogicalAndAssign  => "&&=",
            AssignmentOperator::LogicalOrAssign   => "||=",
            AssignmentOperator::LogicalXorAssign  => "^^=",
        };
        let s = format!("{} {} {}", self.statement.target, operator_str, self.statement.expression.raw);
        serializer.serialize_str(&s)
    }
}
```

**Verification:** `cargo check` compiles.

---

### Step 9: Implement `Deserialize` for `FullStatement`

**File:** `src/machine/statement.rs`

Implement `Deserialize`:

```rust
impl<'de> Deserialize<'de> for FullStatement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = String::deserialize(deserializer)?;
        FullStatement::parse(&raw)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
```

**Verification:** `cargo check` compiles.

---

### Step 10: Implement `Serialize` for `Link`

**File:** `src/machine/link.rs`

```rust
use serde::{Serialize, Serializer};

impl Serialize for Link {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&format!("{}.{}", self.id, self.output))
    }
}
```

**Verification:** `cargo check` compiles.

---

### Step 11: Implement `Deserialize` for `Link`

**File:** `src/machine/link.rs`

```rust
use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for Link {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = String::deserialize(deserializer)?;
        parser::parse_link(&raw)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
```

**Verification:** `cargo check` compiles.

---

### Step 12: Update `actions.rs` — Change Field Types

**File:** `src/machine/actions.rs`

Change the types in `Action` and `Transition`:

```rust
// OLD:
use super::expression::Expression;
pub struct Action {
    pub when: Option<Expression>,
    pub r#do: Vec<Statement>,
}
pub struct Transition {
    pub when: Option<Expression>,
    pub r#do: Vec<Statement>,
    pub target: String,
}

// NEW:
use super::expression::FullExpression;
use super::statement::FullStatement;
pub struct Action {
    pub when: Option<FullExpression>,
    pub r#do: Vec<FullStatement>,
}
pub struct Transition {
    pub when: Option<FullExpression>,
    pub r#do: Vec<FullStatement>,
    pub target: String,
}
```

**Verification:** `cargo check` compiles.

---

### Step 13: Update `connections.rs` — No Changes Needed

`Input.link: Option<Link>` uses the `Link` type which now has custom Serialize/Deserialize. No changes needed — serde will use the new impls automatically.

---

### Step 14: Update `mod.rs` — Re-exports

**File:** `src/machine/mod.rs`

Update re-exports to include new types:

```rust
pub use types::{Type, Value, IntegerValue, FloatValue, StringValue, IntegerFmt, FloatFmt, StringFmt};
pub use expression::{Reference, Expression, BinaryOperator, UnaryOperator, FullExpression};
pub use statement::{Statement, AssignmentOperator, FullStatement};
pub use link::Link;
pub use connections::{Input, Output};
pub use variables::{Signal, Timer, Variable, Constant};
pub use actions::{Action, Transition, State};
```

**Verification:** `cargo check` compiles.

---

### Step 15: Update Tests in `test.rs`

**File:** `src/machine/test.rs`

Update test YAML strings if needed. The existing tests (`test_deserialize_machine_minimal`, `test_deserialize_machine`) only test minimal machines with empty states/transitions — they should work as-is since `actions` and `transitions` are optional with defaults.

If tests need to include expressions/statements, add them with YAML string format:

```yaml
transitions:
  - when: "x > 5"
    do:
      - "y = 10"
    target: "next_state"
```

**Verification:** `cargo test` passes all existing tests.

---

### Step 16: Run Full Test Suite

**Command:** `cargo test`

Expected results:
- All existing parser tests pass (they test `Expression`/`Statement`/`Link` parsing, not the `Full*` wrappers)
- All existing YAML deserialization tests pass
- `cargo check` has no errors (warnings about unused types are fine)

---

### Step 17: Verify Round-Trip (Manual Check)

Write a small test or main function that:

1. Deserializes a YAML string containing expressions, statements, and links
2. Verifies `FullExpression.raw` and `FullStatement.raw` match the original input
3. Serializes back to YAML
4. Deserializes again and verifies values are identical

Example test data:
```yaml
id: test
states:
  initial:
    transitions:
      - when: "0xff + 3.14e-2 == a"
        do:
          - "result += 0o17"
        target: "next"
inputs:
  my_input:
    type: Bool
    link: "source.my_signal"
```

**Verification:** Round-trip produces equivalent YAML.

---

## Execution Order Summary

| Step | File(s) | Concern |
|------|---------|---------|
| 1 | `types.rs` | Add `IntegerFmt`, `FloatFmt`, `StringFmt` enums |
| 2 | `types.rs` | Restructure `Value` with `IntegerValue`, `FloatValue`, `StringValue` |
| 3 | `parser/expression.rs` | Update parser to produce format-enriched `Value` variants |
| 4 | `expression.rs`, `mod.rs` | Create `FullExpression` struct + parse method + re-export |
| 5 | `statement.rs`, `mod.rs` | Create `FullStatement` struct + parse method + re-export |
| 6 | `expression.rs` | `Serialize` impl for `FullExpression` (outputs `raw`) |
| 7 | `expression.rs` | `Deserialize` impl for `FullExpression` (calls parser) |
| 8 | `statement.rs` | `Serialize` impl for `FullStatement` (formats as `"{} {} {}"`) |
| 9 | `statement.rs` | `Deserialize` impl for `FullStatement` (calls parser) |
| 10 | `link.rs` | `Serialize` impl for `Link` (formats as `"id.output"`) |
| 11 | `link.rs` | `Deserialize` impl for `Link` (calls parser) |
| 12 | `actions.rs` | Change `Action`/`Transition` field types to `FullExpression`/`FullStatement` |
| 13 | `connections.rs` | No changes needed (uses `Link` which now has custom impls) |
| 14 | `mod.rs` | Update re-exports for all new types |
| 15 | `test.rs` | Update tests if needed for new types |
| 16 | All | Run `cargo test` |
| 17 | All | Manual round-trip verification |

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| `Value` restructuring breaks parser tests | Step 3 updates parser first; tests check values not format info |
| Custom Serialize/Deserialize causes circular issues | No circular deps — each type depends only on simpler types |
| Float scientific notation detection | Check for `e`/`E` in original string during parsing |
| `FullStatement` serialize depends on `expression.raw` | `FullStatement.expression` is `FullExpression`, which has `raw` |
| Existing YAML files may have invalid expressions | Parser errors propagate via `Result` in `Deserialize` |
