use std::collections::HashMap;

use crate::machine::{
    Action, BinaryOperator, Constant, Expression, FullExpression, FullStatement,
    Input, Signal, Statement, Timer, UnaryOperator, Value,
};

use super::error::CompileError;

/// Build a JSON data context for a single machine's types template.
pub fn build_types_data(machine: &crate::machine::StateMachine) -> serde_json::Value {
    let mut inputs = vec![];
    for (name, input) in &machine.inputs {
        let ident = safe_ident(name);
        inputs.push(serde_json::json!({
            "name": ident,
            "rust_type": type_to_rust(&input.r#type),
            "output": input.output,
        }));
    }

    let mut variables = vec![];
    for (name, var) in &machine.variables {
        let ident = safe_ident(name);
        let default = if let Some(ref init) = var.initial {
            value_to_literal(init)
        } else {
            default_value_for_type(&var.r#type)
        };
        variables.push(serde_json::json!({
            "name": ident,
            "rust_type": type_to_rust(&var.r#type),
            "default_value": default,
            "output": var.output,
        }));
    }

    let mut signals = vec![];
    for (name, sig) in &machine.signals {
        let ident = safe_ident(name);
        signals.push(serde_json::json!({
            "name": ident,
            "rust_type": type_to_rust(&sig.r#type),
            "output": sig.output,
        }));
    }

    let mut timers = vec![];
    for (name, timer) in &machine.timers {
        let ident = safe_ident(name);
        timers.push(serde_json::json!({
            "name": ident,
            "rust_type": type_to_rust(&timer.r#type),
        }));
    }

    let mut constants = vec![];
    for (name, constant) in &machine.constants {
        let ident = safe_ident(name);
        constants.push(serde_json::json!({
            "name": ident,
            "rust_type": type_to_rust(&constant.r#type),
            "value": value_to_literal_typed(&constant.value, &constant.r#type),
            "output": constant.output,
        }));
    }

    serde_json::json!({
        "machine_id": machine.id,
        "initial": machine.initial.clone().unwrap_or_else(|| "initial".to_string()),
        "inputs": inputs,
        "variables": variables,
        "signals": signals,
        "timers": timers,
        "constants": constants,
    })
}

/// Build a JSON data context for a single machine's tick template.
pub fn build_tick_data(
    machine: &crate::machine::StateMachine,
) -> Result<serde_json::Value, Vec<CompileError>> {
    let mut errors = Vec::new();

    let initial = machine.initial.clone().unwrap_or_else(|| "initial".to_string());

    // Build field identifiers for expr_to_rust
    let mut field_list: Vec<String> = Vec::new();
    for (name, _) in &machine.inputs {
        field_list.push(safe_ident(name));
    }
    for (name, _) in &machine.variables {
        field_list.push(safe_ident(name));
    }
    for (name, _) in &machine.signals {
        field_list.push(safe_ident(name));
    }
    for (name, _) in &machine.timers {
        field_list.push(safe_ident(name));
    }
    for (name, _) in &machine.constants {
        field_list.push(safe_ident(name));
    }

    let mut states = vec![];
    for (state_name, state) in &machine.states {
        let mut actions_json = vec![];
        for action in &state.actions {
            let when_code = match &action.when {
                Some(expr) => match condition_to_rust(expr, &field_list) {
                    Ok(code) => code,
                    Err(e) => {
                        errors.push(e);
                        "false".to_string()
                    }
                },
                None => String::new(),
            };

            let mut stmts = vec![];
            for stmt in &action.r#do {
                match stmt_to_rust(stmt, &field_list) {
                    Ok(code) => stmts.push(code),
                    Err(e) => errors.push(e),
                }
            }

            actions_json.push(serde_json::json!({
                "when_rust_code": when_code,
                "statements": stmts,
            }));
        }

        let mut transitions_json = vec![];
        for trans in &state.transitions {
            let when_code = match &trans.when {
                Some(expr) => match condition_to_rust(expr, &field_list) {
                    Ok(code) => code,
                    Err(e) => {
                        errors.push(e);
                        "false".to_string()
                    }
                },
                None => String::new(),
            };

            let mut stmts = vec![];
            for stmt in &trans.r#do {
                match stmt_to_rust(stmt, &field_list) {
                    Ok(code) => stmts.push(code),
                    Err(e) => errors.push(e),
                }
            }

            transitions_json.push(serde_json::json!({
                "when_rust_code": when_code,
                "statements": stmts,
                "target": trans.target,
            }));
        }

        states.push(serde_json::json!({
            "name": state_name,
            "actions": actions_json,
            "transitions": transitions_json,
        }));
    }

    let mut signals_json = vec![];
    for (name, sig) in &machine.signals {
        let expr_code = match expr_to_rust(&sig.expr, &field_list) {
            Ok(code) => code,
            Err(e) => {
                errors.push(e);
                "0".to_string()
            }
        };
        signals_json.push(serde_json::json!({
            "name": safe_ident(name),
            "rust_type": type_to_rust(&sig.r#type),
            "expr_rust_code": expr_code,
        }));
    }

    let mut timers_json = vec![];
    for (name, timer) in &machine.timers {
        let when_code = match &timer.when {
            Some(expr) => match expr_to_rust(expr, &field_list) {
                Ok(code) => code,
                Err(e) => {
                    errors.push(e);
                    "false".to_string()
                }
            },
            None => String::new(),
        };
        timers_json.push(serde_json::json!({
            "name": safe_ident(name),
            "rust_type": type_to_rust(&timer.r#type),
            "when_rust_code": when_code,
        }));
    }

    let mut constants_json = vec![];
    for (name, constant) in &machine.constants {
        constants_json.push(serde_json::json!({
            "name": safe_ident(name),
            "rust_type": type_to_rust(&constant.r#type),
            "value": format!("{}", safe_ident(name)),
        }));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(serde_json::json!({
        "machine_id": machine.id,
        "initial": initial,
        "states": states,
        "signals": signals_json,
        "timers": timers_json,
        "constants": constants_json,
    }))
}

/// Build a JSON data context for the mod.rs template.
pub fn build_mod_data(machines: &[crate::machine::StateMachine]) -> serde_json::Value {
    let machines_json: Vec<serde_json::Value> = machines
        .iter()
        .map(|m| serde_json::json!({ "id": m.id }))
        .collect();
    serde_json::json!({ "machines": machines_json })
}

/// Build a JSON data context for the group.rs template.
pub fn build_group_data(machines: &[crate::machine::StateMachine]) -> serde_json::Value {
    let mut machine_fields = vec![];
    let mut machine_ticks = vec![];
    let mut link_assignments = vec![];

    for machine in machines {
        machine_fields.push(serde_json::json!({
            "id": machine.id,
            "types_ref": format!("{}", machine.id),
        }));
        machine_ticks.push(serde_json::json!({
            "id": machine.id,
            "types_ref": format!("{}", machine.id),
        }));

        // Generate link propagation code
        for (input_name, input) in &machine.inputs {
            if let Some(link) = &input.link {
                link_assignments.push(serde_json::json!({
                    "source_machine": link.id,
                    "source_var": link.output,
                    "target_machine": &machine.id,
                    "target_var": safe_ident(input_name),
                }));
            }
        }
    }

    serde_json::json!({
        "machine_fields": machine_fields,
        "machine_ticks": machine_ticks,
        "link_assignments": link_assignments,
    })
}

// ── Type mapping: Type → Rust type string ───────────────────────────────────

pub fn type_to_rust(t: &crate::machine::Type) -> &str {
    match t {
        crate::machine::Type::Bool => "bool",
        crate::machine::Type::U8 => "u8",
        crate::machine::Type::U16 => "u16",
        crate::machine::Type::U32 => "u32",
        crate::machine::Type::U64 => "u64",
        crate::machine::Type::I8 => "i8",
        crate::machine::Type::I16 => "i16",
        crate::machine::Type::I32 => "i32",
        crate::machine::Type::I64 => "i64",
        crate::machine::Type::F32 => "f32",
        crate::machine::Type::F64 => "f64",
        crate::machine::Type::String => "String",
    }
}

/// Make a Rust-safe identifier (use r# prefix for keywords).
fn safe_ident(name: &str) -> String {
    let keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern",
        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
        "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
        "struct", "super", "trait", "type", "unsafe", "use", "where", "while",
        "async", "await", "dyn", "abstract", "become", "box", "do", "final",
        "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
    ];
    if keywords.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

// ── Expression to Rust code string ──────────────────────────────────────────

pub fn expr_to_rust(
    expr: &Expression,
    persistent_fields: &[String],
) -> Result<String, CompileError> {
    match expr {
        Expression::Value(v) => Ok(value_to_rust(v)),
        Expression::Reference(r) => {
            let ident = safe_ident(&r.target);
            Ok(format!("state.{ident}"))
        }
        Expression::Parenthesis(inner) => {
            let inner_code = expr_to_rust(inner, persistent_fields)?;
            Ok(format!("({inner_code})"))
        }
        Expression::Unary(op, inner) => {
            let inner_code = expr_to_rust(inner, persistent_fields)?;
            let rust_op = match op {
                UnaryOperator::Negate => "-",
                UnaryOperator::Not => "!",
                UnaryOperator::BitNot => "~",
            };
            Ok(format!("{rust_op}{inner_code}"))
        }
        Expression::Binary(left, op, right) => {
            let left_code = expr_to_rust(left, persistent_fields)?;
            let right_code = expr_to_rust(right, persistent_fields)?;
            let rust_op = match op {
                BinaryOperator::Add => "+",
                BinaryOperator::Sub => "-",
                BinaryOperator::Mul => "*",
                BinaryOperator::Div => "/",
                BinaryOperator::Mod => "%",
                BinaryOperator::And => " & ",
                BinaryOperator::Or => " | ",
                BinaryOperator::Xor => " ^ ",
                BinaryOperator::BitAnd => " & ",
                BinaryOperator::BitOr => " | ",
                BinaryOperator::BitXor => " ^ ",
                BinaryOperator::LogicalOr => " || ",
                BinaryOperator::LogicalAnd => " && ",
                BinaryOperator::LogicalXor => " ^^ ",
                BinaryOperator::Equal => "==",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::LessThan => "<",
                BinaryOperator::LessEqual => "<=",
                BinaryOperator::GreaterThan => ">",
                BinaryOperator::GreaterEqual => ">=",
            };
            Ok(format!("{left_code} {rust_op} {right_code}"))
        }
    }
}

fn value_to_rust(v: &Value) -> String {
    match v {
        Value::Integer(iv) => format!("{}i64", iv.value),
        Value::Float(fv) => {
            // Format as a Rust float literal
            let s = format!("{}", fv.value);
            if s.contains('e') || s.contains('E') {
                s // already scientific
            } else {
                format!("{s}.0")
            }
        }
        Value::String(sv) => {
            // Escape special characters for Rust string literal
            let escaped = sv
                .value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\t', "\\t")
                .replace('\r', "\\r");
            format!("\"{escaped}\"")
        }
    }
}

// ── Statement to Rust code string ───────────────────────────────────────────

pub fn stmt_to_rust(
    stmt: &FullStatement,
    persistent_fields: &[String],
) -> Result<String, CompileError> {
    let target = safe_ident(&stmt.statement.target);
    let expr_code = expr_to_rust(
        &stmt.statement.expression,
        persistent_fields,
    )?;

    let op = &stmt.statement.operator;
    let rust_stmt = match op {
        crate::machine::AssignmentOperator::Assign => {
            format!("update.{target} = Some({expr_code});")
        }
        crate::machine::AssignmentOperator::AddAssign => {
            format!("update.{target} = Some({target} + {expr_code});")
        }
        crate::machine::AssignmentOperator::SubAssign => {
            format!("update.{target} = Some({target} - {expr_code});")
        }
        crate::machine::AssignmentOperator::MulAssign => {
            format!("update.{target} = Some({target} * {expr_code});")
        }
        crate::machine::AssignmentOperator::DivAssign => {
            format!("update.{target} = Some({target} / {expr_code});")
        }
        crate::machine::AssignmentOperator::ModAssign => {
            format!("update.{target} = Some({target} % {expr_code});")
        }
        crate::machine::AssignmentOperator::AndAssign => {
            format!("update.{target} = Some({target} & {expr_code});")
        }
        crate::machine::AssignmentOperator::OrAssign => {
            format!("update.{target} = Some({target} | {expr_code});")
        }
        crate::machine::AssignmentOperator::XorAssign => {
            format!("update.{target} = Some({target} ^ {expr_code});")
        }
        crate::machine::AssignmentOperator::LogicalAndAssign => {
            format!("update.{target} = Some({target} && {expr_code});")
        }
        crate::machine::AssignmentOperator::LogicalOrAssign => {
            format!("update.{target} = Some({target} || {expr_code});")
        }
        crate::machine::AssignmentOperator::LogicalXorAssign => {
            format!("update.{target} = Some({target} ^^ {expr_code});")
        }
    };
    Ok(rust_stmt)
}

// ── Default value for type ──────────────────────────────────────────────────

pub fn default_value_for_type(t: &crate::machine::Type) -> String {
    match t {
        crate::machine::Type::Bool => "false".to_string(),
        crate::machine::Type::U8
        | crate::machine::Type::U16
        | crate::machine::Type::U32
        | crate::machine::Type::U64
        | crate::machine::Type::I8
        | crate::machine::Type::I16
        | crate::machine::Type::I32
        | crate::machine::Type::I64 => "0".to_string(),
        crate::machine::Type::F32 | crate::machine::Type::F64 => "0.0".to_string(),
        crate::machine::Type::String => "String::new()".to_string(),
    }
}

// ── Value to Rust literal ───────────────────────────────────────────────────

pub fn value_to_literal(v: &Value) -> String {
    match v {
        Value::Integer(iv) => format!("{}i64", iv.value),
        Value::Float(fv) => {
            let s = format!("{}", fv.value);
            if s.contains('e') || s.contains('E') {
                s
            } else {
                format!("{s}.0")
            }
        }
        Value::String(sv) => {
            let escaped = sv
                .value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\t', "\\t")
                .replace('\r', "\\r");
            format!("\"{escaped}\"")
        }
    }
}

/// Value to Rust literal with explicit type suffix.
pub fn value_to_literal_typed(v: &Value, target_type: &crate::machine::Type) -> String {
    match v {
        Value::Integer(iv) => format!("{}{}", iv.value, int_suffix_for_type(target_type)),
        Value::Float(fv) => {
            let s = format!("{}", fv.value);
            let suffix = match target_type {
                crate::machine::Type::F32 => "f32",
                crate::machine::Type::F64 => "f64",
                _ => "f64",
            };
            if s.contains('e') || s.contains('E') {
                format!("{s} as {suffix}")
            } else {
                format!("{s}.0 as {suffix}")
            }
        }
        Value::String(sv) => {
            let escaped = sv
                .value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\t', "\\t")
                .replace('\r', "\\r");
            format!("\"{escaped}\"")
        }
    }
}

fn int_suffix_for_type(t: &crate::machine::Type) -> &'static str {
    match t {
        crate::machine::Type::U8 => "u8",
        crate::machine::Type::U16 => "u16",
        crate::machine::Type::U32 => "u32",
        crate::machine::Type::U64 => "u64",
        crate::machine::Type::I8 => "i8",
        crate::machine::Type::I16 => "i16",
        crate::machine::Type::I32 => "i32",
        crate::machine::Type::I64 => "i64",
        _ => "i64", // default fallback
    }
}

// ── Condition code for when clauses ──────────────────────────────────────────

/// Convert a FullExpression to Rust code suitable as a condition (truthy check).
/// Returns the raw expression code — truthiness is implicit in Rust.
pub fn condition_to_rust(
    expr: &FullExpression,
    persistent_fields: &[String],
) -> Result<String, CompileError> {
    expr_to_rust(&expr.expression, persistent_fields)
}
