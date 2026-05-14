/// Type inference engine for the Pall compiler.
///
/// This module provides:
/// - `ExpressionId`: unique identifier for each AST node
/// - `TypeEnv`: maps ExpressionId → Type
/// - `VariableScope`: maps variable names → Type
/// - `TypeChecker`: walks the AST, assigns IDs, infers types
/// - `infer_all`: entry point for type inference on all machines

use std::collections::HashMap;

use crate::machine::{
    BinaryOperator, Constant, Expression, FloatFmt, FloatValue, FullExpression, FullStatement, Input,
    Reference, Signal, StateMachine, Timer, Type, UnaryOperator, Value, Variable,
};

use super::error::CompileError;
use super::typecheck_rules::*;

// ── ExpressionId ──────────────────────────────────────────────────────────────

/// Unique identifier for an AST node.
/// Each node gets a unique ID during traversal, even if the expression text is the same.
pub type ExpressionId = usize;

// ── TypeEnv ───────────────────────────────────────────────────────────────────

/// Maps ExpressionId → Type for all expressions in a machine.
pub type TypeEnv = HashMap<ExpressionId, Type>;

// ── VariableScope ─────────────────────────────────────────────────────────────

/// Maps variable names to their types for a single machine.
/// Populated from the machine's variables, inputs, signals, timers, and constants.
#[derive(Debug, Clone)]
pub struct VariableScope {
    variables: HashMap<String, Type>,
}

impl VariableScope {
    /// Create a scope from a StateMachine.
    pub fn from_machine(machine: &StateMachine) -> Self {
        let mut vars = HashMap::new();

        // Add variables
        for (name, var) in &machine.variables {
            vars.insert(name.clone(), var.r#type.clone());
        }

        // Add inputs
        for (name, input) in &machine.inputs {
            vars.insert(name.clone(), input.r#type.clone());
        }

        // Add signals
        for (name, sig) in &machine.signals {
            vars.insert(name.clone(), sig.r#type.clone());
        }

        // Add timers
        for (name, timer) in &machine.timers {
            vars.insert(name.clone(), timer.r#type.clone());
        }

        // Add constants
        for (name, constant) in &machine.constants {
            vars.insert(name.clone(), constant.r#type.clone());
        }

        Self { variables: vars }
    }

    /// Look up a variable's type by name.
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.variables.get(name)
    }

    /// Get all variable names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.variables.keys()
    }
}

// ── TypeChecker ───────────────────────────────────────────────────────────────

/// Type checker that walks an AST, assigns unique IDs, and infers types.
pub struct TypeChecker {
    /// Unique ID counter.
    next_id: ExpressionId,
    /// Scope for variable lookups.
    scope: VariableScope,
    /// Inferred types: ExpressionId → Type.
    env: TypeEnv,
    /// Errors encountered during inference.
    errors: Vec<CompileError>,
}

impl TypeChecker {
    /// Create a new TypeChecker for a machine.
    pub fn new(scope: VariableScope) -> Self {
        Self {
            next_id: 0,
            scope,
            env: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Allocate a new unique ExpressionId.
    fn alloc_id(&mut self) -> ExpressionId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Insert a type into the environment.
    fn insert(&mut self, id: ExpressionId, ty: Type) {
        self.env.insert(id, ty);
    }

    /// Get a type from the environment.
    fn get(&self, id: ExpressionId) -> Option<&Type> {
        self.env.get(&id)
    }

    /// Infer types for all expressions in a machine.
    /// Returns the TypeEnv and any errors.
    pub fn infer_all_expressions(
        mut self,
        machine: &StateMachine,
    ) -> (TypeEnv, Vec<CompileError>) {
        // Infer types for all expressions in the machine

        // 1. Signal expressions
        for (_name, signal) in &machine.signals {
            self.infer_expression(&signal.expr);
        }

        // 2. Timer when expressions (Timer.when is Expression, not FullExpression)
        for (_name, timer) in &machine.timers {
            if let Some(ref when_expr) = timer.when {
                self.infer_expression(when_expr);
            }
        }

        // 3. Transition when + do expressions
        for (_state_name, state) in &machine.states {
            for transition in &state.transitions {
                if let Some(ref when_expr) = transition.when {
                    self.infer_expression(&when_expr.expression);
                }
                for stmt in &transition.r#do {
                    self.infer_statement(stmt);
                }
            }
        }

        // 4. Action when + do expressions
        for (_state_name, state) in &machine.states {
            for action in &state.actions {
                if let Some(ref when_expr) = action.when {
                    self.infer_expression(&when_expr.expression);
                }
                for stmt in &action.r#do {
                    self.infer_statement(stmt);
                }
            }
        }

        (self.env, self.errors)
    }

    /// Infer types for a single expression.
    fn infer_expression(&mut self, expr: &Expression) -> Option<ExpressionId> {
        match expr {
            Expression::Value(val) => self.infer_value(expr, val),
            Expression::Reference(ref_) => self.infer_reference(expr, ref_),
            Expression::Parenthesis(inner) => self.infer_parenthesis(expr, inner),
            Expression::Unary(op, inner) => self.infer_unary(expr, op, inner),
            Expression::Binary(left, op, right) => self.infer_binary(expr, left, op, right),
        }
    }

    /// Infer type for a value literal.
    fn infer_value(&mut self, _expr: &Expression, val: &Value) -> Option<ExpressionId> {
        let id = self.alloc_id();
        let ty = match val {
            Value::Integer(iv) => {
                // Check if the value fits in i64
                if iv.value >= i64::MIN as i64 && iv.value <= i64::MAX as i64 {
                    Type::I64
                } else {
                    // Value overflows i64 — use u64
                    Type::U64
                }
            }
            Value::Float(_) => Type::F64,
            Value::String(_) => Type::String,
            Value::Bool(_) => Type::Bool,
        };
        self.insert(id, ty);
        Some(id)
    }

    /// Infer type for a variable reference.
    /// Unknown references are handled by the reference_validation pass with richer context.
    /// Returns None so downstream expressions short-circuit without producing spurious type errors.
    fn infer_reference(&mut self, _expr: &Expression, ref_: &Reference) -> Option<ExpressionId> {
        let id = self.alloc_id();
        if let Some(ty) = self.scope.get(&ref_.target) {
            self.insert(id, ty.clone());
            Some(id)
        } else {
            // Unknown references are caught by the reference_validation pass
            // which provides richer, contextual error messages.
            // Return None so downstream expressions short-circuit cleanly.
            None
        }
    }

    /// Infer type for a parenthesized expression.
    fn infer_parenthesis(
        &mut self,
        _expr: &Expression,
        inner: &Expression,
    ) -> Option<ExpressionId> {
        let inner_id = self.infer_expression(inner)?;
        let inner_ty = self.get(inner_id)?.clone();
        let id = self.alloc_id();
        self.insert(id, inner_ty);
        Some(id)
    }

    /// Infer type for a unary operation.
    fn infer_unary(
        &mut self,
        _expr: &Expression,
        op: &UnaryOperator,
        inner: &Expression,
    ) -> Option<ExpressionId> {
        let inner_id = self.infer_expression(inner)?;
        let inner_ty = self.get(inner_id)?.clone();
        let id = self.alloc_id();

        let result_ty = match op {
            UnaryOperator::Negate => {
                // Negate requires signed numeric type
                if !is_numeric_type(&inner_ty) {
                    self.errors.push(CompileError::new(
                        super::error::CompileErrorKind::InvalidSignalExpr,
                        format!("negation requires numeric type, got {:?}", inner_ty),
                    ));
                    return None;
                }
                // Result is same type as operand (signed numeric)
                inner_ty
            }
            UnaryOperator::Not => {
                // Not requires truthy type (Bool or numeric)
                if !is_truthy_type(&inner_ty) {
                    self.errors.push(CompileError::new(
                        super::error::CompileErrorKind::InvalidSignalExpr,
                        format!("logical not requires truthy type, got {:?}", inner_ty),
                    ));
                    return None;
                }
                Type::Bool
            }
            UnaryOperator::BitNot => {
                // BitNot requires integer type (not float, not Bool)
                if !is_integer_type(&inner_ty) {
                    self.errors.push(CompileError::new(
                        super::error::CompileErrorKind::InvalidSignalExpr,
                        format!("bitwise not requires integer type, got {:?}", inner_ty),
                    ));
                    return None;
                }
                inner_ty
            }
        };

        self.insert(id, result_ty);
        Some(id)
    }

    /// Infer type for a binary operation.
    fn infer_binary(
        &mut self,
        _expr: &Expression,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Option<ExpressionId> {
        let left_id = self.infer_expression(left)?;
        let right_id = self.infer_expression(right)?;
        let left_ty = self.get(left_id)?.clone();
        let right_ty = self.get(right_id)?.clone();
        let id = self.alloc_id();

        // Check operator-specific type compatibility
        if let Some(result_ty) = check_operator_compatibility(&left_ty, &right_ty, op) {
            self.insert(id, result_ty);
            Some(id)
        } else {
            self.errors.push(CompileError::new(
                super::error::CompileErrorKind::InvalidSignalExpr,
                format!(
                    "operator {:?} incompatible with types {:?} and {:?}",
                    op, left_ty, right_ty
                ),
            ));
            None
        }
    }

    /// Infer types for a statement (assignment).
    fn infer_statement(&mut self, stmt: &FullStatement) {
        self.infer_expression(&stmt.statement.expression);
        // Note: assignment target type checking is done in validation, not here
    }

    /// Get the final environment and errors.
    pub fn into_result(self) -> (TypeEnv, Vec<CompileError>) {
        (self.env, self.errors)
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Infer types for all expressions in all machines.
///
/// Returns a vector of (TypeEnv, errors) pairs, one per machine.
pub fn infer_all(
    machines: &[StateMachine],
) -> Vec<(TypeEnv, Vec<CompileError>)> {
    machines
        .iter()
        .map(|machine| {
            let scope = VariableScope::from_machine(machine);
            let checker = TypeChecker::new(scope);
            checker.infer_all_expressions(machine)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{
        FullStatement, IntegerFmt, IntegerValue, State, Transition,
        Statement as MachineStatement,
    };
    use std::collections::HashMap;

    fn make_simple_machine() -> StateMachine {
        let mut states = HashMap::new();
        states.insert(
            "initial".to_string(),
            State {
                actions: vec![],
                transitions: vec![],
            },
        );
        StateMachine {
            id: "test_machine".to_string(),
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
                    "flag".to_string(),
                    Variable {
                        r#type: Type::Bool,
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
            ]
            .into_iter()
            .collect(),
            constants: HashMap::new(),
        }
    }

    #[test]
    fn test_infer_value_types() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        // Integer literal
        let int_val = Expression::Value(Value::Integer(IntegerValue {
            value: 42,
            fmt: IntegerFmt::Dec,
        }));
        let id = checker.infer_expression(&int_val).unwrap();
        assert_eq!(checker.get(id), Some(&Type::I64));

        // Bool literal
        let bool_val = Expression::Value(Value::Bool(true));
        let id = checker.infer_expression(&bool_val).unwrap();
        assert_eq!(checker.get(id), Some(&Type::Bool));

        // Float literal
        let float_val = Expression::Value(Value::Float(
            FloatValue {
                value: 3.14,
                fmt: FloatFmt::Decimal,
            },
        ));
        let id = checker.infer_expression(&float_val).unwrap();
        assert_eq!(checker.get(id), Some(&Type::F64));
    }

    #[test]
    fn test_infer_reference() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        // Reference to counter (U16)
        let ref_expr = Expression::Reference(Reference {
            target: "counter".to_string(),
        });
        let id = checker.infer_expression(&ref_expr).unwrap();
        assert_eq!(checker.get(id), Some(&Type::U16));

        // Reference to flag (Bool)
        let ref_expr = Expression::Reference(Reference {
            target: "flag".to_string(),
        });
        let id = checker.infer_expression(&ref_expr).unwrap();
        assert_eq!(checker.get(id), Some(&Type::Bool));
    }

    #[test]
    fn test_infer_unknown_reference() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        let ref_expr = Expression::Reference(Reference {
            target: "nonexistent".to_string(),
        });
        let result = checker.infer_expression(&ref_expr);
        assert!(result.is_none());
    }

    #[test]
    fn test_infer_unary_negate() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        let neg_expr = Expression::Unary(
            UnaryOperator::Negate,
            Box::new(Expression::Reference(Reference {
                target: "counter".to_string(),
            })),
        );
        let id = checker.infer_expression(&neg_expr).unwrap();
        assert_eq!(checker.get(id), Some(&Type::U16));
    }

    #[test]
    fn test_infer_unary_not() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        let not_expr = Expression::Unary(
            UnaryOperator::Not,
            Box::new(Expression::Reference(Reference {
                target: "flag".to_string(),
            })),
        );
        let id = checker.infer_expression(&not_expr).unwrap();
        assert_eq!(checker.get(id), Some(&Type::Bool));
    }

    #[test]
    fn test_infer_binary_add() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        // x (U8) + y (U32) → U32
        let add_expr = Expression::Binary(
            Box::new(Expression::Reference(Reference {
                target: "x".to_string(),
            })),
            BinaryOperator::Add,
            Box::new(Expression::Reference(Reference {
                target: "y".to_string(),
            })),
        );
        let id = checker.infer_expression(&add_expr).unwrap();
        assert_eq!(checker.get(id), Some(&Type::U32));
    }

    #[test]
    fn test_infer_binary_logical_or() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        // flag || flag → Bool
        let or_expr = Expression::Binary(
            Box::new(Expression::Reference(Reference {
                target: "flag".to_string(),
            })),
            BinaryOperator::LogicalOr,
            Box::new(Expression::Reference(Reference {
                target: "flag".to_string(),
            })),
        );
        let id = checker.infer_expression(&or_expr).unwrap();
        assert_eq!(checker.get(id), Some(&Type::Bool));
    }

    #[test]
    fn test_infer_binary_comparison() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        // counter > 0 → Bool
        let gt_expr = Expression::Binary(
            Box::new(Expression::Reference(Reference {
                target: "counter".to_string(),
            })),
            BinaryOperator::GreaterThan,
            Box::new(Expression::Value(Value::Integer(IntegerValue {
                value: 0,
                fmt: IntegerFmt::Dec,
            }))),
        );
        let id = checker.infer_expression(&gt_expr).unwrap();
        assert_eq!(checker.get(id), Some(&Type::Bool));
    }

    #[test]
    fn test_expression_id_uniqueness() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        // Two identical expressions should get different IDs
        let ref1 = Expression::Reference(Reference {
            target: "counter".to_string(),
        });
        let ref2 = Expression::Reference(Reference {
            target: "counter".to_string(),
        });

        let id1 = checker.infer_expression(&ref1).unwrap();
        let id2 = checker.infer_expression(&ref2).unwrap();

        assert_ne!(id1, id2, "Two expressions should have different IDs");
        assert_eq!(checker.get(id1), Some(&Type::U16));
        assert_eq!(checker.get(id2), Some(&Type::U16));
    }

    #[test]
    fn test_infer_parenthesis() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let mut checker = TypeChecker::new(scope);

        let paren_expr = Expression::Parenthesis(Box::new(
            Expression::Reference(Reference {
                target: "counter".to_string(),
            }),
        ));
        let id = checker.infer_expression(&paren_expr).unwrap();
        assert_eq!(checker.get(id), Some(&Type::U16));
    }
}
