
pub enum Type
{
    Bool,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F32, F64,
    String,
}

pub enum Value
{
    Integer(i64),
    Float(f64),
    String(String),
}

pub struct Link
{
    pub machine: String,
    pub output: String,
}

pub struct Input
{
    pub type: Type,
    pub link: Option<Link>,
}

pub struct Output
{
    pub type: Type,
}

pub struct Signal
{
    pub type: Type,
    pub when: Expression,
}

pub struct Timer
{
    pub type: Type,
    pub when: Option<Expression>,
}

pub struct Variable
{
    pub type: Type,
    pub initial: Option<Value>,
}

pub struct Constant
{
    pub type: Type,
    pub value: Value,
}

pub struct Action
{
    pub do: Statement,
}

pub struct State
{
    pub actions: Vec<Action>,
    pub transitions: Vec<Transition>,
}

pub struct StateMachine
{
    pub states: HashMap<String, State>,
    pub inputs: HashMap<String, Input>,
    pub outputs: HashMap<String, Output>,
    pub signals: HashMap<String, Signal>,
    pub timers: HashMap<String, Timer>,
    pub variables: HashMap<String, Variable>,
    pub constants: HashMap<String, Constant>,
}
