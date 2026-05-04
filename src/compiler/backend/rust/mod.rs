// Phase 4: RustBackend implementation (fleshed out in subsequent steps)

use crate::machine::StateMachine;
use super::super::{Backend, FileSet};
use super::super::error::CompileError;

/// Rust backend: compiles state machines into Rust code using Handlebars templates.
pub struct RustBackend;

impl Backend for RustBackend {
    fn compile(
        &self,
        _machines: &[StateMachine],
    ) -> Result<FileSet, Vec<CompileError>> {
        // Placeholder: code generation will be implemented in Phase 4 steps
        Ok(std::collections::HashMap::new())
    }
}
