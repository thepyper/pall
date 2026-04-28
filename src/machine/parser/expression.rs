use pest::iterators::Pair;
use pest::Parser;

use super::{GrammarParser, Rule, ParseError};
use crate::machine::types::Value;
use crate::machine::expression::{Expression, Reference, UnaryOperator, BinaryOperator};

/// Parse a plain string into an Expression AST.
pub fn parse_expression(input: &str) -> Result<Expression, ParseError> {
    let mut pairs = GrammarParser::parse(Rule::full_expression, input)
        .map_err(|e| ParseError::from_pest(input, e))?;

    let pair = pairs.next().ok_or_else(|| ParseError::new("empty expression"))?;
    expression_from_pair(pair)
}

/// Recursively convert a pest pair to Expression AST.
/// This follows the grammar's precedence levels exactly.
pub(crate) fn expression_from_pair(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    match pair.as_rule() {
        // Top-level — unwrap the full_expression or expression wrapper
        Rule::full_expression | Rule::expression => {
            let mut inner = pair.into_inner();
            let child = inner.next().ok_or_else(|| ParseError::new("empty expression"))?;
            expression_from_pair(child)
        }

        // Unary: prefix operators -, !, ~
        Rule::unary => {
            let children: Vec<Pair<Rule>> = pair.into_inner().collect();
            if !children.is_empty() {
                if matches!(children[0].as_rule(), Rule::SUB | Rule::NOT | Rule::BIT_NOT) {
                    let op = unary_op_from_pair(children[0].clone())?;
                    if children.len() < 2 {
                        return Err(ParseError::new("unary operator without operand"));
                    }
                    let operand = expression_from_pair(children[1].clone())?;
                    return Ok(Expression::Unary(op, Box::new(operand)));
                }
            }
            // No prefix operator, just recurse
            if children.is_empty() {
                return Err(ParseError::new("unary rule with no children"));
            }
            expression_from_pair(children[0].clone())
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

        // Primary: value or parenthesized expression
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
            expression_from_pair(child).map(|e| Expression::Parenthesis(Box::new(e)))
        }

        // Integer types
        Rule::dec_integer => Ok(Expression::Value(Value::Integer(parse_integer(pair.as_str().trim())))),
        Rule::hex_integer => {
            let raw = pair.as_str().trim();
            let val = i64::from_str_radix(&raw[2..], 16)
                .map_err(|_| ParseError::new(format!("invalid hex: {}", raw)))?;
            Ok(Expression::Value(Value::Integer(val)))
        }
        Rule::oct_integer => {
            let raw = pair.as_str().trim();
            let val = i64::from_str_radix(&raw[2..], 8)
                .map_err(|_| ParseError::new(format!("invalid octal: {}", raw)))?;
            Ok(Expression::Value(Value::Integer(val)))
        }
        Rule::bin_integer => {
            let raw = pair.as_str().trim();
            let val = i64::from_str_radix(&raw[2..], 2)
                .map_err(|_| ParseError::new(format!("invalid binary: {}", raw)))?;
            Ok(Expression::Value(Value::Integer(val)))
        }

        // Float
        Rule::float => {
            let s = pair.as_str().trim();
            let val = s.parse::<f64>()
                .map_err(|_| ParseError::new(format!("invalid float: {}", s)))?;
            Ok(Expression::Value(Value::Float(val)))
        },

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
            target: pair.as_str().trim().to_string(),
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

    #[test]
    fn test_literal_int() {
        assert_eq!(expect_parse("42"), Expression::Value(Value::Integer(42)));
    }

    #[test]
    fn test_literal_hex() {
        assert_eq!(expect_parse("0xff"), Expression::Value(Value::Integer(255)));
    }

    #[test]
    fn test_literal_octal() {
        assert_eq!(expect_parse("0o17"), Expression::Value(Value::Integer(15)));
    }

    #[test]
    fn test_literal_binary() {
        assert_eq!(expect_parse("0b1010"), Expression::Value(Value::Integer(10)));
    }

    #[test]
    fn test_literal_float() {
        assert_eq!(expect_parse("3.14"), Expression::Value(Value::Float(3.14)));
    }

    #[test]
    fn test_literal_string_dq() {
        assert_eq!(expect_parse("\"hello\""), Expression::Value(Value::String("hello".into())));
    }

    #[test]
    fn test_literal_string_sq() {
        assert_eq!(expect_parse("'world'"), Expression::Value(Value::String("world".into())));
    }

    #[test]
    fn test_literal_string_escape() {
        assert_eq!(expect_parse("\"a\\nb\""), Expression::Value(Value::String("a\nb".into())));
    }

    #[test]
    fn test_reference() {
        assert_eq!(expect_parse("my_var"), Expression::Reference(Reference{target:"my_var".into()}));
    }

    #[test]
    fn test_addition() {
        match expect_parse("1 + 2") {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Value(Value::Integer(1)));
                assert_eq!(*r, Expression::Value(Value::Integer(2)));
            }
            x => panic!("expected binary add, got {:?}", x),
        }
    }

    #[test]
    fn test_precedence_mul_over_add() {
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

    #[test]
    fn test_unary_negate() {
        match expect_parse("-5") {
            Expression::Unary(UnaryOperator::Negate, op) => {
                assert_eq!(*op, Expression::Value(Value::Integer(5)));
            }
            x => panic!("expected unary negate: {:?}", x),
        }
    }

    #[test]
    fn test_unary_not() {
        match expect_parse("!flag") {
            Expression::Unary(UnaryOperator::Not, op) => {
                assert_eq!(*op, Expression::Reference(Reference{target:"flag".into()}));
            }
            x => panic!("expected unary not: {:?}", x),
        }
    }

    #[test]
    fn test_unary_bitnot() {
        match expect_parse("~x") {
            Expression::Unary(UnaryOperator::BitNot, op) => {
                assert_eq!(*op, Expression::Reference(Reference{target:"x".into()}));
            }
            x => panic!("expected unary bitnot: {:?}", x),
        }
    }

    #[test]
    fn test_logical_or() {
        match expect_parse("a || b") {
            Expression::Binary(l, BinaryOperator::LogicalOr, r) => {
                assert_eq!(*l, Expression::Reference(Reference{target:"a".into()}));
                assert_eq!(*r, Expression::Reference(Reference{target:"b".into()}));
            }
            x => panic!("expected logical or: {:?}", x),
        }
    }

    #[test]
    fn test_precedence_all_levels() {
        // Full precedence chain: || && ^^ | ^ & == != < <= > > + - * / %
        let expr = expect_parse("1 || 2 && 3 ^^ 4 | 5 ^ 6 & 7 == 8 != 9 <= 10 >= 11 < 12 > 13 + 14 - 15 * 16 / 17 % 18");
        // Top level should be logical_or
        match expr {
            Expression::Binary(left, BinaryOperator::LogicalOr, right) => {
                assert_eq!(*left, Expression::Value(Value::Integer(1)));
                assert!(matches!(*right, Expression::Binary(_, _, _)));
            }
            x => panic!("expected || at top: {:?}", x),
        }
    }

    #[test]
    fn test_parentheses() {
        // (1 + 2) * 3 → (1+2) * 3
        match expect_parse("(1 + 2) * 3") {
            Expression::Binary(l, BinaryOperator::Mul, r) => {
                match *l {
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

    #[test]
    fn test_chained_addition() {
        // a + b + c is left-associative: (a + b) + c
        // Outer: Binary( Binary(a, +, b), +, c )
        match expect_parse("a + b + c") {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                // Right operand is 'c'
                assert_eq!(*r, Expression::Reference(Reference{target:"c".into()}));
                // Left operand is 'a + b'
                match *l {
                    Expression::Binary(ll, BinaryOperator::Add, _lr) => {
                        assert_eq!(*ll, Expression::Reference(Reference{target:"a".into()}));
                    }
                    x => panic!("expected inner add: {:?}", x),
                }
            }
            x => panic!("expected outer add: {:?}", x),
        }
    }

    #[test]
    fn test_precedence_or_over_and() {
        // a || b && c → a || (b && c)
        match expect_parse("a || b && c") {
            Expression::Binary(l, BinaryOperator::LogicalOr, r) => {
                assert_eq!(*l, Expression::Reference(Reference{target:"a".into()}));
                match *r {
                    Expression::Binary(ll, BinaryOperator::LogicalAnd, lr) => {
                        assert_eq!(*ll, Expression::Reference(Reference{target:"b".into()}));
                        assert_eq!(*lr, Expression::Reference(Reference{target:"c".into()}));
                    }
                    x => panic!("expected && on right: {:?}", x),
                }
            }
            x => panic!("expected || at top: {:?}", x),
        }
    }

    #[test]
    fn test_precedence_bitwise_over_logical() {
        // a | b || c → a | b || c → (a | b) || c
        match expect_parse("a | b || c") {
            Expression::Binary(_, BinaryOperator::LogicalOr, _) => {} // top is ||
            x => panic!("expected || at top: {:?}", x),
        }
    }

    #[test]
    fn test_unary_precedence() {
        // -a * b → (-a) * b (unary binds tighter than mul)
        match expect_parse("-a * b") {
            Expression::Binary(l, BinaryOperator::Mul, r) => {
                match *l {
                    Expression::Unary(UnaryOperator::Negate, op) => {
                        assert_eq!(*op, Expression::Reference(Reference{target:"a".into()}));
                    }
                    x => panic!("expected unary negate: {:?}", x),
                }
                assert_eq!(*r, Expression::Reference(Reference{target:"b".into()}));
            }
            x => panic!("expected mul at top: {:?}", x),
        }
    }

    #[test]
    fn test_negation_with_comparison() {
        // -x < 0 → (-x) < 0
        match expect_parse("-x < 0") {
            Expression::Binary(l, BinaryOperator::LessThan, r) => {
                match *l {
                    Expression::Unary(UnaryOperator::Negate, op) => {
                        assert_eq!(*op, Expression::Reference(Reference{target:"x".into()}));
                    }
                    x => panic!("expected unary negate: {:?}", x),
                }
                assert_eq!(*r, Expression::Value(Value::Integer(0)));
            }
            x => panic!("expected <: {:?}", x),
        }
    }
}
