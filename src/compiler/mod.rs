pub mod error;
pub mod validation;
pub mod typecheck_rules;
pub mod typecheck;
pub mod type_validation;
pub mod backend;

pub use backend::rust::RustBackend;
pub use error::{CompileError, CompileErrorKind, TickError, TickErrorKind};
pub use backend::Backend;

pub type FileSet = std::collections::HashMap<String, String>;

pub struct TickInfo {
    pub delta_ms: u64,
}

use crate::machine::StateMachine;
use validation::validate_machines;

/// Compiler orchestrator: validates machines, then delegates to the backend.
pub struct Compiler<B: Backend> {
    backend: B,
}

impl<B: Backend> Compiler<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn compile(
        &self,
        machines: &[StateMachine],
    ) -> Result<FileSet, Vec<CompileError>> {
        // Run validation first
        validate_machines(machines)?;
        // Then delegate to backend
        self.backend.compile(machines)
    }
}
