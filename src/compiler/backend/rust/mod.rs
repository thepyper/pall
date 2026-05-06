// Phase 4: RustBackend implementation (fleshed out in subsequent steps)

use std::collections::HashMap;
use handlebars::Handlebars;

use crate::machine::StateMachine;
use super::super::{Backend, FileSet};
use super::super::codegen;
use super::super::codegen::CodegenContext;
use super::super::error::CompileError;

const MOD_TEMPLATE: &str = include_str!("templates/mod.hbs");
const TYPES_TEMPLATE: &str = include_str!("templates/types.hbs");
const TICK_TEMPLATE: &str = include_str!("templates/tick.hbs");
const GROUP_TEMPLATE: &str = include_str!("templates/group.hbs");

/// Rust backend: compiles state machines into Rust code using Handlebars templates.
pub struct RustBackend {
    handlebars: Handlebars<'static>,
}

impl RustBackend {
    /// Variable name for the Persistent struct parameter in per-machine tick().
    pub const STATE_NAME: &'static str = "x";
    /// Variable name for the Update struct local in per-machine tick().
    pub const UPDATE_NAME: &'static str = "y";
    /// Variable name for the Persistent struct parameter in group tick().
    pub const STATE_GROUP_NAME: &'static str = "xs";
    /// Variable name for the Update struct local in group tick().
    pub const UPDATE_GROUP_NAME: &'static str = "ys";

    pub fn new() -> Self {
        let mut hb = Handlebars::new();
        hb.register_template_string("mod", MOD_TEMPLATE.to_string())
            .expect("failed to register mod template");
        hb.register_template_string("types", TYPES_TEMPLATE.to_string())
            .expect("failed to register types template");
        hb.register_template_string("tick", TICK_TEMPLATE.to_string())
            .expect("failed to register tick template");
        hb.register_template_string("group", GROUP_TEMPLATE.to_string())
            .expect("failed to register group template");
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
        machines: &[StateMachine],
    ) -> Result<FileSet, Vec<CompileError>> {
        let mut files = HashMap::new();

        // ── Per-machine types files ────────────────────────────────────────
        let types_ctx = CodegenContext::new(Self::STATE_NAME, Self::UPDATE_NAME);
        for machine in machines {
            let data = codegen::build_types_data(machine, &types_ctx);
            let content = self.render("types", &data);
            files.insert(format!("{}/types.rs", machine.id), content);
        }

        // ── Per-machine tick files ─────────────────────────────────────────
        let tick_ctx = CodegenContext::new(Self::STATE_NAME, Self::UPDATE_NAME);
        for machine in machines {
            let data = codegen::build_tick_data(machine, &tick_ctx)?;
            let content = self.render("tick", &data);
            files.insert(format!("{}/tick.rs", machine.id), content);
        }

        // ── Group file ─────────────────────────────────────────────────────
        let group_ctx = CodegenContext::new(Self::STATE_GROUP_NAME, Self::UPDATE_GROUP_NAME);
        let tick_ctx = CodegenContext::new(Self::STATE_NAME, Self::UPDATE_NAME);
        let group_data = codegen::build_group_data(machines, &group_ctx, &tick_ctx);
        let group_content = self.render("group", &group_data);
        files.insert("group.rs".to_string(), group_content);

        // ── Module file ────────────────────────────────────────────────────
        let mod_data = codegen::build_mod_data(machines);
        let mod_content = self.render("mod", &mod_data);
        files.insert("mod.rs".to_string(), mod_content);

        Ok(files)
    }
}
