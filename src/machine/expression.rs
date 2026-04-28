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
    eprintln!("1. {:?}", ExpressionParser::parse(Rule::expression, r###"1"###).unwrap());
    eprintln!("2. {:?}", ExpressionParser::parse(Rule::expression, r###"(1)"###).unwrap());
    eprintln!("3. {:?}", ExpressionParser::parse(Rule::expression, r###"1 + 2"###).unwrap());
    eprintln!("4. {:?}", ExpressionParser::parse(Rule::expression, r###"(1 + 3)"###).unwrap());
    eprintln!("5. {:?}", ExpressionParser::parse(Rule::expression, r###"1 + (4)"###).unwrap());
    eprintln!("6. {:?}", ExpressionParser::parse(Rule::expression, r###"1 + (5 + 6)"###).unwrap());
    eprintln!("7. {:?}", ExpressionParser::parse(Rule::expression, r###"(1 + 7) + (8 - 9)"###).unwrap());
    eprintln!("8. {:?}", ExpressionParser::parse(Rule::expression, r###"(1 + (2 + (3 + 4))) + 8"###).unwrap());
}
