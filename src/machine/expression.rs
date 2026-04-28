use serde::{Serialize, Deserialize};
use pest_derive::Parser;
use pest::Parser;

use super::types::Value;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum BinaryOperator
{
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    BitAnd,
    BitOr,
    BitXor,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Reference {
    pub target: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Expression
{
    Value(Value),
    Reference(Reference),
    Parenthesis(Box<Expression>),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum AssignmentOperator
{
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Statement
{
    pub target: String,
    pub operator: AssignmentOperator,
    pub expression: Expression,
}

#[derive(Parser)]
#[grammar = "machine/expression.pest"]
struct ExpressionParser;




#[test]
fn test_parse_expression()
{
    eprintln!("1. {:?}", ExpressionParser::parse(Rule::expression,  r###"1"###).unwrap());
    eprintln!("2. {:?}", ExpressionParser::parse(Rule::expression,  r###"(1)"###).unwrap());
    eprintln!("3. {:?}", ExpressionParser::parse(Rule::expression,  r###"1 + 2"###).unwrap());
    eprintln!("4. {:?}", ExpressionParser::parse(Rule::expression,  r###"(1 + 3)"###).unwrap());
    eprintln!("5. {:?}", ExpressionParser::parse(Rule::expression,  r###"1 + (4)"###).unwrap());
    eprintln!("6. {:?}", ExpressionParser::parse(Rule::expression,  r###"1 + (5 + 6)"###).unwrap());
    eprintln!("7. {:?}", ExpressionParser::parse(Rule::expression,  r###"(1 + 7) + (8 - 9)"###).unwrap());
    eprintln!("8. {:?}", ExpressionParser::parse(Rule::expression,  r###"(1 + (2 + (3 + 4))) + 8"###).unwrap());
    eprintln!("9. {:?}", ExpressionParser::parse(Rule::expression,  r###""hahaha""###).unwrap());
    eprintln!("10. {:?}", ExpressionParser::parse(Rule::expression, r###""1 + 2""###).unwrap());
    eprintln!("11. {:?}", ExpressionParser::parse(Rule::expression, r###""(3)""###).unwrap());
    eprintln!("12. {:?}", ExpressionParser::parse(Rule::expression, r###"'aaaa'"###).unwrap());
    eprintln!("13. {:?}", ExpressionParser::parse(Rule::expression, r###"'1 + 2'"###).unwrap());
    eprintln!("14. {:?}", ExpressionParser::parse(Rule::expression, r###"'(3)'"###).unwrap());
    eprintln!("15. {:?}", ExpressionParser::parse(Rule::expression, r###""'''''""###).unwrap());
    eprintln!("16. {:?}", ExpressionParser::parse(Rule::expression, r###"'"""""'"###).unwrap());
    eprintln!("17. {:?}", ExpressionParser::parse(Rule::expression, r###"id1"###).unwrap());
    eprintln!("18. {:?}", ExpressionParser::parse(Rule::expression, r###"_id2"###).unwrap());
    eprintln!("19. {:?}", ExpressionParser::parse(Rule::expression, r###"id_3"###).unwrap());
    eprintln!("20. {:?}", ExpressionParser::parse(Rule::expression, r###"a + b + c + d + e"###).unwrap());
}
