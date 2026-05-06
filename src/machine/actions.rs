use serde::{Serialize, Deserialize};

use super::expression::FullExpression;
use super::statement::FullStatement;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Action
{
    pub when: Option<FullExpression>,
    #[serde(rename = "do")]
    pub r#do: Vec<FullStatement>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Transition
{
    pub when: Option<FullExpression>,
    #[serde(default, rename = "do")]
    pub r#do: Vec<FullStatement>,
    pub target: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct State
{
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
}
