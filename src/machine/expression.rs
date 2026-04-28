use serde::{Serialize, Deserialize};

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
