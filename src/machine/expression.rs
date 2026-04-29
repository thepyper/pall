use serde::{Deserialize, Serialize, Deserializer, Serializer};

use super::parser::ParseError;
use super::types::Value;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum UnaryOperator {
    Negate,   // -
    Not,      // !
    BitNot,   // ~
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum BinaryOperator {
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
    LogicalOr,    // ||
    LogicalAnd,   // &&
    LogicalXor,   // ^^
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Reference {
    pub target: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Expression {
    Value(Value),
    Reference(Reference),
    Parenthesis(Box<Expression>),
    Unary(UnaryOperator, Box<Expression>),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FullExpression {
    pub raw: String,
    pub expression: Expression,
}

impl FullExpression {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let expression = super::parser::parse_expression(input)?;
        Ok(Self { raw: input.to_string(), expression })
    }
}

impl Serialize for FullExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> Deserialize<'de> for FullExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        FullExpression::parse(&raw)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
