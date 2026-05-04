pub mod error;
pub mod validation;
pub mod backend;
pub mod codegen;

pub use error::{CompileError, CompileErrorKind, TickError, TickErrorKind};
pub use backend::Backend;

pub type FileSet = std::collections::HashMap<String, String>;

pub struct TickInfo {
    pub delta_ms: u64,
}
