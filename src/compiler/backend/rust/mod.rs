// Phase 4: RustBackend implementation (fleshed out in subsequent steps)

use std::collections::HashMap;
use handlebars::Handlebars;

use crate::machine::StateMachine;
use super::super::{Backend, FileSet};
use super::super::error::CompileError;

const MOD_TEMPLATE: &str = include_str!("templates/mod.hbs");
const TYPES_TEMPLATE: &str = include_str!("templates/types.hbs");
const TICK_TEMPLATE: &str = include_str!("templates/tick.hbs");

/// Rust backend: compiles state machines into Rust code using Handlebars templates.
pub struct RustBackend {
    handlebars: Handlebars<'static>,
}

impl RustBackend {
    pub fn new() -> Self {
        let mut hb = Handlebars::new();
        hb.register_template_string("mod", MOD_TEMPLATE.to_string())
            .expect("failed to register mod template");
        hb.register_template_string("types", TYPES_TEMPLATE.to_string())
            .expect("failed to register types template");
        hb.register_template_string("tick", TICK_TEMPLATE.to_string())
            .expect("failed to register tick template");
        Self { handlebars: hb }
    }

    fn render(&self, name: &str, data: &serde_json::Value) -> String {
        self.handlebars
            .render(name, data)
            .unwrap_or_else(|e| format!("// Template error: {}", e))
    }
}

impl Default for RustBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for RustBackend {
    fn compile(
        &self,
        _machines: &[StateMachine],
    ) -> Result<FileSet, Vec<CompileError>> {
        // Placeholder: code generation will be implemented in Phase 4 steps
        Ok(HashMap::new())
    }
}
