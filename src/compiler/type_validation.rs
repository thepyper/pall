/// Type validation for the Pall compiler.
///
/// This module validates that:
/// - Assignment target type can hold the expression type (lossless cast)
/// - When conditions are truthy (Bool or numeric)
/// - Operators are used with compatible types
/// - No lossy casts occur

use std::collections::HashMap;

use crate::machine::{
    Expression, FullExpression, FullStatement, StateMachine, Type, UnaryOperator, Value,
};

use super::error::CompileError;
use super::typecheck_rules::*;
use super::typecheck::{TypeEnv, VariableScope};

// ── Type Validation ──────────────────────────────────────────────────────────

/// Validate types for all machines given their TypeEnvs and VariableScopes.
///
/// Returns a list of CompileErrors for any type issues found.
pub fn validate_types(
    machines: &[StateMachine],
    type_envs: &[(TypeEnv, Vec<CompileError>)],
) -> Vec<CompileError> {
    let mut errors = Vec::new();

    for (machine, (type_env, _inference_errors)) in machines.iter().zip(type_envs.iter()) {
        let scope = VariableScope::from_machine(machine);

        // 1. Validate signal expressions
        for (_name, signal) in &machine.signals {
            validate_signal_expression(signal, type_env, &scope, &mut errors);
        }

        // 2. Validate timer when expressions (Timer.when is Expression, not FullExpression)
        for (name, timer) in &machine.timers {
            if let Some(ref when_expr) = timer.when {
                validate_when_expression_expr(when_expr, type_env, &scope, name, "timer", &mut errors);
            }
        }

        // 3. Validate transitions
        for (state_name, state) in &machine.states {
            for (_i, transition) in state.transitions.iter().enumerate() {
                if let Some(ref when_expr) = transition.when {
                    validate_when_expression(
                        when_expr,
                        type_env,
                        &scope,
                        state_name,
                        "transition",
                        &mut errors,
                    );
                }
                for stmt in &transition.r#do {
                    validate_assignment(
                        stmt,
                        type_env,
                        &scope,
                        &machine.variables,
                        &machine.signals,
                        &machine.timers,
                        &machine.inputs,
                        &machine.constants,
                        &machine.id,
                        state_name,
                        &mut errors,
                    );
                }
            }
        }

        // 4. Validate actions
        for (state_name, state) in &machine.states {
            for (_i, action) in state.actions.iter().enumerate() {
                if let Some(ref when_expr) = action.when {
                    validate_when_expression(
                        when_expr,
                        type_env,
                        &scope,
                        state_name,
                        "action",
                        &mut errors,
                    );
                }
                for stmt in &action.r#do {
                    validate_assignment(
                        stmt,
                        type_env,
                        &scope,
                        &machine.variables,
                        &machine.signals,
                        &machine.timers,
                        &machine.inputs,
                        &machine.constants,
                        &machine.id,
                        state_name,
                        &mut errors,
                    );
                }
            }
        }
    }

    errors
}

/// Validate a signal expression.
fn validate_signal_expression(
    signal: &crate::machine::Signal,
    type_env: &TypeEnv,
    scope: &VariableScope,
    errors: &mut Vec<CompileError>,
) {
    let signal_type = &signal.r#type;
    let expr_type = get_expression_type(&signal.expr, type_env, scope);

    if let Some(ref expr_type) = expr_type {
        if !is_cast_lossless(expr_type, signal_type) {
            errors.push(CompileError::new(
                super::error::CompileErrorKind::InvalidSignalExpr,
                format!(
                    "signal '{:?}': expression type {:?} cannot be assigned to signal type {:?}",
                    signal_type, expr_type, signal_type
                ),
            ));
        }
    }
}

/// Validate a when expression (must be truthy). Takes a FullExpression.
fn validate_when_expression(
    expr: &FullExpression,
    type_env: &TypeEnv,
    scope: &VariableScope,
    context_name: &str,
    context_type: &str,
    errors: &mut Vec<CompileError>,
) {
    validate_when_expression_expr(&expr.expression, type_env, scope, context_name, context_type, errors);
}

/// Validate a when expression (must be truthy). Takes a plain Expression (for Timer).
fn validate_when_expression_expr(
    expr: &Expression,
    type_env: &TypeEnv,
    scope: &VariableScope,
    context_name: &str,
    context_type: &str,
    errors: &mut Vec<CompileError>,
) {
    let expr_type = get_expression_type(expr, type_env, scope);

    if let Some(ref ty) = expr_type {
        if !is_truthy_type(ty) {
            errors.push(CompileError::new(
                super::error::CompileErrorKind::InvalidSignalExpr,
                format!(
                    "{context_type} '{}': when condition must be truthy, got {:?}",
                    context_name, ty
                ),
            ));
        }
    }
}

/// Validate an assignment statement.
fn validate_assignment(
    stmt: &FullStatement,
    type_env: &TypeEnv,
    scope: &VariableScope,
    variables: &HashMap<String, crate::machine::Variable>,
    signals: &HashMap<String, crate::machine::Signal>,
    timers: &HashMap<String, crate::machine::Timer>,
    inputs: &HashMap<String, crate::machine::Input>,
    constants: &HashMap<String, crate::machine::Constant>,
    machine_id: &str,
    state_name: &str,
    errors: &mut Vec<CompileError>,
) {
    let target = &stmt.statement.target;
    let target_type = get_target_type(
        target,
        variables,
        signals,
        timers,
        inputs,
        constants,
    );

    if let Some(target_type) = target_type {
        let expr_type = get_expression_type(&stmt.statement.expression, type_env, scope);

        if let Some(ref expr_ty) = expr_type {
            if !is_cast_lossless(expr_ty, &target_type) {
                errors.push(CompileError::new(
                    super::error::CompileErrorKind::InvalidSignalExpr,
                    format!(
                        "assignment in machine '{}', state '{}': cannot assign {:?} to {:?} (lossy cast)",
                        machine_id, state_name, expr_ty, target_type
                    ),
                ));
            }
        }
    }
}

/// Get the type of a target variable/signal/timer/input.
fn get_target_type(
    target: &str,
    variables: &HashMap<String, crate::machine::Variable>,
    signals: &HashMap<String, crate::machine::Signal>,
    timers: &HashMap<String, crate::machine::Timer>,
    inputs: &HashMap<String, crate::machine::Input>,
    constants: &HashMap<String, crate::machine::Constant>,
) -> Option<Type> {
    if let Some(var) = variables.get(target) {
        return Some(var.r#type.clone());
    }
    if let Some(sig) = signals.get(target) {
        return Some(sig.r#type.clone());
    }
    if let Some(timer) = timers.get(target) {
        return Some(timer.r#type.clone());
    }
    if let Some(input) = inputs.get(target) {
        return Some(input.r#type.clone());
    }
    if let Some(constant) = constants.get(target) {
        return Some(constant.r#type.clone());
    }
    None
}

/// Get the type of an expression from the TypeEnv and VariableScope.
fn get_expression_type(
    expr: &Expression,
    type_env: &TypeEnv,
    scope: &VariableScope,
) -> Option<Type> {
    find_type_in_env(expr, type_env, scope)
}

/// Recursively find the type of an expression in the TypeEnv.
fn find_type_in_env(
    expr: &Expression,
    type_env: &TypeEnv,
    scope: &VariableScope,
) -> Option<Type> {
    match expr {
        Expression::Value(val) => match val {
            Value::Integer(_) => Some(Type::I64),
            Value::Float(_) => Some(Type::F64),
            Value::String(_) => Some(Type::String),
            Value::Bool(_) => Some(Type::Bool),
        },
        Expression::Reference(ref_) => {
            // Look up in scope
            scope.get(&ref_.target).cloned()
        }
        Expression::Parenthesis(inner) => find_type_in_env(inner, type_env, scope),
        Expression::Unary(op, inner) => {
            let inner_ty = find_type_in_env(inner, type_env, scope)?;
            match op {
                UnaryOperator::Negate => Some(inner_ty),
                UnaryOperator::Not => Some(Type::Bool),
                UnaryOperator::BitNot => Some(inner_ty),
            }
        }
        Expression::Binary(left, op, right) => {
            let left_ty = find_type_in_env(left, type_env, scope)?;
            let right_ty = find_type_in_env(right, type_env, scope)?;
            check_operator_compatibility(&left_ty, &right_ty, op)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{
        FullStatement, IntegerFmt, IntegerValue, State, StateMachine,
        Statement as MachineStatement, Type, Variable,
    };
    use std::collections::HashMap;

    fn make_test_machine() -> StateMachine {
        let mut states = HashMap::new();
        states.insert(
            "initial".to_string(),
            State {
                actions: vec![],
                transitions: vec![],
            },
        );
        StateMachine {
            id: "test".to_string(),
            initial: Some("initial".to_string()),
            states,
            inputs: HashMap::new(),
            signals: HashMap::new(),
            timers: HashMap::new(),
            variables: [
                (
                    "counter".to_string(),
                    Variable {
                        r#type: Type::U16,
                        initial: None,
                        output: false,
                    },
                ),
                (
                    "x".to_string(),
                    Variable {
                        r#type: Type::U8,
                        initial: None,
                        output: false,
                    },
                ),
                (
                    "y".to_string(),
                    Variable {
                        r#type: Type::U32,
                        initial: None,
                        output: false,
                    },
                ),
                (
                    "flag".to_string(),
                    Variable {
                        r#type: Type::Bool,
                        initial: None,
                        output: false,
                    },
                ),
            ]
            .into_iter()
            .collect(),
            constants: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_assignment_passes() {
        let machine = make_test_machine();
        let type_envs = super::super::typecheck::infer_all(&[machine.clone()]);
        let type_env = &type_envs[0].0;

        let stmt = FullStatement {
            statement: MachineStatement {
                operator: crate::machine::AssignmentOperator::Assign,
                target: "counter".to_string(),
                expression: Expression::Reference(crate::machine::Reference {
                    target: "counter".to_string(),
                }),
            },
            raw: "counter = counter".to_string(),
        };

        let scope = VariableScope::from_machine(&machine);
        let mut errors = Vec::new();
        validate_assignment(
            &stmt,
            type_env,
            &scope,
            &machine.variables,
            &machine.signals,
            &machine.timers,
            &machine.inputs,
            &machine.constants,
            "test",
            "initial",
            &mut errors,
        );
        assert!(errors.is_empty(), "Valid assignment should not produce errors");
    }

    #[test]
    fn test_lossy_assignment_fails() {
        let machine = make_test_machine();
        let type_envs = super::super::typecheck::infer_all(&[machine.clone()]);
        let type_env = &type_envs[0].0;

        // x: U8 = y (U32) → invalid (U32 cannot fit in U8)
        let stmt = FullStatement {
            statement: MachineStatement {
                operator: crate::machine::AssignmentOperator::Assign,
                target: "x".to_string(),
                expression: Expression::Reference(crate::machine::Reference {
                    target: "y".to_string(),
                }),
            },
            raw: "x = y".to_string(),
        };

        let scope = VariableScope::from_machine(&machine);
        let mut errors = Vec::new();
        validate_assignment(
            &stmt,
            type_env,
            &scope,
            &machine.variables,
            &machine.signals,
            &machine.timers,
            &machine.inputs,
            &machine.constants,
            "test",
            "initial",
            &mut errors,
        );
        assert!(!errors.is_empty(), "Lossy assignment should produce errors");
        assert!(errors[0].message.contains("lossy cast"));
    }

    #[test]
    fn test_valid_widening_assignment_passes() {
        let machine = make_test_machine();
        let type_envs = super::super::typecheck::infer_all(&[machine.clone()]);
        let type_env = &type_envs[0].0;

        // y: U32 = x (U8) → valid (U8 fits in U32)
        let stmt = FullStatement {
            statement: MachineStatement {
                operator: crate::machine::AssignmentOperator::Assign,
                target: "y".to_string(),
                expression: Expression::Reference(crate::machine::Reference {
                    target: "x".to_string(),
                }),
            },
            raw: "y = x".to_string(),
        };

        let scope = VariableScope::from_machine(&machine);
        let mut errors = Vec::new();
        validate_assignment(
            &stmt,
            type_env,
            &scope,
            &machine.variables,
            &machine.signals,
            &machine.timers,
            &machine.inputs,
            &machine.constants,
            "test",
            "initial",
            &mut errors,
        );
        assert!(errors.is_empty(), "Valid widening assignment should not produce errors");
    }
}
