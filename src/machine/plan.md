# Plan: Pest-Based Parser with Pratt-Style Precedence

## Problem

The state machine YAML config contains expression-like strings that need parsing into `Expression`, `Statement`, and `Link` Rust types. Current state: grammar.pest has flat recursion (no precedence), incomplete parse stub, missing operators.

## Scope

**In:** Grammar rewrite, Rust type updates, parser implementations, error handling, tests
**Out:** YAML integration, type checking, file-level parsing

## File Structure

```
src/machine/
  grammar.pest     ← rewritten with precedence rules + named tokens
  mod.rs           ← updated module decls + re-exports
  types.rs         ← Type, Value (no changes)
  expression.rs    ← Expression, BinaryOperator, UnaryOperator, Reference
  statement.rs     ← Statement, AssignmentOperator (NEW FILE)
  link.rs          ← Link struct (NEW FILE)
  connections.rs   ← Input, Output (Link import from link.rs)
  variables.rs     ← Signal, Timer, Variable, Constant (no changes)
  actions.rs       ← Action, Transition, State (no changes)
  parser/
    mod.rs         ← ParseError, GrammarParser, Rule re-export
    expression.rs  ← expression_from_pair, parse_expression
    statement.rs   ← statement_from_pair, parse_statement
    link.rs        ← link_from_pair, parse_link
```

## Micro-Steps (14 steps)

---

### Step 1: Rewrite grammar.pest

**Goal:** C-like precedence with separate rules per level + named token rules for visitor.

**File:** `src/machine/grammar.pest` — complete rewrite

```pest
WHITESPACE = _{ " " | "\t" }

alpha     = _{ 'a' .. 'z' | 'A' .. 'Z' }
dec_digit = _{ '0' .. '9' }
hex_digit = _{ dec_digit | 'a' .. 'f' | 'A' .. 'F' }
oct_digit = _{ '0' .. '7' }
bin_digit = _{ '0' | '1' }

hex_integer  = { "0x" ~ hex_digit+ }
oct_integer  = { "0o" ~ oct_digit+ }
bin_integer  = { "0b" ~ bin_digit+ }
dec_integer  = { dec_digit+ }
_integer_value = _{ hex_integer | oct_integer | bin_integer | dec_integer }

float = { dec_digit+ ~ "." ~ dec_digit+ }

escape_char = _{ "\\" ~ ("n" | "t" | "r" | "\\" | "\"" | "'" | "0" | "b" | "f") }
string_dq  = { "\"" ~ (!("\"" | "\\") | escape_char)* ~ "\"" }
string_sq  = { "'" ~ (!("'" | "\\") | escape_char)* ~ "'" }
_string = _{ string_dq | string_sq }

identifier = { ("_" | alpha) ~ ("_" | alpha | dec_digit)* }

_value = _{ _float | _integer_value | _string | identifier }

_parenthesis = { "(" ~ expression ~ ")" }

// ── Named token rules for visitor ────────────────────────────────────────────
// Binary operators
LOGICAL_OR   = { "||" }
LOGICAL_AND  = { "&&" }
LOGICAL_XOR  = { "^^" }
BITWISE_OR   = { "|" }
BITWISE_XOR  = { "^" }
BITWISE_AND  = { "&" }
EQ           = { "==" }
NEQ          = { "!=" }
LT           = { "<" }
LE           = { "<=" }
GT           = { ">" }
GE           = { ">=" }
ADD          = { "+" }
SUB          = { "-" }
MUL          = { "*" }
DIV          = { "/" }
MOD          = { "%" }
// Unary operators
NOT          = { "!" }
BIT_NOT      = { "~" }
// Assignment operators
ASSIGN           = { "=" }
ADD_ASSIGN       = { "+=" }
SUB_ASSIGN       = { "-=" }
MUL_ASSIGN       = { "*=" }
DIV_ASSIGN       = { "/=" }
MOD_ASSIGN       = { "%=" }
AND_ASSIGN       = { "&=" }
OR_ASSIGN        = { "|=" }
XOR_ASSIGN       = { "^=" }
LOGICAL_AND_ASSIGN  = { "&&=" }
LOGICAL_OR_ASSIGN   = { "||=" }
LOGICAL_XOR_ASSIGN  = { "^^=" }

// ── Precedence rules (low → high) ───────────────────────────────────────────
logical_or   = { logical_and ~ (LOGICAL_OR ~ logical_and)* }
logical_and  = { logical_xor ~ (LOGICAL_AND ~ logical_xor)* }
logical_xor  = { bitwise_or ~ (LOGICAL_XOR ~ bitwise_or)* }
bitwise_or   = { bitwise_xor ~ (BITWISE_OR ~ bitwise_xor)* }
bitwise_xor  = { bitwise_and ~ (BITWISE_XOR ~ bitwise_and)* }
bitwise_and  = { equality ~ (BITWISE_AND ~ equality)* }
equality     = { comparison ~ ((EQ | NEQ) ~ comparison)* }
comparison   = { additive ~ ((LE | GE | LT | GT) ~ additive)* }
additive     = { multiplicative ~ ((ADD | SUB) ~ multiplicative)* }
multiplicative = { unary ~ ((MUL | DIV | MOD) ~ unary)* }

primary = { _parenthesis | _value }
unary = { (SUB | NOT | BIT_NOT) ~ unary | primary }

expression = { SOI ~ unary ~ EOI }

statement = { SOI ~ identifier ~ assignment_operator ~ expression ~ EOI }
assignment_operator = _{
    ASSIGN | ADD_ASSIGN | SUB_ASSIGN | MUL_ASSIGN | DIV_ASSIGN | MOD_ASSIGN |
    AND_ASSIGN | OR_ASSIGN | XOR_ASSIGN |
    LOGICAL_AND_ASSIGN | LOGICAL_OR_ASSIGN | LOGICAL_XOR_ASSIGN
}

link = { SOI ~ identifier ~ "." ~ identifier ~ EOI }
```

**Verification:** `cargo test` — old tests should still parse (grammar is backward-compatible).

---

### Step 2: Add UnaryOperator enum to expression.rs

**File:** `src/machine/expression.rs` — add after `BinaryOperator` enum:

```rust
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum UnaryOperator {
    Negate,   // -
    Not,      // !
    BitNot,   // ~
}
```

Also add to `BinaryOperator`:
```rust
    LogicalOr,    // ||
    LogicalAnd,   // &&
    LogicalXor,   // ^^
```

Also add to `Expression` enum:
```rust
    Unary(UnaryOperator, Box<Expression>),  // before Binary
```

**Verification:** `cargo check` compiles.

---

### Step 3: Create statement.rs

**File:** `src/machine/statement.rs` — new file

```rust
use serde::{Serialize, Deserialize};
use super::expression::Expression;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum AssignmentOperator {
    Assign,             // =
    AddAssign,          // +=
    SubAssign,          // -=
    MulAssign,          // *=
    DivAssign,          // /=
    ModAssign,          // %=
    AndAssign,          // &=
    OrAssign,           // |=
    XorAssign,          // ^=
    LogicalAndAssign,   // &&=
    LogicalOrAssign,    // ||=
    LogicalXorAssign,   // ^^=
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Statement {
    pub target: String,
    pub operator: AssignmentOperator,
    pub expression: Expression,
}
```

**File:** `src/machine/mod.rs` — add `pub use statement::{Statement, AssignmentOperator};`

Remove old `Statement`/`AssignmentOperator` from expression.rs (they were there before).

**Verification:** `cargo check` compiles. Update `actions.rs` import: change `use super::expression::Statement` to `use super::statement::Statement`.

---

### Step 4: Create link.rs

**File:** `src/machine/link.rs` — new file

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Link {
    pub id: String,
    pub output: String,
}
```

**File:** `src/machine/mod.rs` — add `pub use link::Link;`

**File:** `src/machine/connections.rs` — remove `Link` struct, add `use super::link::Link;`

**Verification:** `cargo check` compiles.

---

### Step 5: Create parser/mod.rs

**File:** `src/machine/parser/mod.rs` — new file

```rust
use pest::error::Error;
use pest::Span;
use std::fmt;

mod grammar {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "machine/grammar.pest"]
    pub struct GrammarParser;
}

pub use grammar::{GrammarParser, Rule};

mod expression;
mod statement;
mod link;

pub use expression::parse_expression;
pub use statement::parse_statement;
pub use link::parse_link;

// ── ParseError ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub span: Option<Span>,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), line: 0, column: 0, span: None }
    }

    pub fn with_span(message: impl Into<String>, span: Span) -> Self {
        let start = span.start();
        let line_start = span.line_start();
        Self {
            message: message.into(),
            line: span.line_start(),
            column: start - line_start,
            span: Some(span),
        }
    }

    pub fn from_pest(input: &str, err: Error<Rule>) -> Self {
        let loc = err.location();
        let (line, column) = loc.line_col();
        Self {
            message: err.variant().to_string(),
            line,
            column,
            span: Some(loc),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.span.is_some() {
            write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
        } else {
            write!(f, "Parse error: {}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}
```

**File:** `src/machine/mod.rs` — add `pub mod parser;`

**Verification:** `cargo check` compiles.

---

### Step 6: Create parser/expression.rs

**File:** `src/machine/parser/expression.rs` — new file

```rust
use pest::iterators::Pair;
use pest::Parser;

use super::{GrammarParser, Rule, ParseError};
use super::expression as expr_types;
use crate::machine::types::Value;
use crate::machine::expression::{Expression, Reference, UnaryOperator, BinaryOperator};

/// Parse a plain string into an Expression AST.
pub fn parse_expression(input: &str) -> Result<Expression, ParseError> {
    let mut pairs = GrammarParser::parse(Rule::expression, input)
        .map_err(|e| ParseError::from_pest(input, e))?;

    let pair = pairs.next().ok_or_else(|| ParseError::new("empty expression"))?;
    expression_from_pair(pair)
}

/// Recursively convert a pest pair to Expression AST.
pub(crate) fn expression_from_pair(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    match pair.as_rule() {
        Rule::expression => {
            let mut inner = pair.into_inner();
            let child = inner.next().ok_or_else(|| ParseError::new("empty expression"))?;
            expression_from_pair(child)
        }

        // Unary prefix operators
        Rule::unary => {
            let mut children = pair.into_inner().peekable();
            if let Some(first) = children.peek() {
                if matches!(first.as_rule(), Rule::SUB | Rule::NOT | Rule::BIT_NOT) {
                    let op = unary_op_from_pair(children.next().unwrap())?;
                    let operand = expression_from_pair(children.next().ok_or_else(|| {
                        ParseError::new("unary operator without operand")
                    })?);
                    return operand.map(|e| Expression::Unary(op, Box::new(e)));
                }
            }
            let child = pair.into_inner().next().ok_or_else(|| {
                ParseError::new("unary rule with no children")
            })?;
            expression_from_pair(child)
        }

        // Binary operators at each precedence level
        Rule::logical_or
        | Rule::logical_and
        | Rule::logical_xor
        | Rule::bitwise_or
        | Rule::bitwise_xor
        | Rule::bitwise_and
        | Rule::equality
        | Rule::comparison
        | Rule::additive
        | Rule::multiplicative => binary_from_pairs(pair),

        // Primary
        Rule::primary => {
            let child = pair.into_inner().next().ok_or_else(|| {
                ParseError::new("primary with no children")
            })?;
            expression_from_pair(child)
        }

        // Parenthesized
        Rule::_parenthesis => {
            let mut inner = pair.into_inner();
            let child = inner.next().ok_or_else(|| ParseError::new("empty parentheses"))?;
            expression_from_pair(child).map(Expression::Parenthesis)
        }

        // Integer types
        Rule::dec_integer => Ok(Expression::Value(Value::Integer(parse_integer(pair.as_str())))),
        Rule::hex_integer => Ok(Expression::Value(Value::Integer(
            i64::from_str_radix(&pair.as_str()[2..], 16)
                .map_err(|_| ParseError::new(format!("invalid hex: {}", pair.as_str())))?)),
        ))),
        Rule::oct_integer => Ok(Expression::Value(Value::Integer(
            i64::from_str_radix(&pair.as_str()[2..], 8)
                .map_err(|_| ParseError::new(format!("invalid octal: {}", pair.as_str())))?)),
        ))),
        Rule::bin_integer => Ok(Expression::Value(Value::Integer(
            i64::from_str_radix(&pair.as_str()[2..], 2)
                .map_err(|_| ParseError::new(format!("invalid binary: {}", pair.as_str())))?)),
        ))),

        // Float
        Rule::float => Ok(Expression::Value(Value::Float(
            pair.as_str().parse::<f64>()
                .map_err(|_| ParseError::new(format!("invalid float: {}", pair.as_str())))?)),
        ))),

        // Strings
        Rule::string_dq => {
            let inner = &pair.as_str()[1..pair.as_str().len()-1];
            Ok(Expression::Value(Value::String(unescape_string(inner))))
        }
        Rule::string_sq => {
            let inner = &pair.as_str()[1..pair.as_str().len()-1];
            Ok(Expression::Value(Value::String(unescape_string(inner))))
        }

        // Identifier → Reference
        Rule::identifier => Ok(Expression::Reference(Reference {
            target: pair.as_str().to_string(),
        })),

        _ => Err(ParseError::new(format!(
            "unexpected rule {:?} in expression",
            pair.as_rule()
        ))),
    }
}

/// Process left-associative binary chains.
/// Grammar produces: [left, op, right, op, right, ...]
fn binary_from_pairs(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let children: Vec<Pair<Rule>> = pair.into_inner().collect();
    if children.is_empty() {
        return Err(ParseError::new("binary operator with no operands"));
    }
    if children.len() == 1 {
        return expression_from_pair(children.into_iter().next().unwrap());
    }

    let mut result = expression_from_pair(children[0].clone())?;
    let mut i = 1;
    while i + 1 < children.len() {
        let op = binary_op_from_pair(children[i].clone())?;
        let right = expression_from_pair(children[i + 1].clone())?;
        result = Expression::Binary(Box::new(result), op, Box::new(right));
        i += 2;
    }
    Ok(result)
}

fn binary_op_from_pair(pair: Pair<Rule>) -> Result<BinaryOperator, ParseError> {
    match pair.as_rule() {
        Rule::LOGICAL_OR   => Ok(BinaryOperator::LogicalOr),
        Rule::LOGICAL_AND  => Ok(BinaryOperator::LogicalAnd),
        Rule::LOGICAL_XOR  => Ok(BinaryOperator::LogicalXor),
        Rule::BITWISE_OR   => Ok(BinaryOperator::BitOr),
        Rule::BITWISE_XOR  => Ok(BinaryOperator::BitXor),
        Rule::BITWISE_AND  => Ok(BinaryOperator::BitAnd),
        Rule::EQ           => Ok(BinaryOperator::Equal),
        Rule::NEQ          => Ok(BinaryOperator::NotEqual),
        Rule::LT           => Ok(BinaryOperator::LessThan),
        Rule::LE           => Ok(BinaryOperator::LessEqual),
        Rule::GT           => Ok(BinaryOperator::GreaterThan),
        Rule::GE           => Ok(BinaryOperator::GreaterEqual),
        Rule::ADD          => Ok(BinaryOperator::Add),
        Rule::SUB          => Ok(BinaryOperator::Sub),
        Rule::MUL          => Ok(BinaryOperator::Mul),
        Rule::DIV          => Ok(BinaryOperator::Div),
        Rule::MOD          => Ok(BinaryOperator::Mod),
        _ => Err(ParseError::new(format!(
            "unexpected binary operator rule: {:?}", pair.as_rule()
        ))),
    }
}

fn unary_op_from_pair(pair: Pair<Rule>) -> Result<UnaryOperator, ParseError> {
    match pair.as_rule() {
        Rule::SUB     => Ok(UnaryOperator::Negate),
        Rule::NOT     => Ok(UnaryOperator::Not),
        Rule::BIT_NOT => Ok(UnaryOperator::BitNot),
        _ => Err(ParseError::new(format!(
            "unexpected unary operator rule: {:?}", pair.as_rule()
        ))),
    }
}

fn parse_integer(s: &str) -> i64 {
    if s.starts_with("0x") || s.starts_with("0X") {
        i64::from_str_radix(&s[2..], 16).unwrap()
    } else if s.starts_with("0o") || s.starts_with("0O") {
        i64::from_str_radix(&s[2..], 8).unwrap()
    } else if s.starts_with("0b") || s.starts_with("0B") {
        i64::from_str_radix(&s[2..], 2).unwrap()
    } else {
        s.parse::<i64>().unwrap()
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n')  => result.push('\n'),
                Some('t')  => result.push('\t'),
                Some('r')  => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"')  => result.push('"'),
                Some('\'') => result.push('\''),
                Some('0')  => result.push('\0'),
                Some('b')  => result.push('\u{0008}'),
                Some('f')  => result.push('\u{000C}'),
                Some(other) => { result.push('\\'); result.push(other); }
                None        => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::types::Value;
    use crate::machine::expression::{BinaryOperator, UnaryOperator};

    fn expect_parse(input: &str) -> Expression {
        parse_expression(input).unwrap_or_else(|e| panic!("parse failed for '{}': {}", input, e))
    }

    #[test] fn test_literal_int() {
        assert_eq!(expect_parse("42"), Expression::Value(Value::Integer(42)));
    }

    #[test] fn test_literal_hex() {
        assert_eq!(expect_parse("0xff"), Expression::Value(Value::Integer(255)));
    }

    #[test] fn test_literal_octal() {
        assert_eq!(expect_parse("0o17"), Expression::Value(Value::Integer(15)));
    }

    #[test] fn test_literal_binary() {
        assert_eq!(expect_parse("0b1010"), Expression::Value(Value::Integer(10)));
    }

    #[test] fn test_literal_float() {
        assert_eq!(expect_parse("3.14"), Expression::Value(Value::Float(3.14)));
    }

    #[test] fn test_literal_string_dq() {
        assert_eq!(expect_parse("\"hello\""), Expression::Value(Value::String("hello".into())));
    }

    #[test] fn test_literal_string_sq() {
        assert_eq!(expect_parse("'world'"), Expression::Value(Value::String("world".into())));
    }

    #[test] fn test_literal_string_escape() {
        assert_eq!(expect_parse("\"a\\nb\""), Expression::Value(Value::String("a\nb".into())));
    }

    #[test] fn test_reference() {
        assert_eq!(expect_parse("my_var"), Expression::Reference(Reference{target:"my_var".into()}));
    }

    #[test] fn test_addition() {
        match expect_parse("1 + 2") {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Value(Value::Integer(1)));
                assert_eq!(*r, Expression::Value(Value::Integer(2)));
            }
            x => panic!("expected binary add, got {:?}", x),
        }
    }

    #[test] fn test_precedence_mul_over_add() {
        // 1 + 2 * 3 → 1 + (2 * 3)
        match expect_parse("1 + 2 * 3") {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Value(Value::Integer(1)));
                match *r {
                    Expression::Binary(ll, BinaryOperator::Mul, rr) => {
                        assert_eq!(*ll, Expression::Value(Value::Integer(2)));
                        assert_eq!(*rr, Expression::Value(Value::Integer(3)));
                    }
                    x => panic!("expected mul on right: {:?}", x),
                }
            }
            x => panic!("expected add: {:?}", x),
        }
    }

    #[test] fn test_unary_negate() {
        match expect_parse("-5") {
            Expression::Unary(UnaryOperator::Negate, op) => {
                assert_eq!(*op, Expression::Value(Value::Integer(5)));
            }
            x => panic!("expected unary negate: {:?}", x),
        }
    }

    #[test] fn test_unary_not() {
        match expect_parse("!flag") {
            Expression::Unary(UnaryOperator::Not, op) => {
                assert_eq!(*op, Expression::Reference(Reference{target:"flag".into()}));
            }
            x => panic!("expected unary not: {:?}", x),
        }
    }

    #[test] fn test_unary_bitnot() {
        match expect_parse("~x") {
            Expression::Unary(UnaryOperator::BitNot, op) => {
                assert_eq!(*op, Expression::Reference(Reference{target:"x".into()}));
            }
            x => panic!("expected unary bitnot: {:?}", x),
        }
    }

    #[test] fn test_logical_or() {
        match expect_parse("a || b") {
            Expression::Binary(l, BinaryOperator::LogicalOr, r) => {
                assert_eq!(*l, Expression::Reference(Reference{target:"a".into()}));
                assert_eq!(*r, Expression::Reference(Reference{target:"b".into()}));
            }
            x => panic!("expected logical or: {:?}", x),
        }
    }

    #[test] fn test_precedence_all_levels() {
        // Verify complex expression parses without error
        let expr = expect_parse("1 || 2 && 3 ^^ 4 | 5 ^ 6 & 7 == 8 != 9 <= 10 >= 11 < 12 > 13 + 14 - 15 * 16 / 17 % 18");
        // Top level is || (lowest precedence)
        match expr {
            Expression::Binary(l, BinaryOperator::LogicalOr, r) => {
                assert_eq!(*l, Expression::Value(Value::Integer(1)));
                assert!(matches!(*r, Expression::Binary(_, _, _)));
            }
            x => panic!("expected || at top: {:?}", x),
        }
    }

    #[test] fn test_parentheses() {
        // (1 + 2) * 3 → (1+2) * 3 (parenthesis preserves grouping)
        match expect_parse("(1 + 2) * 3") {
            Expression::Binary(l, BinaryOperator::Mul, r) => {
                match **l {
                    Expression::Parenthesis(inner) => {
                        assert!(matches!(*inner, Expression::Binary(_, _, _)));
                    }
                    x => panic!("expected paren: {:?}", x),
                }
                assert_eq!(*r, Expression::Value(Value::Integer(3)));
            }
            x => panic!("expected mul: {:?}", x),
        }
    }

    #[test] fn test_chained_addition() {
        // a + b + c is left-associative: (a + b) + c
        match expect_parse("a + b + c") {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Reference(Reference{target:"a".into()}));
                assert_eq!(*r, Expression::Reference(Reference{target:"c".into()}));
                match **l {
                    Expression::Binary(ll, BinaryOperator::Add, lr) => {
                        assert_eq!(*ll, Expression::Reference(Reference{target:"b".into()}));
                    }
                    x => panic!("expected inner add: {:?}", x),
                }
            }
            x => panic!("expected outer add: {:?}", x),
        }
    }
}
```

**Verification:** `cargo check` compiles, then `cargo test` passes.

---

### Step 7: Create parser/statement.rs

**File:** `src/machine/parser/statement.rs` — new file

```rust
use pest::iterators::Pair;
use pest::Parser;

use super::{GrammarParser, Rule, ParseError};
use super::expression::expression_from_pair;
use crate::machine::statement::{Statement, AssignmentOperator};

pub fn parse_statement(input: &str) -> Result<Statement, ParseError> {
    let mut pairs = GrammarParser::parse(Rule::statement, input)
        .map_err(|e| ParseError::from_pest(input, e))?;

    let pair = pairs.next().ok_or_else(|| ParseError::new("empty statement"))?;
    statement_from_pair(pair)
}

fn statement_from_pair(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut children = pair.into_inner();

    let target_pair = children.next().ok_or_else(|| ParseError::new("missing target"))?;
    let target = match target_pair.as_rule() {
        Rule::identifier => target_pair.as_str().to_string(),
        _ => return Err(ParseError::new(format!("expected identifier, got {:?}", target_pair.as_rule()))),
    };

    let op_pair = children.next().ok_or_else(|| ParseError::new("missing operator"))?;
    let operator = assignment_op_from_pair(op_pair)?;

    let expr_pair = children.next().ok_or_else(|| ParseError::new("missing expression"))?;
    let expression = expression_from_pair(expr_pair)?;

    if children.next().is_some() {
        return Err(ParseError::new("extra tokens after expression"));
    }

    Ok(Statement { target, operator, expression })
}

fn assignment_op_from_pair(pair: Pair<Rule>) -> Result<AssignmentOperator, ParseError> {
    match pair.as_rule() {
        Rule::ASSIGN            => Ok(AssignmentOperator::Assign),
        Rule::ADD_ASSIGN        => Ok(AssignmentOperator::AddAssign),
        Rule::SUB_ASSIGN        => Ok(AssignmentOperator::SubAssign),
        Rule::MUL_ASSIGN        => Ok(AssignmentOperator::MulAssign),
        Rule::DIV_ASSIGN        => Ok(AssignmentOperator::DivAssign),
        Rule::MOD_ASSIGN        => Ok(AssignmentOperator::ModAssign),
        Rule::AND_ASSIGN        => Ok(AssignmentOperator::AndAssign),
        Rule::OR_ASSIGN         => Ok(AssignmentOperator::OrAssign),
        Rule::XOR_ASSIGN        => Ok(AssignmentOperator::XorAssign),
        Rule::LOGICAL_AND_ASSIGN => Ok(AssignmentOperator::LogicalAndAssign),
        Rule::LOGICAL_OR_ASSIGN  => Ok(AssignmentOperator::LogicalOrAssign),
        Rule::LOGICAL_XOR_ASSIGN => Ok(AssignmentOperator::LogicalXorAssign),
        _ => Err(ParseError::new(format!("unexpected operator rule: {:?}", pair.as_rule()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::statement::AssignmentOperator;
    use crate::machine::expression::{Expression, Reference, BinaryOperator};
    use crate::machine::types::Value;

    fn expect_stmt(input: &str) -> Statement {
        parse_statement(input).unwrap_or_else(|e| panic!("parse failed for '{}': {}", input, e))
    }

    #[test] fn test_simple_assign() {
        let s = expect_stmt("x = 1");
        assert_eq!(s.target, "x");
        assert_eq!(s.operator, AssignmentOperator::Assign);
        assert_eq!(s.expression, Expression::Value(Value::Integer(1)));
    }

    #[test] fn test_add_assign() {
        let s = expect_stmt("y += a + b");
        assert_eq!(s.operator, AssignmentOperator::AddAssign);
        assert_eq!(s.target, "y");
        match s.expression {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Reference(Reference{target:"a".into()}));
                assert_eq!(*r, Expression::Reference(Reference{target:"b".into()}));
            }
            x => panic!("expected add expr: {:?}", x),
        }
    }

    #[test] fn test_logical_and_assign() {
        let s = expect_stmt("flags &&= mask");
        assert_eq!(s.operator, AssignmentOperator::LogicalAndAssign);
    }

    #[test] fn test_precedence_in_statement_expr() {
        let s = expect_stmt("x = 1 + 5 * 2");
        match s.expression {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Value(Value::Integer(1)));
                match *r {
                    Expression::Binary(ll, BinaryOperator::Mul, rr) => {
                        assert_eq!(*ll, Expression::Value(Value::Integer(5)));
                        assert_eq!(*rr, Expression::Value(Value::Integer(2)));
                    }
                    x => panic!("expected mul: {:?}", x),
                }
            }
            x => panic!("expected add: {:?}", x),
        }
    }

    #[test] fn test_unary_in_statement() {
        let s = expect_stmt("y = -1 + 2");
        match s.expression {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                match *l {
                    Expression::Unary(op, opnd) => {
                        assert_eq!(op, UnaryOperator::Negate);
                        assert_eq!(*opnd, Expression::Value(Value::Integer(1)));
                    }
                    x => panic!("expected unary: {:?}", x),
                }
                assert_eq!(*r, Expression::Value(Value::Integer(2)));
            }
            x => panic!("expected add: {:?}", x),
        }
    }
}
```

**Verification:** `cargo test` passes.

---

### Step 8: Create parser/link.rs

**File:** `src/machine/parser/link.rs` — new file

```rust
use pest::iterators::Pair;
use pest::Parser;

use super::{GrammarParser, Rule, ParseError};
use crate::machine::link::Link;

pub fn parse_link(input: &str) -> Result<Link, ParseError> {
    let mut pairs = GrammarParser::parse(Rule::link, input)
        .map_err(|e| ParseError::from_pest(input, e))?;

    let pair = pairs.next().ok_or_else(|| ParseError::new("empty link"))?;
    link_from_pair(pair)
}

fn link_from_pair(pair: Pair<Rule>) -> Result<Link, ParseError> {
    let mut children = pair.into_inner();

    let id_pair = children.next().ok_or_else(|| ParseError::new("link: missing id"))?;
    let id = match id_pair.as_rule() {
        Rule::identifier => id_pair.as_str().to_string(),
        _ => return Err(ParseError::new(format!("expected identifier, got {:?}", id_pair.as_rule()))),
    };

    // The "." is a silent match in the grammar, skip it
    if let Some(dot) = children.next() {
        if dot.as_rule() != Rule::DOT {
            return Err(ParseError::new(format!("expected '.', got {:?}", dot.as_rule())));
        }
    }

    let output_pair = children.next().ok_or_else(|| ParseError::new("link: missing output"))?;
    let output = match output_pair.as_rule() {
        Rule::identifier => output_pair.as_str().to_string(),
        _ => return Err(ParseError::new(format!("expected identifier, got {:?}", output_pair.as_rule()))),
    };

    if children.next().is_some() {
        return Err(ParseError::new("extra tokens in link"));
    }

    Ok(Link(id, output))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect_link(input: &str) -> Link {
        parse_link(input).unwrap_or_else(|e| panic!("parse failed for '{}': {}", input, e))
    }

    #[test]
    fn test_simple_link() {
        let link = expect_link("myid.myoutput");
        assert_eq!(link.id, "myid");
        assert_eq!(link.output, "myoutput");
    }

    #[test]
    fn test_link_with_underscore() {
        let link = expect_link("id_1.output_2");
        assert_eq!(link.id, "id_1");
        assert_eq!(link.output, "output_2");
    }

    #[test]
    fn test_link_error() {
        let result = parse_link("invalid");
        assert!(result.is_err());
    }
}
```

### Step 9: Update mod.rs

**File:** `src/machine/mod.rs` — update to reflect new file structure

```rust
mod test;
mod types;
mod connections;
mod variables;
mod expression;
mod actions;
mod statement;   // NEW
mod link;         // NEW
pub mod parser;   // NEW

pub use types::{Type, Value};
pub use link::Link;
pub use connections::{Input, Output};
pub use variables::{Signal, Timer, Variable, Constant};
pub use expression::{Reference, Expression, BinaryOperator, UnaryOperator};
pub use statement::{Statement, AssignmentOperator};
pub use actions::{Action, Transition, State};
```

### Step 10: Update expression.rs — remove Statement types

**File:** `src/machine/expression.rs` — remove `Statement` and `AssignmentOperator` types (moved to statement.rs), keep `Expression`, `BinaryOperator`, `UnaryOperator`, `Reference`, and the `test_parse_expression` tests.

### Step 11: Update actions.rs — fix Statement import

**File:** `src/machine/actions.rs` — change:
```rust
// OLD:
use super::expression::Statement;

// NEW:
use super::statement::Statement;
```

### Step 12: Update test.rs — fix any needed imports

**File:** `src/machine/test.rs` — no changes needed (imports from `crate::machine::*` which re-exports everything).

### Step 13: Final verification

Run `cargo test` — all tests must pass:
- Original `test_deserialize_machine_minimal`
- Original `test_deserialize_machine`
- All new grammar parse tests in `expression.rs`
- All new statement tests in `statement.rs` parser
- All new link tests in `parser/link.rs`
- Precedence tests: mul>add, or>and, etc.
- Unary tests: -, !, ~
- New operator tests: &&=, ||=, ^^=
- Escape sequence tests in strings
- Octal, binary, hex integer tests

### Step 14: Remove old test stub from expression.rs

Remove the old `test_parse_expression` and `test_parse_statement` test functions from `src/machine/expression.rs` — they're now in the parser sub-modules.

---

## Execution Order Summary

1. **Step 1** — Rewrite `grammar.pest` (independent)
2. **Step 2** — Add `UnaryOperator`, new `BinaryOperator` variants, `Unary` variant to `Expression` in `expression.rs`
3. **Step 3** — Create `statement.rs`, update `mod.rs` imports, fix `actions.rs`
4. **Step 4** — Create `link.rs`, update `mod.rs` imports, update `connections.rs`
5. **Step 5** — Create `parser/mod.rs`
6. **Step 6** — Create `parser/expression.rs`
7. **Step 7** — Create `parser/statement.rs`
8. **Step 8** — Create `parser/link.rs`
9. **Step 9** — Update `mod.rs` final structure
10. **Step 10** — Clean up `expression.rs` (remove moved types)
11. **Step 11** — Fix `actions.rs` import
12. **Step 12** — Verify `test.rs` still works
13. **Step 13** — Run `cargo test` for full verification
14. **Step 14** — Clean up old test stubs
