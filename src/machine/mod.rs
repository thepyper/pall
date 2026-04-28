
pub enum Type
{
    Bool,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F32, F64,
    String,
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

pub struct Action
{

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
}
