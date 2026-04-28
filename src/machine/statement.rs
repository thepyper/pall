use serde::{Deserialize, Serialize};
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
