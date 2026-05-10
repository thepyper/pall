use std::collections::HashMap;

use crate::machine::{
    Action, BinaryOperator, Constant, Expression, FullExpression, FullStatement,
    Input, Signal, Statement, Timer, UnaryOperator, Value,
};

use super::super::super::error::{CompileError, CompileErrorKind};

/// Context passed to code generation helpers for variable naming.
pub struct CodegenContext {
    /// Variable name for the Persistent struct parameter (e.g. "x").
    pub state_var: String,
    /// Variable name for the Persistent struct local (e.g. "y").
    pub update_var: String,
}

impl CodegenContext {
    pub fn new(state_var: &str, update_var: &str) -> Self {
        Self {
            state_var: state_var.to_string(),
            update_var: update_var.to_string(),
        }
    }
}

/// Precalculated field access map for expressions.
/// Maps field names to their Rust access strings.
pub struct FieldAccessMap {
    /// All field access strings: "counter" -> "y.counter", "start" -> "x.start"
    pub accesses: std::collections::HashMap<String, String>,
}

impl FieldAccessMap {
    pub fn new(
        state_var: &str,
        update_var: &str,
        inputs: &std::collections::HashMap<String, crate::machine::Input>,
        variables: &std::collections::HashMap<String, crate::machine::Variable>,
        signals: &std::collections::HashMap<String, crate::machine::Signal>,
        timers: &std::collections::HashMap<String, crate::machine::Timer>,
        constants: &std::collections::HashMap<String, crate::machine::Constant>,
    ) -> Self {
        let mut accesses = std::collections::HashMap::new();

        // State: y.state.as_str() (special: always treated as string)
        accesses.insert("state".to_string(), format!("{}.state.as_str()", update_var));

        // Inputs: always x.field (read-only, not in y)
        for (name, _) in inputs {
            accesses.insert(name.clone(), format!("{}.{}", state_var, safe_ident(name)));
        }

        // Variables, signals, timers, constants: y.field
        for (name, _) in variables {
            accesses.insert(name.clone(), format!("{}.{}", update_var, safe_ident(name)));
        }
        for (name, _) in signals {
            accesses.insert(name.clone(), format!("{}.{}", update_var, safe_ident(name)));
        }
        for (name, _) in timers {
            accesses.insert(name.clone(), format!("{}.{}", update_var, safe_ident(name)));
        }
        for (name, _) in constants {
            accesses.insert(name.clone(), format!("{}.{}", update_var, safe_ident(name)));
        }

        Self { accesses }
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.accesses.get(name)
    }
}

/// Build a JSON data context for a single machine's types template.
pub fn build_types_data(
    machine: &crate::machine::StateMachine,
    context: &CodegenContext,
) -> serde_json::Value {
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
            value_to_literal_typed(init, &var.r#type)
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

    let mut states_json = vec![];
    for (state_name, _) in &machine.states {
        let variant_name = to_pascal_case(state_name);
        states_json.push(serde_json::json!({
            "name": variant_name,
            "raw_name": state_name,
        }));
    }

    serde_json::json!({
        "machine_id": machine.id,
        "initial": machine.initial.clone().unwrap_or_else(|| "initial".to_string()),
        "initial_variant": to_pascal_case(&machine.initial.clone().unwrap_or_else(|| "initial".to_string())),
        "states": states_json,
        "inputs": inputs,
        "variables": variables,
        "signals": signals,
        "timers": timers,
        "constants": constants,
        "state_var": context.state_var.clone(),
        "update_var": context.update_var.clone(),
    })
}

/// Build a JSON data context for a single machine's tick template.
pub fn build_tick_data(
    machine: &crate::machine::StateMachine,
    context: &CodegenContext,
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

    // Build variables list for init() in tick template
    let mut variables_json = vec![];
    for (name, var) in &machine.variables {
        let ident = safe_ident(name);
        let default_val = if let Some(ref init) = var.initial {
            value_to_literal_typed(init, &var.r#type)
        } else {
            default_value_for_type(&var.r#type)
        };
        variables_json.push(serde_json::json!({
            "name": ident,
            "rust_type": type_to_rust(&var.r#type),
            "default_value": default_val,
        }));
    }

    // Build inputs list for init() in tick template
    let mut inputs_json = vec![];
    for (name, input) in &machine.inputs {
        inputs_json.push(serde_json::json!({
            "name": safe_ident(name),
            "rust_type": type_to_rust(&input.r#type),
        }));
    }

    // Build constants list for init() in tick template
    let mut constants_json = vec![];
    for (name, constant) in &machine.constants {
        constants_json.push(serde_json::json!({
            "name": safe_ident(name),
            "rust_type": type_to_rust(&constant.r#type),
            "value": format!("{}", safe_ident(name)),
        }));
    }

    let context = CodegenContext::new(&context.state_var, &context.update_var);

    // Build the precalculated field access map
    let field_accesses = FieldAccessMap::new(
        &context.state_var,
        &context.update_var,
        &machine.inputs,
        &machine.variables,
        &machine.signals,
        &machine.timers,
        &machine.constants,
    );

    // Build the match body as a single Rust code string
    let mut match_body = String::new();

    for (state_name, state) in &machine.states {
        let variant = to_pascal_case(state_name);
        match_body.push_str(&format!("        State::{} => {{\n", variant));

        // Execute Actions
        for action in &state.actions {
            let when_code = match &action.when {
                Some(expr) => match condition_to_rust(expr, &field_accesses) {
                    Ok(code) => code,
                    Err(e) => {
                        errors.push(CompileError::new(
                            CompileErrorKind::InvalidSignalExpr,
                            e,
                        ));
                        "false".to_string()
                    }
                },
                None => String::new(),
            };

            let mut stmts = vec![];
            for stmt in &action.r#do {
                match stmt_to_rust(stmt, &context, &field_accesses) {
                    Ok(code) => stmts.push(code),
                    Err(e) => {
                        errors.push(CompileError::new(
                            CompileErrorKind::InvalidSignalExpr,
                            e,
                        ));
                    }
                }
            }

            if !when_code.is_empty() {
                match_body.push_str(&format!("            if {} {{\n", when_code));
            }
            for stmt in &stmts {
                match_body.push_str(&format!("            {}\n", stmt));
            }
            if !when_code.is_empty() {
                match_body.push_str("            }\n");
            }
        }

        // Execute Transitions as if/else-if chain
        let num_transitions = state.transitions.len();
        let mut needs_closing = false;
        for (i, trans) in state.transitions.iter().enumerate() {
            let when_code = match &trans.when {
                Some(expr) => match condition_to_rust(expr, &field_accesses) {
                    Ok(code) => code,
                    Err(e) => {
                        errors.push(CompileError::new(
                            CompileErrorKind::InvalidSignalExpr,
                            e,
                        ));
                        "false".to_string()
                    }
                },
                None => String::new(),
            };

            let target_variant = to_pascal_case(&trans.target);
            let update_var = &context.update_var;
            let target_code = format!("{}.state = State::{};", update_var, target_variant);

            let mut stmts = vec![];
            for stmt in &trans.r#do {
                match stmt_to_rust(stmt, &context, &field_accesses) {
                    Ok(code) => stmts.push(code),
                    Err(e) => {
                        errors.push(CompileError::new(
                            CompileErrorKind::InvalidSignalExpr,
                            e,
                        ));
                    }
                }
            }

            let is_last = i == num_transitions - 1;
            let is_always_true = when_code.is_empty();

            if is_always_true {
                // Always-true: close previous if-branch, use else
                if needs_closing {
                    // Previous was an if/else-if branch, close it and open else
                    match_body.push_str("            } else {\n");
                }
                // First always-true transition: no if/else needed, just execute
                for stmt in &stmts {
                    match_body.push_str(&format!("            {}\n", stmt));
                }
                match_body.push_str(&format!("            {}\n", target_code));
                // Only close if we opened an else above
                if needs_closing || i > 0 {
                    match_body.push_str("            }\n");
                }
                needs_closing = false;
            } else {
                // Conditional: open if or else-if
                if i > 0 && needs_closing {
                    // Close previous, open else-if (combined)
                    match_body.push_str(&format!("            }} else if {} {{\n", when_code));
                } else if i == 0 {
                    // First transition with condition
                    match_body.push_str(&format!("            if {} {{\n", when_code));
                } else {
                    // Previous was always-true, current is conditional
                    match_body.push_str(&format!("            }} else if {} {{\n", when_code));
                }
                for stmt in &stmts {
                    match_body.push_str(&format!("            {}\n", stmt));
                }
                match_body.push_str(&format!("            {}\n", target_code));
                if !is_last {
                    // Don't close yet, next branch will close it
                    needs_closing = true;
                } else {
                    // Last conditional, close it
                    match_body.push_str("            }\n");
                    needs_closing = false;
                }
            }
        }

        match_body.push_str("\n        }\n\n");
    }

    let mut signals_json = vec![];
    for (name, sig) in &machine.signals {
        let expr_code = match expr_to_rust(&sig.expr, &field_accesses) {
            Ok(code) => code,
            Err(e) => {
                errors.push(CompileError::new(
                    CompileErrorKind::InvalidSignalExpr,
                    e,
                ));
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
            Some(expr) => match expr_to_rust(expr, &field_accesses) {
                Ok(code) => code,
                Err(e) => {
                    errors.push(CompileError::new(
                        CompileErrorKind::InvalidSignalExpr,
                        e,
                    ));
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

    if !errors.is_empty() {
        return Err(errors);
    }

    let state_variants: Vec<serde_json::Value> = machine
        .states
        .keys()
        .map(|name| serde_json::json!({
            "variant": to_pascal_case(name),
            "raw_name": name,
        }))
        .collect();

    Ok(serde_json::json!({
        "machine_id": machine.id,
        "initial": initial,
        "initial_variant": to_pascal_case(&initial),
        "match_body": match_body,
        "signals": signals_json,
        "timers": timers_json,
        "variables": variables_json,
        "inputs": inputs_json,
        "constants": constants_json,
        "state_var": context.state_var.clone(),
        "update_var": context.update_var.clone(),
        "types_module": format!("{}_types", machine.id),
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
pub fn build_group_data(
    machines: &[crate::machine::StateMachine],
    group_ctx: &CodegenContext,
    tick_ctx: &CodegenContext,
) -> serde_json::Value {
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
        "state_group_var": group_ctx.state_var.clone(),
        "update_group_var": group_ctx.update_var.clone(),
        "tick_state_var": tick_ctx.state_var.clone(),
    })
}

// ── PascalCase conversion ─────────────────────────────────────────────────

fn to_pascal_case(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let mut result = String::with_capacity(input.len());
    let mut capitalize_next = true;
    for ch in input.chars() {
        if ch == '_' {
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            for uc in ch.to_uppercase() {
                result.push(uc);
            }
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
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
    field_accesses: &FieldAccessMap,
) -> Result<String, String> {
    match expr {
        Expression::Value(v) => Ok(value_to_rust(v)),
        Expression::Reference(r) => {
            // Look up precalculated access string
            let access = field_accesses.get(&r.target).ok_or_else(|| {
                format!("unknown field reference: {}", r.target)
            })?;
            Ok(access.clone())
        }
        Expression::Parenthesis(inner) => {
            let inner_code = expr_to_rust(inner, field_accesses)?;
            Ok(format!("({inner_code})"))
        }
        Expression::Unary(op, inner) => {
            let inner_code = expr_to_rust(inner, field_accesses)?;
            let rust_op = match op {
                UnaryOperator::Negate => "-",
                UnaryOperator::Not => "!",
                UnaryOperator::BitNot => "!",
            };
            Ok(format!("{rust_op}{inner_code}"))
        }
        Expression::Binary(left, op, right) => {
            let left_code = expr_to_rust(left, field_accesses)?;
            let right_code = expr_to_rust(right, field_accesses)?;
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
                BinaryOperator::LogicalXor => " ^ ",
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
        Value::Bool(b) => format!("{}", b),
    }
}

// ── Statement to Rust code string ───────────────────────────────────────────

pub fn stmt_to_rust(
    stmt: &FullStatement,
    ctx: &CodegenContext,
    field_accesses: &FieldAccessMap,
) -> Result<String, String> {
    let target = safe_ident(&stmt.statement.target);
    // Target is always y.field (inputs can't be targets)
    let target_access = format!("{}.{}", ctx.update_var, target);
    let expr_code = expr_to_rust(
        &stmt.statement.expression,
        field_accesses,
    ).map_err(|e| format!("statement error: {}", e))?;

    let op = &stmt.statement.operator;
    // Get the field access string for the target (always y.field for statement targets)
    let field_access = field_accesses.get(&stmt.statement.target)
        .cloned()
        .unwrap_or_else(|| format!("{}.{}", ctx.update_var, target));

    let rust_stmt = match op {
        crate::machine::AssignmentOperator::Assign => {
            format!("{target_access} = {expr_code};")
        }
        crate::machine::AssignmentOperator::AddAssign => {
            format!("{target_access} = {field_access} + {expr_code};")
        }
        crate::machine::AssignmentOperator::SubAssign => {
            format!("{target_access} = {field_access} - {expr_code};")
        }
        crate::machine::AssignmentOperator::MulAssign => {
            format!("{target_access} = {field_access} * {expr_code};")
        }
        crate::machine::AssignmentOperator::DivAssign => {
            format!("{target_access} = {field_access} / {expr_code};")
        }
        crate::machine::AssignmentOperator::ModAssign => {
            format!("{target_access} = {field_access} % {expr_code};")
        }
        crate::machine::AssignmentOperator::AndAssign => {
            format!("{target_access} = {field_access} & {expr_code};")
        }
        crate::machine::AssignmentOperator::OrAssign => {
            format!("{target_access} = {field_access} | {expr_code};")
        }
        crate::machine::AssignmentOperator::XorAssign => {
            format!("{target_access} = {field_access} ^ {expr_code};")
        }
        crate::machine::AssignmentOperator::LogicalAndAssign => {
            format!("{target_access} = {field_access} && {expr_code};")
        }
        crate::machine::AssignmentOperator::LogicalOrAssign => {
            format!("{target_access} = {field_access} || {expr_code};")
        }
        crate::machine::AssignmentOperator::LogicalXorAssign => {
            format!("{target_access} = {field_access} ^ {expr_code};")
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
        Value::Bool(b) => format!("{}", b),
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
        Value::Bool(b) => format!("{}", b),
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
    field_accesses: &FieldAccessMap,
) -> Result<String, String> {
    expr_to_rust(&expr.expression, field_accesses)
}
