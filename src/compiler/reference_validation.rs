/// Reference validation for the Pall compiler.
///
/// This module validates that:
/// - Every variable reference in expressions resolves to a defined machine field
/// - Every assignment target is a variable (not signal, timer, input, or constant)
///
/// Reserved internal names (like "state") are exempt from the unknown reference check.

use std::collections::HashMap;

use crate::machine::{
    Expression, FullExpression, FullStatement, StateMachine, Variable,
};

use super::error::{CompileError, CompileErrorKind};
use super::typecheck::VariableScope;

// ── Reserved names ────────────────────────────────────────────────────────────

/// Check if a name is reserved for internal compiler use.
/// Reserved names are exempt from unknown-reference checks.
/// Add new reserved names here as the compiler evolves.
pub fn is_reserved_internal_name(name: &str) -> bool {
    matches!(name, "state")
}

// ── Reference existence check ─────────────────────────────────────────────────

/// Check if a reference name exists in the scope (or is a reserved internal name).
fn is_known_reference(name: &str, scope: &VariableScope) -> bool {
    is_reserved_internal_name(name) || scope.get(name).is_some()
}

// ── Assignment target validation ──────────────────────────────────────────────

/// Check if a target name is a valid variable (not signal, timer, input, or constant).
fn is_valid_assignment_target(target: &str, variables: &HashMap<String, Variable>) -> bool {
    variables.contains_key(target)
}

// ── Context types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationContext {
    Signal,
    TimerWhen,
    TransitionWhen,
    TransitionDo,
    ActionWhen,
    ActionDo,
}

impl ValidationContext {
    fn display_name(&self) -> &'static str {
        match self {
            ValidationContext::Signal => "signal",
            ValidationContext::TimerWhen => "timer when",
            ValidationContext::TransitionWhen => "transition when",
            ValidationContext::TransitionDo => "transition do",
            ValidationContext::ActionWhen => "action when",
            ValidationContext::ActionDo => "action do",
        }
    }

    fn kind_name(&self) -> &'static str {
        match self {
            ValidationContext::Signal => "signal",
            ValidationContext::TimerWhen => "timer",
            ValidationContext::TransitionWhen => "transition",
            ValidationContext::TransitionDo => "transition",
            ValidationContext::ActionWhen => "action",
            ValidationContext::ActionDo => "action",
        }
    }
}

// ── Expression walking ────────────────────────────────────────────────────────

/// Walk an expression tree and check all references exist in the scope.
fn check_expression_refs(
    expr: &Expression,
    scope: &VariableScope,
    machine_id: &str,
    context_name: &str,
    context: ValidationContext,
    errors: &mut Vec<CompileError>,
) {
    match expr {
        Expression::Reference(r) => {
            if !is_known_reference(&r.target, scope) {
                errors.push(CompileError::new(
                    CompileErrorKind::UnknownVariableReference,
                    format!(
                        "unknown variable reference '{}' in machine '{}', {}, {}: '{}' is not defined",
                        r.target, machine_id, context.display_name(), context_name, r.target
                    ),
                ));
            }
        }
        Expression::Parenthesis(inner) => {
            check_expression_refs(inner, scope, machine_id, context_name, context, errors);
        }
        Expression::Unary(_, inner) => {
            check_expression_refs(inner, scope, machine_id, context_name, context, errors);
        }
        Expression::Binary(left, _, right) => {
            check_expression_refs(left, scope, machine_id, context_name, context, errors);
            check_expression_refs(right, scope, machine_id, context_name, context, errors);
        }
        Expression::Value(_) => {}
    }
}

/// Check a FullExpression's inner expression for unknown references.
fn check_full_expression_refs(
    full_expr: &FullExpression,
    scope: &VariableScope,
    machine_id: &str,
    context_name: &str,
    context: ValidationContext,
    errors: &mut Vec<CompileError>,
) {
    check_expression_refs(
        &full_expr.expression,
        scope,
        machine_id,
        context_name,
        context,
        errors,
    );
}

/// Check a plain Expression for unknown references (used for Signal.expr and Timer.when).
fn check_expression_refs_plain(
    expr: &Expression,
    scope: &VariableScope,
    machine_id: &str,
    context_name: &str,
    context: ValidationContext,
    errors: &mut Vec<CompileError>,
) {
    check_expression_refs(
        expr,
        scope,
        machine_id,
        context_name,
        context,
        errors,
    );
}

/// Validate that an assignment target is a valid variable.
fn validate_assignment_target(
    stmt: &FullStatement,
    variables: &HashMap<String, Variable>,
    machine_id: &str,
    state_name: &str,
    context: ValidationContext,
    errors: &mut Vec<CompileError>,
) {
    let target = &stmt.statement.target;
    if !is_valid_assignment_target(target, variables) {
        errors.push(CompileError::new(
            CompileErrorKind::InvalidAssignmentTarget,
            format!(
                "invalid assignment target '{}' in machine '{}', state '{}', {}: '{}' is not a variable",
                target, machine_id, state_name, context.display_name(), target
            ),
        ));
    }
}

// ── Main validation function ──────────────────────────────────────────────────

/// Validate all references and assignment targets in a machine.
///
/// Returns a list of CompileErrors for any issues found.
pub fn validate_references(machine: &StateMachine) -> Vec<CompileError> {
    let mut errors = Vec::new();
    let scope = VariableScope::from_machine(machine);

    // 1. Validate signal expressions (Signal.expr is Expression, not FullExpression)
    for (name, signal) in &machine.signals {
        check_expression_refs_plain(
            &signal.expr,
            &scope,
            &machine.id,
            name,
            ValidationContext::Signal,
            &mut errors,
        );
    }

    // 2. Validate timer when expressions
    for (name, timer) in &machine.timers {
        if let Some(ref when_expr) = timer.when {
            check_expression_refs(
                when_expr,
                &scope,
                &machine.id,
                name,
                ValidationContext::TimerWhen,
                &mut errors,
            );
        }
    }

    // 3. Validate transitions
    for (state_name, state) in &machine.states {
        for (idx, transition) in state.transitions.iter().enumerate() {
            let context_name = format!("transition {}", idx);
            let context_name_s = context_name.clone();

            if let Some(ref when_expr) = transition.when {
                check_full_expression_refs(
                    when_expr,
                    &scope,
                    &machine.id,
                    &context_name_s,
                    ValidationContext::TransitionWhen,
                    &mut errors,
                );
            }
            for stmt in &transition.r#do {
                validate_assignment_target(
                    stmt,
                    &machine.variables,
                    &machine.id,
                    state_name,
                    ValidationContext::TransitionDo,
                    &mut errors,
                );
                check_expression_refs(
                    &stmt.statement.expression,
                    &scope,
                    &machine.id,
                    &context_name_s,
                    ValidationContext::TransitionDo,
                    &mut errors,
                );
            }
        }
    }

    // 4. Validate actions
    for (state_name, state) in &machine.states {
        for (idx, action) in state.actions.iter().enumerate() {
            let context_name = format!("action {}", idx);
            let context_name_s = context_name.clone();

            if let Some(ref when_expr) = action.when {
                check_full_expression_refs(
                    when_expr,
                    &scope,
                    &machine.id,
                    &context_name_s,
                    ValidationContext::ActionWhen,
                    &mut errors,
                );
            }
            for stmt in &action.r#do {
                validate_assignment_target(
                    stmt,
                    &machine.variables,
                    &machine.id,
                    state_name,
                    ValidationContext::ActionDo,
                    &mut errors,
                );
                check_expression_refs(
                    &stmt.statement.expression,
                    &scope,
                    &machine.id,
                    &context_name_s,
                    ValidationContext::ActionDo,
                    &mut errors,
                );
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{
        FullStatement, IntegerFmt, State, StateMachine, Statement as MachineStatement, Transition,
        Type, Variable,
    };
    use std::collections::HashMap;

    fn make_machine_with_vars(
        var_defs: &[(String, Type)],
    ) -> StateMachine {
        let mut states = HashMap::new();
        states.insert(
            "initial".to_string(),
            State {
                actions: vec![],
                transitions: vec![],
            },
        );
        let variables: HashMap<String, Variable> = var_defs
            .iter()
            .map(|(name, ty)| {
                (
                    name.clone(),
                    Variable {
                        r#type: ty.clone(),
                        initial: None,
                        output: false,
                    },
                )
            })
            .collect();
        StateMachine {
            id: "test".to_string(),
            initial: Some("initial".to_string()),
            states,
            inputs: HashMap::new(),
            signals: HashMap::new(),
            timers: HashMap::new(),
            variables,
            constants: HashMap::new(),
        }
    }

    fn var_pair(name: &str, ty: Type) -> (String, Type) {
        (name.to_string(), ty)
    }

    #[test]
    fn test_is_reserved_internal_name() {
        assert!(is_reserved_internal_name("state"));
        assert!(!is_reserved_internal_name("counter"));
        assert!(!is_reserved_internal_name("input"));
    }

    #[test]
    fn test_is_valid_assignment_target() {
        let mut variables = HashMap::new();
        variables.insert(
            "counter".to_string(),
            Variable {
                r#type: Type::U32,
                initial: None,
                output: false,
            },
        );
        assert!(is_valid_assignment_target("counter", &variables));
        assert!(!is_valid_assignment_target("nonexistent", &variables));
    }

    #[test]
    fn test_valid_reference_passes() {
        let machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        let errors = validate_references(&machine);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_unknown_reference_in_signal_expression() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        let expr = Expression::Binary(
            Box::new(Expression::Reference(crate::machine::Reference {
                target: "counter".to_string(),
            })),
            crate::machine::BinaryOperator::Add,
            Box::new(Expression::Reference(crate::machine::Reference {
                target: "unknown_var".to_string(),
            })),
        );
        machine.signals.insert(
            "result".to_string(),
            crate::machine::Signal {
                r#type: Type::U32,
                output: false,
                expr,
            },
        );
        let errors = validate_references(&machine);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.kind == CompileErrorKind::UnknownVariableReference));
        assert!(errors.iter().any(|e| e.message.contains("unknown_var")));
    }

    #[test]
    fn test_unknown_reference_in_transition_when() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        let mut initial = machine.states.get_mut("initial").unwrap();
        initial.transitions.push(Transition {
            when: Some(FullExpression::parse("unknown_val > 0").unwrap()),
            r#do: vec![],
            target: "initial".to_string(),
        });
        let errors = validate_references(&machine);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.kind == CompileErrorKind::UnknownVariableReference));
    }

    #[test]
    fn test_unknown_reference_in_action_do() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        let mut initial = machine.states.get_mut("initial").unwrap();
        initial.actions.push(crate::machine::Action {
            when: None,
            r#do: vec![FullStatement {
                statement: MachineStatement {
                    operator: crate::machine::AssignmentOperator::Assign,
                    target: "counter".to_string(),
                    expression: Expression::Reference(crate::machine::Reference {
                        target: "missing".to_string(),
                    }),
                },
                raw: "counter = missing".to_string(),
            }],
        });
        let errors = validate_references(&machine);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.kind == CompileErrorKind::UnknownVariableReference));
        assert!(errors.iter().any(|e| e.message.contains("missing")));
    }

    #[test]
    fn test_signal_as_assignment_target() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        machine.signals.insert(
            "flag".to_string(),
            crate::machine::Signal {
                r#type: Type::Bool,
                output: false,
                expr: Expression::Value(crate::machine::Value::Bool(true)),
            },
        );
        let mut initial = machine.states.get_mut("initial").unwrap();
        initial.transitions.push(Transition {
            when: None,
            r#do: vec![FullStatement {
                statement: MachineStatement {
                    operator: crate::machine::AssignmentOperator::Assign,
                    target: "flag".to_string(),
                    expression: Expression::Value(crate::machine::Value::Bool(true)),
                },
                raw: "flag = true".to_string(),
            }],
            target: "initial".to_string(),
        });
        let errors = validate_references(&machine);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.kind == CompileErrorKind::InvalidAssignmentTarget));
    }

    #[test]
    fn test_input_as_assignment_target() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        machine.inputs.insert(
            "input1".to_string(),
            crate::machine::Input {
                r#type: Type::Bool,
                link: None,
                output: false,
            },
        );
        let mut initial = machine.states.get_mut("initial").unwrap();
        initial.transitions.push(Transition {
            when: None,
            r#do: vec![FullStatement {
                statement: MachineStatement {
                    operator: crate::machine::AssignmentOperator::Assign,
                    target: "input1".to_string(),
                    expression: Expression::Value(crate::machine::Value::Bool(true)),
                },
                raw: "input1 = true".to_string(),
            }],
            target: "initial".to_string(),
        });
        let errors = validate_references(&machine);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.kind == CompileErrorKind::InvalidAssignmentTarget));
    }

    #[test]
    fn test_constant_as_assignment_target() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        machine.constants.insert(
            "MAX".to_string(),
            crate::machine::Constant {
                r#type: Type::U32,
                output: false,
                value: crate::machine::Value::Integer(crate::machine::IntegerValue {
                    value: 100,
                    fmt: crate::machine::IntegerFmt::Dec,
                }),
            },
        );
        let mut initial = machine.states.get_mut("initial").unwrap();
        initial.transitions.push(Transition {
            when: None,
            r#do: vec![FullStatement {
                statement: MachineStatement {
                    operator: crate::machine::AssignmentOperator::Assign,
                    target: "MAX".to_string(),
                    expression: Expression::Value(crate::machine::Value::Bool(true)),
                },
                raw: "MAX = true".to_string(),
            }],
            target: "initial".to_string(),
        });
        let errors = validate_references(&machine);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.kind == CompileErrorKind::InvalidAssignmentTarget));
    }

    #[test]
    fn test_timer_as_assignment_target() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        machine.timers.insert(
            "t1".to_string(),
            crate::machine::Timer {
                r#type: Type::U32,
                when: None,
            },
        );
        let mut initial = machine.states.get_mut("initial").unwrap();
        initial.transitions.push(Transition {
            when: None,
            r#do: vec![FullStatement {
                statement: MachineStatement {
                    operator: crate::machine::AssignmentOperator::Assign,
                    target: "t1".to_string(),
                    expression: Expression::Value(crate::machine::Value::Integer(crate::machine::IntegerValue {
                        value: 0,
                        fmt: crate::machine::IntegerFmt::Dec,
                    })),
                },
                raw: "t1 = 0".to_string(),
            }],
            target: "initial".to_string(),
        });
        let errors = validate_references(&machine);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.kind == CompileErrorKind::InvalidAssignmentTarget));
    }

    #[test]
    fn test_state_reference_does_not_error() {
        let machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        // A machine with "state" as a variable would be rejected by reserved name validation,
        // but referencing "state" in an expression should not trigger unknown reference error.
        // We test this by checking the helper function directly.
        assert!(is_reserved_internal_name("state"));
    }

    #[test]
    fn test_multiple_errors_reported() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        // Add two unknown references in different contexts
        let signal_expr = Expression::Binary(
            Box::new(Expression::Reference(crate::machine::Reference {
                target: "counter".to_string(),
            })),
            crate::machine::BinaryOperator::Add,
            Box::new(Expression::Reference(crate::machine::Reference {
                target: "foo".to_string(),
            })),
        );
        machine.signals.insert(
            "result".to_string(),
            crate::machine::Signal {
                r#type: Type::U32,
                output: false,
                expr: signal_expr,
            },
        );
        let mut initial = machine.states.get_mut("initial").unwrap();
        initial.transitions.push(Transition {
            when: Some(FullExpression::parse("bar > baz").unwrap()),
            r#do: vec![],
            target: "initial".to_string(),
        });
        let errors = validate_references(&machine);
        assert_eq!(errors.len(), 3); // foo, bar, baz
        assert!(errors.iter().all(|e| e.kind == CompileErrorKind::UnknownVariableReference));
    }

    #[test]
    fn test_nested_expression_reference_check() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        // Deeply nested: (counter + unknown1) * (unknown2 - counter)
        let expr = FullExpression::parse("(counter + unknown1) * (unknown2 - counter)").unwrap();
        machine.signals.insert(
            "result".to_string(),
            crate::machine::Signal {
                r#type: Type::U32,
                output: false,
                expr: expr.expression,
            },
        );
        let errors = validate_references(&machine);
        assert_eq!(errors.len(), 2); // unknown1 and unknown2
        assert!(errors.iter().all(|e| e.kind == CompileErrorKind::UnknownVariableReference));
    }

    #[test]
    fn test_assignment_target_in_action() {
        let mut machine = make_machine_with_vars(&[var_pair("counter", Type::U32)]);
        machine.signals.insert(
            "flag".to_string(),
            crate::machine::Signal {
                r#type: Type::Bool,
                output: false,
                expr: Expression::Value(crate::machine::Value::Bool(true)),
            },
        );
        let mut initial = machine.states.get_mut("initial").unwrap();
        initial.actions.push(crate::machine::Action {
            when: None,
            r#do: vec![FullStatement {
                statement: MachineStatement {
                    operator: crate::machine::AssignmentOperator::Assign,
                    target: "flag".to_string(),
                    expression: Expression::Value(crate::machine::Value::Bool(true)),
                },
                raw: "flag = true".to_string(),
            }],
        });
        let errors = validate_references(&machine);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.kind == CompileErrorKind::InvalidAssignmentTarget));
        assert!(errors.iter().any(|e| e.message.contains("action")));
    }
}
