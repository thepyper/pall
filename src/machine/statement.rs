use serde::{Deserialize, Serialize, Serializer};
use super::expression::Expression;
use super::parser::ParseError;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Statement {
    pub target: String,
    pub operator: AssignmentOperator,
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub struct FullStatement {
    pub raw: String,
    pub statement: Statement,
}

impl FullStatement {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let statement = super::parser::parse_statement(input)?;
        Ok(Self { raw: input.to_string(), statement })
    }
}

impl Serialize for FullStatement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw)
    }
}
