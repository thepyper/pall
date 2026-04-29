# INTERVIEW: Refactor Types for Serialization

## Objective

Refactor the serialization format for `StateMachine` and all its nested types so that `Expression`, `Statement`, and `Link` are:

1. **Deserialized from YAML strings** via the existing pest-based parser
2. **Re-serializable as YAML strings** (high-fidelity round-trip: YAML → AST → YAML should produce equivalent output)
3. **Carry format information** (integer base, float notation, string quoting) for a secondary code-generation use case

The rest of the `StateMachine` types (Type, Value primitives, Input, Output, Signal, Timer, Variable, Constant, Action, Transition, State) serialize/deserialize normally via `serde_yaml`.

---

## Design Decisions

### 1. Format Types in `types.rs`

Three new format enums, all `pub`:

```rust
pub enum IntegerFmt {
    Dec,
    Hex,
    Oct,
    Bin,
}

pub enum FloatFmt {
    Decimal,      // e.g., 3.14
    Scientific,   // e.g., 3.14e+0
}

pub enum StringFmt {
    DoubleQuote,  // "hello"
    SingleQuote,  // 'hello'
}
```

Default values (for hypothetical programmatic construction): `IntegerFmt::Dec`, `FloatFmt::Decimal`, `StringFmt::DoubleQuote`.

### 2. Restructured `Value` Enum in `types.rs`

The existing `Value` enum is restructured to carry struct-typed leaves instead of primitives:

```rust
pub enum Value {
    Integer(IntegerValue),
    Float(FloatValue),
    String(StringValue),
}

pub struct IntegerValue {
    pub value: i64,
    pub fmt: IntegerFmt,
}

pub struct FloatValue {
    pub value: f64,
    pub fmt: FloatFmt,
}

pub struct StringValue {
    pub value: String,    // original unescaped string
    pub fmt: StringFmt,
}
```

`Value` enum name is kept as-is. All types are `pub`.

### 3. `FullExpression` Struct in `expression.rs`

A new wrapper struct pairs the raw YAML string with the parsed AST:

```rust
pub struct FullExpression {
    pub raw: String,        // always present, never optional
    pub expression: Expression,
}

impl FullExpression {
    pub fn parse(input: &str) -> Result<Self, ParseError>;
}
```

- `raw` is the original YAML string, stored verbatim during parsing
- `raw` is always present — guaranteed by the API (no `Option`, no `None` path)
- `expression` is the existing `Expression` AST enum

### 4. `Expression` Enum Stays as-Is

The existing `Expression` enum is kept unchanged structurally. It continues to reference `Value`, but `Value` now carries struct-typed leaves with format info.

```rust
pub enum Expression {
    Value(Value),
    Reference(Reference),
    Parenthesis(Box<Expression>),
    Unary(UnaryOperator, Box<Expression>),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
}
```

### 5. `Statement` Uses `FullExpression`

```rust
pub struct Statement {
    pub target: String,
    pub operator: AssignmentOperator,
    pub expression: FullExpression,   // changed from Expression
}
```

`Statement` has no `raw` field of its own — it carries the `raw` through `expression.raw`.

### 6. `Link` Stays as-Is (No Format Info Needed)

```rust
pub struct Link {
    pub id: String,
    pub output: String,
}
```

Custom Serialize/Deserialize will format it as a YAML string `"id.output"`. No raw or fmt fields.

### 7. Custom Serialize/Deserialize (Approach A)

Each type implements `Serialize`/`Deserialize` manually to serialize as YAML strings. This means fields in `Action`, `Transition`, `Input` etc. remain clean — no `#[serde(serialize_with)]` annotations needed.

**FullExpression:**
- **Serialize:** Outputs YAML string equal to `raw`
- **Deserialize:** Calls the parser, returns `FullExpression { raw: input.into(), expression }`

**Statement:**
- **Serialize:** Outputs YAML string `"{target} {operator_string} {expression.raw}"` (space-separated)
- **Deserialize:** Calls parser, wraps result: `Statement { expression: FullExpression { raw: input.into(), expression } }`

**Link:**
- **Serialize:** Outputs YAML string `"{id}.{output}"`
- **Deserialize:** Calls parser

### 8. Field Changes in `StateMachine` Types

```rust
pub struct Action {
    pub when: Option<FullExpression>,     // was Option<Expression>
    pub r#do: Vec<Statement>,             // unchanged (Statement already has FullExpression)
}

pub struct Transition {
    pub when: Option<FullExpression>,     // was Option<Expression>
    pub r#do: Vec<Statement>,             // unchanged
    pub target: String,
}
```

### 9. Operator Strings in Statement Serialization

`AssignmentOperator` variants serialize to their grammar.pest operator strings:
`=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `&&=`, `||=`, `^^=`

Statement format: `"{target} {operator_str} {expression.raw}"`

### 10. Parser Integration

- Parser module already exists (`src/machine/parser/`)
- `parse_expression(input)` → `Result<Expression, ParseError>` (existing)
- `parse_statement(input)` → `Result<Statement, ParseError>` (existing, but `Statement` needs update for `FullExpression`)
- `parse_link(input)` → `Result<Link, ParseError>` (existing)

New method:
- `FullExpression::parse(input)` → calls parser, returns `FullExpression { raw: input.into(), expression }`

Parser produces AST with format info stored in `Value` variants (`IntegerValue`, `FloatValue`, `StringValue`).

### 11. No Format Module

No `Formatter` module is needed for the YAML loop. The round-trip is: parse → store raw → emit raw. No formatting logic required.

Format info (`IntegerFmt`, `FloatFmt`, `StringFmt`) is stored in the AST solely for the code-generation use case (secondary, less constrained).

### 12. Comments — Not Preserved

YAML `#` comments are not preserved. `serde_yaml` strips them. This is acceptable.

### 13. No Mutation Path in YAML Loop

Programmatic mutation of AST nodes is not part of the YAML loop. If an expression needs to change, it is re-parsed from a new string. The `raw` field is set at parse time and never modified.

### 14. `FloatValue.value` Stored as `f64`

Floats are stored as `f64`, not as `String`. The `FloatFmt` enum captures whether it was scientific or decimal notation.

### 15. `StringValue.value` Stores Original Unescaped String

The original string from YAML (including escape sequences like `\n`, `\t`). The code generation layer will re-escape as needed for the target language.

### 16. `Reference`, `Parenthesis`, `Unary` — No Format Info

These AST nodes have no format-specific data. They remain as-is.

### 17. All Types Are `pub`

Every new and modified type is public: `FullExpression`, `IntegerValue`, `FloatValue`, `StringValue`, `IntegerFmt`, `FloatFmt`, `StringFmt`.

### 18. `FullExpression::parse()` Returns `Result`

For error handling, `FullExpression::parse(&str) -> Result<FullExpression, ParseError>`. Parser errors propagate via `ParseError`.

### 19. File Locations

- `IntegerFmt`, `FloatFmt`, `StringFmt`, `Value`, `IntegerValue`, `FloatValue`, `StringValue` → `types.rs`
- `FullExpression` → `expression.rs`
- Parser integration methods → `parser/mod.rs` and `FullExpression` impl in `expression.rs`
- `Statement`, `Link` custom Serialize/Deserialize → `statement.rs`, `link.rs`

---

## Files to Modify

| File | Changes |
|------|---------|
| `types.rs` | Add `IntegerFmt`, `FloatFmt`, `StringFmt`; restructure `Value` to use `IntegerValue`, `FloatValue`, `StringValue` |
| `expression.rs` | Add `FullExpression` struct; add `FullExpression::parse()`; add custom `Serialize`/`Deserialize` for `FullExpression` |
| `statement.rs` | Change `expression: Expression` → `expression: FullExpression`; add custom `Serialize`/`Deserialize` |
| `link.rs` | Add custom `Serialize`/`Deserialize` (no struct changes) |
| `actions.rs` | Change `when: Option<Expression>` → `when: Option<FullExpression>` in `Action` and `Transition` |
| `parser/mod.rs` | Add `FullExpression::parse` integration |
| `mod.rs` | Update re-exports if needed |

## Not Changed

- `Expression` enum (structurally unchanged, just references new `Value` types)
- `UnaryOperator`, `BinaryOperator`, `Reference`
- `AssignmentOperator`
- `Input`, `Output`, `Signal`, `Timer`, `Variable`, `Constant`, `Action` (struct), `Transition` (struct), `State`
- `Type` enum
- Parser module structure (grammar.pest, parser/expression.rs, parser/statement.rs, parser/link.rs)
