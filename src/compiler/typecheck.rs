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

/// Maps ExpressionId → ResolvedType for all expressions in a machine.
/// ResolvedType is either a definite type or a candidate set (for literals/constants).
pub type TypeEnv = HashMap<ExpressionId, ResolvedType>;

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
    /// Constants map (for value-based candidate resolution).
    constants: HashMap<String, Constant>,
    /// Inferred types: ExpressionId → ResolvedType.
    env: TypeEnv,
    /// Errors encountered during inference.
    errors: Vec<CompileError>,
}

impl TypeChecker {
    /// Create a new TypeChecker for a machine.
    pub fn new(scope: VariableScope, constants: HashMap<String, Constant>) -> Self {
        Self {
            next_id: 0,
            scope,
            constants,
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

    /// Insert a resolved type into the environment.
    fn insert(&mut self, id: ExpressionId, resolved: ResolvedType) {
        self.env.insert(id, resolved);
    }

    /// Get a resolved type from the environment.
    fn get(&self, id: ExpressionId) -> Option<&ResolvedType> {
        self.env.get(&id)
    }

    /// Infer types for all expressions in a machine.
    /// Returns the TypeEnv and any errors.
    pub fn infer_all_expressions(
        mut self,
        machine: &StateMachine,
    ) -> (TypeEnv, Vec<CompileError>) {
        // Infer types for all expressions in the machine.
        // All HashMap iterations are sorted by key for deterministic ExpressionId assignment.

        // 1. Signal expressions
        let mut signal_names: Vec<_> = machine.signals.keys().collect();
        signal_names.sort();
        for name in signal_names {
            let signal = machine.signals.get(name).unwrap();
            self.infer_expression(&signal.expr);
        }

        // 2. Timer when expressions (Timer.when is Expression, not FullExpression)
        let mut timer_names: Vec<_> = machine.timers.keys().collect();
        timer_names.sort();
        for name in timer_names {
            let timer = machine.timers.get(name).unwrap();
            if let Some(ref when_expr) = timer.when {
                self.infer_expression(when_expr);
            }
        }

        // 3. Transition when + do expressions (sorted for deterministic IDs)
        let mut state_names: Vec<_> = machine.states.keys().cloned().collect();
        state_names.sort();
        for state_name in &state_names {
            let state = machine.states.get(state_name).unwrap();
            for transition in &state.transitions {
                if let Some(ref when_expr) = transition.when {
                    self.infer_expression(&when_expr.expression);
                }
                for stmt in &transition.r#do {
                    self.infer_statement(stmt);
                }
            }
        }

        // 4. Action when + do expressions (same sorted order)
        for state_name in &state_names {
            let state = machine.states.get(state_name).unwrap();
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

    /// Infer type for a value literal — emits candidate set based on value.
    fn infer_value(&mut self, _expr: &Expression, val: &Value) -> Option<ExpressionId> {
        let id = self.alloc_id();
        let candidates = match val {
            Value::Integer(iv) => candidate_types_for_value(iv.value),
            Value::Float(fv) => candidate_types_for_float_value(fv.value),
            Value::Bool(_) => candidate_types_for_bool_value(),
            Value::String(_) => CandidateSet(vec![Type::String]),
        };
        self.insert(id, ResolvedType::Candidates(candidates));
        Some(id)
    }

    /// Infer type for a variable reference.
    /// Constants use value-based candidates (ignoring declared type).
    /// Unknown references are handled by the reference_validation pass.
    fn infer_reference(&mut self, _expr: &Expression, ref_: &Reference) -> Option<ExpressionId> {
        let id = self.alloc_id();
        // Constants always use value-based candidates (ignoring declared type)
        if let Some(constant) = self.constants.get(&ref_.target) {
            let candidates = candidate_types_for_constant(&constant.value);
            self.insert(id, ResolvedType::Candidates(candidates));
            return Some(id);
        }
        // Regular references use declared type
        if let Some(ty) = self.scope.get(&ref_.target) {
            self.insert(id, ResolvedType::Definite(ty.clone()));
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
        let inner_resolved = self.get(inner_id)?.clone();
        let id = self.alloc_id();
        self.insert(id, inner_resolved);
        Some(id)
    }

    /// Infer type for a unary operation — filters candidates through operator constraints.
    fn infer_unary(
        &mut self,
        _expr: &Expression,
        op: &UnaryOperator,
        inner: &Expression,
    ) -> Option<ExpressionId> {
        let inner_id = self.infer_expression(inner)?;
        let inner_resolved = self.get(inner_id)?;
        let inner_candidates = inner_resolved.to_candidates();
        let id = self.alloc_id();

        let result_resolved = match op {
            UnaryOperator::Not => {
                // Logical NOT always produces Bool
                if !is_truthy_candidate_set(&inner_candidates) {
                    self.errors.push(CompileError::new(
                        super::error::CompileErrorKind::InvalidSignalExpr,
                        format!("logical not requires truthy type, got {:?}", inner_candidates),
                    ));
                    return None;
                }
                ResolvedType::Definite(Type::Bool)
            }
            UnaryOperator::Negate => {
                let filtered = candidate_types_for_unary(&inner_candidates, UnaryOperator::Negate);
                if filtered.is_empty() {
                    self.errors.push(CompileError::new(
                        super::error::CompileErrorKind::InvalidSignalExpr,
                        format!("negation requires signed type, got {:?}", inner_candidates),
                    ));
                    return None;
                }
                ResolvedType::Candidates(filtered)
            }
            UnaryOperator::BitNot => {
                let filtered = candidate_types_for_unary(&inner_candidates, UnaryOperator::BitNot);
                if filtered.is_empty() {
                    self.errors.push(CompileError::new(
                        super::error::CompileErrorKind::InvalidSignalExpr,
                        format!("bitwise not requires unsigned integer type, got {:?}", inner_candidates),
                    ));
                    return None;
                }
                ResolvedType::Candidates(filtered)
            }
        };

        self.insert(id, result_resolved);
        Some(id)
    }

    /// Infer type for a binary operation — uses candidate set intersection.
    fn infer_binary(
        &mut self,
        _expr: &Expression,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Option<ExpressionId> {
        let left_id = self.infer_expression(left)?;
        let right_id = self.infer_expression(right)?;
        let left_resolved = self.get(left_id)?;
        let right_resolved = self.get(right_id)?;
        let left_candidates = left_resolved.to_candidates();
        let right_candidates = right_resolved.to_candidates();
        let id = self.alloc_id();

        match op {
            // Logical operators: both operands must be truthy, result is Bool
            BinaryOperator::LogicalAnd
            | BinaryOperator::LogicalOr
            | BinaryOperator::LogicalXor => {
                if is_truthy_candidate_set(&left_candidates) && is_truthy_candidate_set(&right_candidates) {
                    self.insert(id, ResolvedType::Definite(Type::Bool));
                    Some(id)
                } else {
                    self.errors.push(CompileError::new(
                        super::error::CompileErrorKind::InvalidSignalExpr,
                        format!(
                            "operator {:?} requires truthy operands, got {:?} and {:?}",
                            op, left_candidates, right_candidates
                        ),
                    ));
                    None
                }
            }
            // Comparison operators: operands get common type, result is Bool
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqual => {
                match find_common_type_sets(&left_candidates, &right_candidates) {
                    Some(_common) => {
                        // Result is Bool (comparison result)
                        self.insert(id, ResolvedType::Definite(Type::Bool));
                        Some(id)
                    }
                    None => {
                        self.errors.push(CompileError::new(
                            super::error::CompileErrorKind::InvalidSignalExpr,
                            format!(
                                "operator {:?} incompatible with types {:?} and {:?}",
                                op, left_candidates, right_candidates
                            ),
                        ));
                        None
                    }
                }
            }
            // Arithmetic, bitwise: result is common type
            _ => {
                match find_common_type_sets(&left_candidates, &right_candidates) {
                    Some(result_ty) => {
                        self.insert(id, ResolvedType::Definite(result_ty));
                        Some(id)
                    }
                    None => {
                        self.errors.push(CompileError::new(
                            super::error::CompileErrorKind::InvalidSignalExpr,
                            format!(
                                "operator {:?} incompatible with types {:?} and {:?}",
                                op, left_candidates, right_candidates
                            ),
                        ));
                        None
                    }
                }
            }
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

/// Check if all candidates in a set are truthy types.
fn is_truthy_candidate_set(candidates: &CandidateSet) -> bool {
    !candidates.is_empty() && candidates.iter().all(|t| is_truthy_type(t))
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Infer types for all expressions in all machines.
///
/// Returns a vector of (TypeEnv, errors) pairs, one per machine.
/// TypeEnv maps ExpressionId → ResolvedType (either definite or candidate set).
pub fn infer_all(
    machines: &[StateMachine],
) -> Vec<(TypeEnv, Vec<CompileError>)> {
    machines
        .iter()
        .map(|machine| {
            let scope = VariableScope::from_machine(machine);
            let constants = machine.constants.clone();
            let checker = TypeChecker::new(scope, constants);
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
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

        // Integer literal — emits candidate set
        let int_val = Expression::Value(Value::Integer(IntegerValue {
            value: 42,
            fmt: IntegerFmt::Dec,
        }));
        let id = checker.infer_expression(&int_val).unwrap();
        assert!(matches!(checker.get(id), Some(ResolvedType::Candidates(_))));
        assert_eq!(checker.get(id).unwrap().as_type(), Some(&Type::U8)); // best candidate

        // Bool literal — emits candidate set
        let bool_val = Expression::Value(Value::Bool(true));
        let id = checker.infer_expression(&bool_val).unwrap();
        assert!(matches!(checker.get(id), Some(ResolvedType::Candidates(_))));
        assert_eq!(checker.get(id).unwrap().as_type(), Some(&Type::Bool));

        // Float literal — emits candidate set
        let float_val = Expression::Value(Value::Float(
            FloatValue {
                value: 3.14,
                fmt: FloatFmt::Decimal,
            },
        ));
        let id = checker.infer_expression(&float_val).unwrap();
        assert!(matches!(checker.get(id), Some(ResolvedType::Candidates(_))));
        assert_eq!(checker.get(id).unwrap().as_type(), Some(&Type::F32)); // best candidate
    }

    #[test]
    fn test_infer_reference() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

        // Reference to counter (U16) — definite type
        let ref_expr = Expression::Reference(Reference {
            target: "counter".to_string(),
        });
        let id = checker.infer_expression(&ref_expr).unwrap();
        assert!(matches!(checker.get(id), Some(ResolvedType::Definite(Type::U16))));

        // Reference to flag (Bool) — definite type
        let ref_expr = Expression::Reference(Reference {
            target: "flag".to_string(),
        });
        let id = checker.infer_expression(&ref_expr).unwrap();
        assert!(matches!(checker.get(id), Some(ResolvedType::Definite(Type::Bool))));
    }

    #[test]
    fn test_infer_unknown_reference() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

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
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

        let neg_expr = Expression::Unary(
            UnaryOperator::Negate,
            Box::new(Expression::Reference(Reference {
                target: "counter".to_string(),
            })),
        );
        let id = checker.infer_expression(&neg_expr).unwrap();
        // counter is U16, negate filters to signed types: I16, I32, I64
        assert!(matches!(checker.get(id), Some(ResolvedType::Candidates(_))));
    }

    #[test]
    fn test_infer_unary_not() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

        let not_expr = Expression::Unary(
            UnaryOperator::Not,
            Box::new(Expression::Reference(Reference {
                target: "flag".to_string(),
            })),
        );
        let id = checker.infer_expression(&not_expr).unwrap();
        // Logical NOT always produces Bool
        assert!(matches!(checker.get(id), Some(ResolvedType::Definite(Type::Bool))));
    }

    #[test]
    fn test_infer_binary_add() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

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
        assert!(matches!(checker.get(id), Some(ResolvedType::Definite(Type::U32))));
    }

    #[test]
    fn test_infer_binary_logical_or() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

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
        assert!(matches!(checker.get(id), Some(ResolvedType::Definite(Type::Bool))));
    }

    #[test]
    fn test_infer_binary_comparison() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

        // counter (U16) > 0 (candidate set) → Bool
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
        assert!(matches!(checker.get(id), Some(ResolvedType::Definite(Type::Bool))));
    }

    #[test]
    fn test_expression_id_uniqueness() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

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
        assert!(matches!(checker.get(id1), Some(ResolvedType::Definite(Type::U16))));
        assert!(matches!(checker.get(id2), Some(ResolvedType::Definite(Type::U16))));
    }

    #[test]
    fn test_infer_parenthesis() {
        let machine = make_simple_machine();
        let scope = VariableScope::from_machine(&machine);
        let constants = HashMap::new();
        let mut checker = TypeChecker::new(scope, constants);

        let paren_expr = Expression::Parenthesis(Box::new(
            Expression::Reference(Reference {
                target: "counter".to_string(),
            }),
        ));
        let id = checker.infer_expression(&paren_expr).unwrap();
        assert!(matches!(checker.get(id), Some(ResolvedType::Definite(Type::U16))));
    }
}
