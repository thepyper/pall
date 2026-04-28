use std::collections::HashMap;
use serde::{Serialize, Deserialize};

mod test;
mod parse;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Type
{
    Bool,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F32, F64,
    String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Value
{
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Link
{
    pub id: String,
    pub output: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Input
{
    pub r#type: Type,
    pub link: Option<Link>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Output
{
    pub r#type: Type,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Signal
{
    pub r#type: Type,
    pub when: Expression,
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
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Constant
{
    pub r#type: Type,
    pub value: Value,
}

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
struct Reference {
    pub target: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Expression
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Action
{
    pub when: Option<Expression>,
    #[serde(rename = "do")]
    pub r#do: Vec<Statement>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Transition
{
    pub when: Option<Expression>,
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct StateMachine
{
    pub id: String,
    pub initial: Option<String>,            /// default: "initial"
    pub states: HashMap<String, State>,
    #[serde(default)]
    pub inputs: HashMap<String, Input>,
    #[serde(default)]
    pub outputs: HashMap<String, Output>,
    #[serde(default)]
    pub signals: HashMap<String, Signal>,
    #[serde(default)]
    pub timers: HashMap<String, Timer>,
    #[serde(default)]
    pub variables: HashMap<String, Variable>,
    #[serde(default)]
    pub constants: HashMap<String, Constant>,
}
