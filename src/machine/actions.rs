use serde::{Serialize, Deserialize};

use super::statement::Statement;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Action
{
    pub when: Option<super::expression::Expression>,
    #[serde(rename = "do")]
    pub r#do: Vec<Statement>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Transition
{
    pub when: Option<super::expression::Expression>,
    #[serde(default, rename = "do")]
    pub r#do: Vec<Statement>,
    pub target: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct State
{
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
}
