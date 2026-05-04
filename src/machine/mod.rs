mod test;
mod types;
mod connections;
mod variables;
mod expression;
mod actions;
mod statement;
mod link;
pub mod parser;

pub use types::{Type, Value, IntegerValue, FloatValue, StringValue, IntegerFmt, FloatFmt, StringFmt};
pub use link::Link;
pub use connections::Input;
pub use variables::{Signal, Timer, Variable, Constant};
pub use expression::{Reference, Expression, BinaryOperator, UnaryOperator, FullExpression};
pub use statement::{Statement, AssignmentOperator, FullStatement};
pub use actions::{Action, Transition, State};

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct StateMachine
{
    pub id: String,
    pub initial: Option<String>,            /// default: "initial"
    pub states: HashMap<String, State>,
    #[serde(default)]
    pub inputs: HashMap<String, Input>,

    #[serde(default)]
    pub signals: HashMap<String, Signal>,
    #[serde(default)]
    pub timers: HashMap<String, Timer>,
    #[serde(default)]
    pub variables: HashMap<String, Variable>,
    #[serde(default)]
    pub constants: HashMap<String, Constant>,
}
