pub struct Persistent; pub enum State; pub fn init() -> Persistent { Persistent } pub fn tick(_: &Persistent, _: &super::TickInfo) -> Result<Persistent, super::error::TickError> { Ok(Persistent) }
