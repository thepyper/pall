pub mod rust;

use crate::machine::StateMachine;

use super::FileSet;

use crate::compiler::error::CompileError;

pub trait Backend: Sync {
    fn compile(&self, machines: &[StateMachine]) -> Result<FileSet, CompileError>;
}
