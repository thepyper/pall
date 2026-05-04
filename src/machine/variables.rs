use serde::{Serialize, Deserialize};

use super::types::{Type, Value};
use super::expression::Expression;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Signal
{
    pub r#type: Type,
    #[serde(default)]
    pub output: bool,
    pub expr: Expression,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Timer
{
    pub r#type: Type,
    pub when: Option<Expression>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Variable
{
    pub r#type: Type,
    pub initial: Option<Value>,
    #[serde(default)]
    pub output: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Constant
{
    pub r#type: Type,
    pub value: Value,
}
