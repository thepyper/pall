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
use typecheck::infer_all;
use type_validation::validate_types;

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
        // Phase 1: Type inference
        let type_results = infer_all(machines);
        
        // Check for inference errors
        for (_env, errors) in type_results.iter() {
            if !errors.is_empty() {
                return Err(errors.to_vec());
            }
        }
        
        // Phase 2: Type validation
        let type_errors = validate_types(machines, &type_results);
        if !type_errors.is_empty() {
            return Err(type_errors);
        }
        
        // Phase 3: Original validation
        validate_machines(machines)?;
        
        // Phase 4: Code generation (backend)
        self.backend.compile(machines)
    }
}
