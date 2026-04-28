use pest::iterators::Pair;
use pest::Parser;

use super::{GrammarParser, Rule, ParseError};
use super::expression::expression_from_pair;
use crate::machine::statement::{Statement, AssignmentOperator};

/// Parse a plain string into a Statement AST.
pub fn parse_statement(input: &str) -> Result<Statement, ParseError> {
    let mut pairs = GrammarParser::parse(Rule::statement, input)
        .map_err(|e| ParseError::from_pest(input, e))?;

    let pair = pairs.next()
        .ok_or_else(|| ParseError::new("empty statement"))?;
    statement_from_pair(pair)
}

fn statement_from_pair(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut children = pair.into_inner();

    // First child: identifier (target)
    let target_pair = children.next()
        .ok_or_else(|| ParseError::new("statement: missing target"))?;
    let target = match target_pair.as_rule() {
        Rule::identifier => target_pair.as_str().trim().to_string(),
        _ => return Err(ParseError::new(format!(
            "statement: expected identifier for target, got {:?}",
            target_pair.as_rule()
        ))),
    };

    // Second child: assignment operator
    let op_pair = children.next()
        .ok_or_else(|| ParseError::new("statement: missing operator"))?;
    let operator = assignment_op_from_pair(op_pair)?;

    // Third child: expression
    let expr_pair = children.next()
        .ok_or_else(|| ParseError::new("statement: missing expression"))?;
    let expression = expression_from_pair(expr_pair)?;

    // No more children expected
    // EOI/SOI may appear as empty pairs, skip them
    for child in children {
        if child.as_rule() != Rule::EOI {
            return Err(ParseError::new("statement: extra tokens after expression"));
        }
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
        _ => Err(ParseError::new(format!(
            "unexpected assignment operator rule: {:?}", pair.as_rule()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::statement::AssignmentOperator;
    use crate::machine::expression::{Expression, Reference, BinaryOperator, UnaryOperator};
    use crate::machine::types::Value;

    fn expect_stmt(input: &str) -> Statement {
        parse_statement(input).unwrap_or_else(|e| panic!("parse failed for '{}': {}", input, e))
    }

    #[test]
    fn test_simple_assign() {
        let s = expect_stmt("x = 1");
        assert_eq!(s.target, "x");
        assert_eq!(s.operator, AssignmentOperator::Assign);
        assert_eq!(s.expression, Expression::Value(Value::Integer(1)));
    }

    #[test]
    fn test_add_assign_with_expr() {
        let s = expect_stmt("y += a + b");
        assert_eq!(s.target, "y");
        assert_eq!(s.operator, AssignmentOperator::AddAssign);
        match s.expression {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Reference(Reference{target:"a".into()}));
                assert_eq!(*r, Expression::Reference(Reference{target:"b".into()}));
            }
            x => panic!("expected add expr: {:?}", x),
        }
    }

    #[test]
    fn test_logical_and_assign() {
        let s = expect_stmt("flags &&= mask");
        assert_eq!(s.target, "flags");
        assert_eq!(s.operator, AssignmentOperator::LogicalAndAssign);
    }

    #[test]
    fn test_logical_or_assign() {
        let s = expect_stmt("flags ||= mask");
        assert_eq!(s.operator, AssignmentOperator::LogicalOrAssign);
    }

    #[test]
    fn test_logical_xor_assign() {
        let s = expect_stmt("flags ^^= mask");
        assert_eq!(s.operator, AssignmentOperator::LogicalXorAssign);
    }

    #[test]
    fn test_precedence_in_expression() {
        // x = 1 + 5 * 2 — multiplication binds tighter
        let s = expect_stmt("x = 1 + 5 * 2");
        match s.expression {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Value(Value::Integer(1)));
                match *r {
                    Expression::Binary(ll, BinaryOperator::Mul, rr) => {
                        assert_eq!(*ll, Expression::Value(Value::Integer(5)));
                        assert_eq!(*rr, Expression::Value(Value::Integer(2)));
                    }
                    x => panic!("expected mul on right: {:?}", x),
                }
            }
            x => panic!("expected add: {:?}", x),
        }
    }

    #[test]
    fn test_unary_in_statement() {
        // y = -1 + 2
        let s = expect_stmt("y = -1 + 2");
        match s.expression {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                match *l {
                    Expression::Unary(UnaryOperator::Negate, opnd) => {
                        assert_eq!(*opnd, Expression::Value(Value::Integer(1)));
                    }
                    x => panic!("expected unary negate: {:?}", x),
                }
                assert_eq!(*r, Expression::Value(Value::Integer(2)));
            }
            x => panic!("expected add: {:?}", x),
        }
    }

    #[test]
    fn test_bitwise_and_assign() {
        let s = expect_stmt("y &= 1 + 5");
        assert_eq!(s.operator, AssignmentOperator::AndAssign);
    }

    #[test]
    fn test_complex_expr() {
        // z /= 9 + (5 * y)
        let s = expect_stmt("z /= 9 + (5 * y)");
        assert_eq!(s.operator, AssignmentOperator::DivAssign);
        assert_eq!(s.target, "z");
        match s.expression {
            Expression::Binary(l, BinaryOperator::Add, r) => {
                assert_eq!(*l, Expression::Value(Value::Integer(9)));
                match *r {
                    Expression::Parenthesis(inner) => {
                        match *inner {
                            Expression::Binary(ll, BinaryOperator::Mul, lr) => {
                                assert_eq!(*ll, Expression::Value(Value::Integer(5)));
                                assert_eq!(*lr, Expression::Reference(Reference{target:"y".into()}));
                            }
                            x => panic!("expected mul in parens: {:?}", x),
                        }
                    }
                    x => panic!("expected paren on right: {:?}", x),
                }
            }
            x => panic!("expected add: {:?}", x),
        }
    }
}
