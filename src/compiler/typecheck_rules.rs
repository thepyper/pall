/// Type casting rules for the Pall compiler.
/// All casting is lossless — only allows casting to types that fully contain the source type.
///
/// Key rules:
/// - Bool → all numeric types
/// - Unsigned → larger unsigned + signed (never to smaller types)
/// - Signed → larger signed only (never to unsigned)
/// - F32 → F64 only
/// - Int → Float: limited by mantissa size (i16→f32, i32→f64, etc.)
/// - No signed → unsigned casting

use crate::machine::{BinaryOperator, Type, UnaryOperator, Value};

// ── Candidate sets and resolved types ─────────────────────────────────────────

/// A set of candidate types — all types that can represent a value without loss.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateSet(pub Vec<Type>);

impl CandidateSet {
    /// Get the best (smallest) candidate type.
    pub fn best(&self) -> Option<&Type> {
        self.0.first()
    }

    /// Check if a type is in this set.
    pub fn contains(&self, ty: &Type) -> bool {
        self.0.contains(ty)
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over the types in the set.
    pub fn iter(&self) -> impl Iterator<Item = &Type> {
        self.0.iter()
    }
}

/// The resolved type of an expression — either a definite type or a candidate set.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedType {
    Definite(Type),
    Candidates(CandidateSet),
}

impl ResolvedType {
    /// Get the type if definite, or the first (best) candidate.
    pub fn as_type(&self) -> Option<&Type> {
        match self {
            ResolvedType::Definite(t) => Some(t),
            ResolvedType::Candidates(cs) => cs.best(),
        }
    }

    /// Convert to a candidate set (wrap definite type in a set).
    pub fn to_candidates(&self) -> CandidateSet {
        match self {
            ResolvedType::Definite(t) => CandidateSet(vec![t.clone()]),
            ResolvedType::Candidates(cs) => cs.clone(),
        }
    }
}

// ── Value-based candidate type generation ───────────────────────────────────────

/// Get the default type for a value.
pub fn value_default_type(value: &Value) -> Type {
    match value {
        Value::Integer(_) => Type::I64,
        Value::Float(_) => Type::F64,
        Value::Bool(_) => Type::Bool,
        Value::String(_) => Type::String,
    }
}

/// Compute the candidate type set for an integer literal value.
/// Returns all types that can hold the value without loss, ordered smallest-first.
pub fn candidate_types_for_value(value: i64) -> CandidateSet {
    let mut candidates = Vec::new();
    let all_types = [
        Type::U8, Type::I8, Type::U16, Type::I16,
        Type::U32, Type::I32, Type::U64, Type::I64,
        Type::F32, Type::F64,
    ];
    for ty in &all_types {
        if int_value_fits(value, ty) {
            candidates.push(ty.clone());
        }
    }
    CandidateSet(candidates)
}

/// Check if an i64 value fits in the given type without loss.
fn int_value_fits(value: i64, ty: &Type) -> bool {
    match ty {
        Type::U8 => value >= 0 && value <= u8::MAX as i64,
        Type::U16 => value >= 0 && value <= u16::MAX as i64,
        Type::U32 => value >= 0 && value <= u32::MAX as i64,
        Type::U64 => value >= 0,
        Type::I8 => value >= i8::MIN as i64 && value <= i8::MAX as i64,
        Type::I16 => value >= i16::MIN as i64 && value <= i16::MAX as i64,
        Type::I32 => value >= i32::MIN as i64 && value <= i32::MAX as i64,
        Type::I64 => true,
        Type::F32 => value.abs() <= (1 << 24) as i64,
        Type::F64 => value.abs() <= (1i64 << 52),
        _ => false,
    }
}

/// Compute the candidate type set for a float literal value.
/// Float literals only have F32 and F64 as candidates.
pub fn candidate_types_for_float_value(_value: f64) -> CandidateSet {
    CandidateSet(vec![Type::F32, Type::F64])
}

/// Compute the candidate type set for a boolean value.
pub fn candidate_types_for_bool_value() -> CandidateSet {
    CandidateSet(vec![Type::Bool])
}

/// Compute the candidate type set for a constant value.
pub fn candidate_types_for_constant(value: &Value) -> CandidateSet {
    match value {
        Value::Integer(iv) => candidate_types_for_value(iv.value),
        Value::Float(fv) => candidate_types_for_float_value(fv.value),
        Value::Bool(_) => candidate_types_for_bool_value(),
        Value::String(_) => CandidateSet(vec![Type::String]),
    }
}

/// Apply unary operator constraints to a candidate set.
/// For each candidate type, finds the smallest operator-compatible type it can cast to.
/// For literals: filters the candidate set to operator-compatible types.
/// For typed references: finds the cast target in the operator's allowed set.
pub fn candidate_types_for_unary(candidates: &CandidateSet, op: UnaryOperator) -> CandidateSet {
    match op {
        UnaryOperator::Not => {
            // Logical NOT always produces Bool
            if candidates.iter().all(|t| is_truthy_type(t)) {
                CandidateSet(vec![Type::Bool])
            } else {
                CandidateSet(vec![])
            }
        }
        UnaryOperator::Negate => {
            // Negation produces signed types. Find the smallest signed type each candidate can cast to.
            let signed_types = [
                Type::I8, Type::I16, Type::I32, Type::I64, Type::F32, Type::F64,
            ];
            let mut result = Vec::new();
            for inner_ty in &candidates.0 {
                for target in &signed_types {
                    if is_cast_lossless(inner_ty, target) && !result.contains(target) {
                        result.push(target.clone());
                    }
                }
            }
            // Sort by bits, prefer smaller
            result.sort_by_key(|t| get_target_bits(t));
            CandidateSet(result)
        }
        UnaryOperator::BitNot => {
            // Bitwise NOT works on any integer type (signed or unsigned), keeps same type.
            // Filter to integer types, keeping the same types from candidates.
            let int_types = [
                Type::U8, Type::I8, Type::U16, Type::I16,
                Type::U32, Type::I32, Type::U64, Type::I64,
            ];
            let mut result = Vec::new();
            for inner_ty in &candidates.0 {
                for target in &int_types {
                    if is_cast_lossless(inner_ty, target) && !result.contains(target) {
                        result.push(target.clone());
                    }
                }
            }
            result.sort_by_key(|t| get_target_bits(t));
            CandidateSet(result)
        }
    }
}

/// Check if a literal/constant value can be assigned to a target type.
pub fn is_cast_lossless_value(value: &Value, target_type: &Type) -> bool {
    if value_default_type(value) == *target_type {
        return true;
    }
    match (value, target_type) {
        (Value::Integer(iv), _) => int_value_fits(iv.value, target_type),
        (Value::Float(_), Type::F32 | Type::F64) => true,
        (Value::Float(_), _) => false,
        (Value::Bool(_), _) => is_numeric_type(target_type),
        (Value::String(_), Type::String) => true,
        (Value::String(_), _) => false,
    }
}

/// Find the best common type between two candidate sets.
/// Returns the smallest type that both operand sets can cast to.
pub fn find_common_type_sets(set_a: &CandidateSet, set_b: &CandidateSet) -> Option<Type> {
    let all_types = [
        Type::Bool, Type::U8, Type::U16, Type::U32, Type::U64,
        Type::I8, Type::I16, Type::I32, Type::I64,
        Type::F32, Type::F64,
    ];
    let candidates: Vec<Type> = all_types.iter()
        .filter(|target| {
            set_a.iter().any(|t| is_cast_lossless(t, target))
                && set_b.iter().any(|t| is_cast_lossless(t, target))
        })
        .cloned()
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let mut sorted = candidates;
    sorted.sort_by_key(|t| (get_target_bits(t), !is_unsigned(t) as u8));
    sorted.first().cloned()
}

// ── Type classification ───────────────────────────────────────────────────────

/// Check if a type is a numeric type (all integers + floats, not Bool/String).
pub fn is_numeric_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::F32
            | Type::F64
    )
}

/// Check if a type is an integer type (not float, not Bool, not String).
pub fn is_integer_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
    )
}

/// Check if a type is Bool.
pub fn is_bool_type(ty: &Type) -> bool {
    matches!(ty, Type::Bool)
}

/// Check if a type is truthy (Bool or any numeric type).
/// In C++-style: 0 = false, non-zero = true.
pub fn is_truthy_type(ty: &Type) -> bool {
    is_bool_type(ty) || is_numeric_type(ty)
}

/// Check if a type is unsigned.
pub fn is_unsigned(ty: &Type) -> bool {
    matches!(
        ty,
        Type::U8 | Type::U16 | Type::U32 | Type::U64
    )
}

/// Get the bit width of a type for ordering purposes.
pub fn get_target_bits(ty: &Type) -> u32 {
    match ty {
        Type::Bool => 1,
        Type::U8 | Type::I8 => 8,
        Type::U16 | Type::I16 => 16,
        Type::U32 | Type::I32 => 32,
        Type::U64 | Type::I64 => 64,
        Type::F32 => 32,
        Type::F64 => 64,
        Type::String => 0, // Not used for numeric casting
    }
}

/// Get the Rust type string for a given Pall Type.
pub fn type_to_rust_str(ty: &Type) -> &'static str {
    match ty {
        Type::Bool => "bool",
        Type::U8 => "u8",
        Type::U16 => "u16",
        Type::U32 => "u32",
        Type::U64 => "u64",
        Type::I8 => "i8",
        Type::I16 => "i16",
        Type::I32 => "i32",
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
        Type::String => "String",
    }
}

// ── Lossless casting rules ────────────────────────────────────────────────────

/// Check if casting from `from` to `to` is lossless.
///
/// Rules:
/// - Bool → all numeric
/// - Unsigned → larger unsigned + larger signed (but NOT to smaller signed that could lose sign)
/// - Signed → larger signed only (never to unsigned — sign conversion is not lossless)
/// - F32 → F64
/// - Int → Float: only if the mantissa can perfectly represent the integer type
///
/// Mantissa limits:
/// - f32: 23 bits of mantissa → can represent all i16/u16 perfectly (16 bits)
/// - f64: 52 bits of mantissa → can represent all i32/u32 perfectly (32 bits)
/// - i64/u64 cannot be losslessly cast to f64 (64 > 52 bits)
pub fn is_cast_lossless(from: &Type, to: &Type) -> bool {
    if from == to {
        return true;
    }

    // Bool → all numeric
    if is_bool_type(from) {
        return is_numeric_type(to);
    }

    // F32 → F64
    if matches!(from, Type::F32) && matches!(to, Type::F64) {
        return true;
    }

    // Int → Float: only if mantissa can represent perfectly
    if is_integer_type(from) && matches!(to, Type::F32 | Type::F64) {
        return int_to_float_lossless(from, to);
    }

    // Float → Int: NEVER (precision loss)
    if matches!(from, Type::F32 | Type::F64) && is_integer_type(to) {
        return false;
    }

    // Integer to Integer
    if is_integer_type(from) && is_integer_type(to) {
        return int_to_int_lossless(from, to);
    }

    false
}

/// Check if integer → float casting is lossless (based on mantissa size).
fn int_to_float_lossless(int_ty: &Type, float_ty: &Type) -> bool {
    match (int_ty, float_ty) {
        // u8/u16/i8/i16 can fit in f32 mantissa (24 bits including implicit)
        (Type::U8 | Type::I8 | Type::U16 | Type::I16, Type::F32) => true,
        // u32/i32 can fit in f64 mantissa (52 bits)
        (Type::U32 | Type::I32, Type::F64) => true,
        // Small ints can also cast to f64
        (Type::U8 | Type::I8, Type::F64) => true,
        (Type::U16 | Type::I16, Type::F64) => true,
        _ => false,
    }
}

/// Check if integer → integer casting is lossless.
/// Uses explicit type matching — no bit-counting magic.
fn int_to_int_lossless(from: &Type, to: &Type) -> bool {
    if from == to {
        return true;
    }
    match (from, to) {
        // Unsigned → larger unsigned
        (Type::U8, Type::U16 | Type::U32 | Type::U64) => true,
        (Type::U16, Type::U32 | Type::U64) => true,
        (Type::U32, Type::U64) => true,
        // Unsigned → larger signed (target must hold ALL unsigned values)
        (Type::U8, Type::I16 | Type::I32 | Type::I64) => true,
        (Type::U16, Type::I32 | Type::I64) => true,
        (Type::U32, Type::I64) => true,
        // Signed → larger signed
        (Type::I8, Type::I16 | Type::I32 | Type::I64) => true,
        (Type::I16, Type::I32 | Type::I64) => true,
        (Type::I32, Type::I64) => true,
        // Never: signed → unsigned (sign conversion is not lossless)
        (Type::I8 | Type::I16 | Type::I32 | Type::I64, Type::U8 | Type::U16 | Type::U32 | Type::U64) => false,
        // Catch-all: no other casts allowed
        _ => false,
    }
}

// ── Common type resolution ────────────────────────────────────────────────────

/// Find a common type that both `from` and `to` can cast to losslessly.
///
/// Algorithm:
/// 1. Find all types that both `from` and `to` can cast to (intersection of cast targets)
/// 2. Among candidates, pick the smallest (fewest bits)
/// 3. If two candidates have the same bit count, prefer unsigned (since both were unsigned)
/// 4. Return None if no common type exists
///
/// Note: For logical operators (&&, ||, ^^), no common numeric type is needed —
/// both operands just need to be truthy. This is handled separately.
pub fn find_common_type(from: &Type, to: &Type) -> Option<Type> {
    // Same type — no cast needed
    if from == to {
        return Some(from.clone());
    }

    // Bool + numeric: Bool casts to all numeric, so common type is the numeric type
    if is_bool_type(from) && is_numeric_type(to) {
        return Some(to.clone());
    }
    if is_numeric_type(from) && is_bool_type(to) {
        return Some(from.clone());
    }

    // String is not compatible with anything for arithmetic
    if matches!(from, Type::String) || matches!(to, Type::String) {
        return None;
    }

    // Both numeric — find common numeric type
    if is_numeric_type(from) && is_numeric_type(to) {
        return find_common_numeric_type(from, to);
    }

    // Bool + Bool
    if is_bool_type(from) && is_bool_type(to) {
        return Some(Type::Bool);
    }

    None
}

/// Find a common numeric type for two numeric types.
fn find_common_numeric_type(from: &Type, to: &Type) -> Option<Type> {
    find_common_type_sets(&CandidateSet(vec![from.clone()]), &CandidateSet(vec![to.clone()]))
}

// ── Operator-specific type compatibility ──────────────────────────────────────

/// Check if an operator's operands are type-compatible.
/// Returns the expected result type, or None if incompatible.
pub fn check_operator_compatibility(
    left_type: &Type,
    right_type: &Type,
    op: &BinaryOperator,
) -> Option<Type> {
    match op {
        // Arithmetic: both must be numeric
        BinaryOperator::Add
        | BinaryOperator::Sub
        | BinaryOperator::Mul
        | BinaryOperator::Div
        | BinaryOperator::Mod => {
            if !is_numeric_type(left_type) || !is_numeric_type(right_type) {
                return None;
            }
            find_common_numeric_type(left_type, right_type)
        }

        // Bitwise: both must be integer (not float, not Bool)
        // And/Or/Xor are legacy names; BitAnd/BitOr/BitXor are current
        BinaryOperator::And
        | BinaryOperator::Or
        | BinaryOperator::Xor
        | BinaryOperator::BitAnd
        | BinaryOperator::BitOr
        | BinaryOperator::BitXor => {
            if !is_integer_type(left_type) || !is_integer_type(right_type) {
                return None;
            }
            find_common_numeric_type(left_type, right_type)
        }

        // Logical: both must be truthy, result is Bool
        // LogicalAnd/LogicalOr/LogicalXor are current names
        BinaryOperator::LogicalAnd
        | BinaryOperator::LogicalOr
        | BinaryOperator::LogicalXor => {
            if !is_truthy_type(left_type) || !is_truthy_type(right_type) {
                return None;
            }
            Some(Type::Bool)
        }

        // Comparison: can be any type; if different, need common type
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            if left_type == right_type {
                Some(Type::Bool)
            } else {
                find_common_type(left_type, right_type)
                    .map(|_| Type::Bool)
            }
        }

        // Ordering: both must be numeric
        BinaryOperator::LessThan
        | BinaryOperator::LessEqual
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterEqual => {
            if !is_numeric_type(left_type) || !is_numeric_type(right_type) {
                return None;
            }
            find_common_numeric_type(left_type, right_type)
                .map(|_| Type::Bool)
        }
    }
}

// ── Truthiness for expressions ────────────────────────────────────────────────

/// Check if an expression is truthy (can be used as a boolean condition).
/// This is recursive: it checks the expression structure, not just the type.
pub fn is_truthy_expression(expr: &crate::machine::Expression) -> bool {
    match expr {
        crate::machine::Expression::Value(val) => {
            // Bool literal is truthy
            matches!(val, crate::machine::Value::Bool(_))
            // Numeric literals are truthy (0 is "falsy" in runtime, but the TYPE is truthy)
            // Actually, we check TYPE truthiness, not value truthiness
        }
        crate::machine::Expression::Value(_) => false,
        crate::machine::Expression::Reference(_) => {
            // We can't determine truthiness of a reference without type info
            // This should be handled by the TypeEnv lookup
            false
        }
        crate::machine::Expression::Parenthesis(inner) => {
            is_truthy_expression(inner)
        }
        crate::machine::Expression::Unary(op, inner) => {
            match op {
                // ! returns Bool, which is truthy
                crate::machine::UnaryOperator::Not => true,
                // - returns numeric, which is truthy
                crate::machine::UnaryOperator::Negate => true,
                // ~ returns integer, which is truthy
                crate::machine::UnaryOperator::BitNot => true,
            }
        }
        crate::machine::Expression::Binary(_, binary_op, _) => {
            // Comparison and logical operators return Bool, which is truthy
            matches!(
                binary_op,
                BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::LessThan
                | BinaryOperator::LessEqual
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterEqual
                | BinaryOperator::LogicalOr
                | BinaryOperator::LogicalAnd
                | BinaryOperator::LogicalXor
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Type classification tests ─────────────────────────────────────────────

    #[test]
    fn test_is_numeric_type() {
        assert!(is_numeric_type(&Type::U8));
        assert!(is_numeric_type(&Type::I32));
        assert!(is_numeric_type(&Type::F64));
        assert!(!is_numeric_type(&Type::Bool));
        assert!(!is_numeric_type(&Type::String));
    }

    #[test]
    fn test_is_integer_type() {
        assert!(is_integer_type(&Type::U8));
        assert!(is_integer_type(&Type::I64));
        assert!(!is_integer_type(&Type::F32));
        assert!(!is_integer_type(&Type::Bool));
    }

    #[test]
    fn test_is_bool_type() {
        assert!(is_bool_type(&Type::Bool));
        assert!(!is_bool_type(&Type::U8));
    }

    #[test]
    fn test_is_truthy_type() {
        assert!(is_truthy_type(&Type::Bool));
        assert!(is_truthy_type(&Type::U8));
        assert!(is_truthy_type(&Type::I32));
        assert!(is_truthy_type(&Type::F64));
        assert!(!is_truthy_type(&Type::String));
    }

    #[test]
    fn test_is_unsigned() {
        assert!(is_unsigned(&Type::U8));
        assert!(!is_unsigned(&Type::I8));
    }

    #[test]
    fn test_get_target_bits() {
        assert_eq!(get_target_bits(&Type::Bool), 1);
        assert_eq!(get_target_bits(&Type::U8), 8);
        assert_eq!(get_target_bits(&Type::U16), 16);
        assert_eq!(get_target_bits(&Type::U32), 32);
        assert_eq!(get_target_bits(&Type::U64), 64);
        assert_eq!(get_target_bits(&Type::I32), 32);
        assert_eq!(get_target_bits(&Type::F64), 64);
    }

    // ── Lossless casting tests ────────────────────────────────────────────────

    #[test]
    fn test_bool_casts_to_all_numeric() {
        assert!(is_cast_lossless(&Type::Bool, &Type::U8));
        assert!(is_cast_lossless(&Type::Bool, &Type::U64));
        assert!(is_cast_lossless(&Type::Bool, &Type::I8));
        assert!(is_cast_lossless(&Type::Bool, &Type::I64));
        assert!(is_cast_lossless(&Type::Bool, &Type::F32));
        assert!(is_cast_lossless(&Type::Bool, &Type::F64));
    }

    #[test]
    fn test_unsigned_casts_larger() {
        assert!(is_cast_lossless(&Type::U8, &Type::U16));
        assert!(is_cast_lossless(&Type::U8, &Type::U32));
        assert!(is_cast_lossless(&Type::U16, &Type::U32));
        assert!(is_cast_lossless(&Type::U32, &Type::U64));
    }

    #[test]
    fn test_unsigned_casts_larger_signed() {
        assert!(is_cast_lossless(&Type::U8, &Type::I16));
        assert!(is_cast_lossless(&Type::U16, &Type::I32));
        assert!(is_cast_lossless(&Type::U32, &Type::I64));
    }

    #[test]
    fn test_unsigned_does_not_cast_smaller_signed() {
        assert!(!is_cast_lossless(&Type::U8, &Type::I8)); // i8 can't hold 128..255
    }

    #[test]
    fn test_signed_casts_larger_signed_only() {
        assert!(is_cast_lossless(&Type::I8, &Type::I16));
        assert!(is_cast_lossless(&Type::I8, &Type::I32));
        assert!(is_cast_lossless(&Type::I32, &Type::I64));
    }

    #[test]
    fn test_no_signed_to_unsigned() {
        assert!(!is_cast_lossless(&Type::I8, &Type::U8));
        assert!(!is_cast_lossless(&Type::I8, &Type::U16));
        assert!(!is_cast_lossless(&Type::I32, &Type::U32));
        assert!(!is_cast_lossless(&Type::I64, &Type::U64));
    }

    #[test]
    fn test_f32_to_f64() {
        assert!(is_cast_lossless(&Type::F32, &Type::F64));
    }

    #[test]
    fn test_no_float_to_int() {
        assert!(!is_cast_lossless(&Type::F32, &Type::U8));
        assert!(!is_cast_lossless(&Type::F64, &Type::I32));
    }

    #[test]
    fn test_int_to_float_lossless() {
        // Small ints can cast to f32
        assert!(is_cast_lossless(&Type::U8, &Type::F32));
        assert!(is_cast_lossless(&Type::I8, &Type::F32));
        assert!(is_cast_lossless(&Type::U16, &Type::F32));
        assert!(is_cast_lossless(&Type::I16, &Type::F32));

        // Medium ints can cast to f64
        assert!(is_cast_lossless(&Type::U32, &Type::F64));
        assert!(is_cast_lossless(&Type::I32, &Type::F64));

        // Small ints can also cast to f64
        assert!(is_cast_lossless(&Type::U8, &Type::F64));
        assert!(is_cast_lossless(&Type::I8, &Type::F64));

        // Large ints cannot cast to f64 (mantissa too small)
        assert!(!is_cast_lossless(&Type::U64, &Type::F64));
        assert!(!is_cast_lossless(&Type::I64, &Type::F64));
    }

    #[test]
    fn test_same_type() {
        assert!(is_cast_lossless(&Type::U8, &Type::U8));
        assert!(is_cast_lossless(&Type::I32, &Type::I32));
    }

    // ── Common type tests ─────────────────────────────────────────────────────

    #[test]
    fn test_common_type_same() {
        assert_eq!(find_common_type(&Type::U8, &Type::U8), Some(Type::U8));
    }

    #[test]
    fn test_common_type_unsigned_pair() {
        // u8 + u16 → u16
        assert_eq!(find_common_type(&Type::U8, &Type::U16), Some(Type::U16));
        // u8 + u32 → u32
        assert_eq!(find_common_type(&Type::U8, &Type::U32), Some(Type::U32));
    }

    #[test]
    fn test_common_type_signed_unsigned_pair() {
        // i8 + u16 → i32 (i8 needs i16+, u16 needs i32+ to avoid signed→unsigned)
        // Wait, u16 → i32 is OK (32 > 16 + 1 = 17? No, 32 > 16). Let me check.
        // Actually: u16 → i32: to_bits(32) > from_bits(16)? Yes. So OK.
        // And i8 → i32: signed → larger signed, OK.
        // So common type is i32.
        let result = find_common_type(&Type::I8, &Type::U16);
        assert!(result.is_some(), "i8 + u16 should have a common type");
        assert_eq!(get_target_bits(&result.unwrap()), 32);
    }

    #[test]
    fn test_common_type_unsigned_priority() {
        // u32 and i32 have same bit count; u32 is preferred (unsigned priority)
        let result = find_common_type(&Type::U32, &Type::U32);
        assert_eq!(result, Some(Type::U32));
    }

    #[test]
    fn test_common_type_bool_plus_numeric() {
        assert_eq!(find_common_type(&Type::Bool, &Type::U8), Some(Type::U8));
        assert_eq!(find_common_type(&Type::U16, &Type::Bool), Some(Type::U16));
    }

    #[test]
    fn test_common_type_incompatible() {
        // String + anything numeric → None
        assert!(find_common_type(&Type::String, &Type::U8).is_none());
    }

    // ── Operator compatibility tests ──────────────────────────────────────────

    #[test]
    fn test_arithmetic_requires_numeric() {
        assert!(check_operator_compatibility(&Type::U8, &Type::U16, &BinaryOperator::Add).is_some());
        assert!(check_operator_compatibility(&Type::Bool, &Type::U8, &BinaryOperator::Add).is_none());
    }

    #[test]
    fn test_bitwise_requires_integer() {
        assert!(check_operator_compatibility(&Type::U8, &Type::U16, &BinaryOperator::And).is_some());
        assert!(check_operator_compatibility(&Type::F32, &Type::F64, &BinaryOperator::And).is_none());
        assert!(check_operator_compatibility(&Type::Bool, &Type::Bool, &BinaryOperator::And).is_none());
    }

    #[test]
    fn test_logical_requires_truthy() {
        assert!(check_operator_compatibility(&Type::Bool, &Type::Bool, &BinaryOperator::LogicalAnd).is_some());
        assert!(check_operator_compatibility(&Type::U8, &Type::Bool, &BinaryOperator::LogicalAnd).is_some());
        assert!(check_operator_compatibility(&Type::String, &Type::Bool, &BinaryOperator::LogicalAnd).is_none());
    }

    #[test]
    fn test_comparison_returns_bool() {
        let result = check_operator_compatibility(&Type::U8, &Type::U16, &BinaryOperator::Equal);
        assert_eq!(result, Some(Type::Bool));
    }

    #[test]
    fn test_ordering_requires_numeric() {
        assert!(check_operator_compatibility(&Type::U8, &Type::U16, &BinaryOperator::LessThan).is_some());
        assert!(check_operator_compatibility(&Type::Bool, &Type::Bool, &BinaryOperator::LessThan).is_none());
    }

    // ── Truthiness expression tests ───────────────────────────────────────────

    #[test]
    fn test_truthy_expression_bool_literal() {
        let expr = crate::machine::Expression::Value(crate::machine::Value::Bool(true));
        assert!(is_truthy_expression(&expr));
    }

    #[test]
    fn test_truthy_expression_comparison() {
        let left = crate::machine::Expression::Value(crate::machine::Value::Integer(
            crate::machine::IntegerValue { value: 1, fmt: crate::machine::IntegerFmt::Dec }
        ));
        let right = crate::machine::Expression::Value(crate::machine::Value::Integer(
            crate::machine::IntegerValue { value: 0, fmt: crate::machine::IntegerFmt::Dec }
        ));
        let expr = crate::machine::Expression::Binary(
            Box::new(left),
            BinaryOperator::GreaterThan,
            Box::new(right),
        );
        assert!(is_truthy_expression(&expr));
    }

    #[test]
    fn test_truthy_expression_string_literal() {
        let expr = crate::machine::Expression::Value(crate::machine::Value::String(
            crate::machine::StringValue { value: "hello".into(), fmt: crate::machine::StringFmt::DoubleQuote }
        ));
        assert!(!is_truthy_expression(&expr));
    }

    // ── Numeric cast restrictions tests ───────────────────────────────────────

    #[test]
    fn test_i16_to_f32_lossless() {
        // i16: -32768 to 32767, f32 mantissa: 24 bits → can represent all i16 perfectly
        assert!(is_cast_lossless(&Type::I16, &Type::F32));
        assert!(is_cast_lossless(&Type::U16, &Type::F32));
    }

    #[test]
    fn test_i32_to_f64_lossless() {
        // i32: ±2^31, f64 mantissa: 52 bits → can represent all i32 perfectly
        assert!(is_cast_lossless(&Type::I32, &Type::F64));
        assert!(is_cast_lossless(&Type::U32, &Type::F64));
    }

    #[test]
    fn test_i64_to_f64_not_lossless() {
        // i64: ±2^63, f64 mantissa: 52 bits → cannot represent all i64 perfectly
        assert!(!is_cast_lossless(&Type::I64, &Type::F64));
        assert!(!is_cast_lossless(&Type::U64, &Type::F64));
    }

    // ── Candidate set tests ───────────────────────────────────────────────────

    #[test]
    fn test_candidate_types_for_value_positive() {
        let cs = candidate_types_for_value(42);
        // 42 fits in all integer types and floats
        assert!(cs.contains(&Type::U8));
        assert!(cs.contains(&Type::I8));
        assert!(cs.contains(&Type::U16));
        assert!(cs.contains(&Type::I16));
        assert!(cs.contains(&Type::U32));
        assert!(cs.contains(&Type::I32));
        assert!(cs.contains(&Type::U64));
        assert!(cs.contains(&Type::I64));
        assert!(cs.contains(&Type::F32));
        assert!(cs.contains(&Type::F64));
        // Smallest is U8
        assert_eq!(cs.best(), Some(&Type::U8));
    }

    #[test]
    fn test_candidate_types_for_value_negative() {
        let cs = candidate_types_for_value(-42);
        // -42 doesn't fit in unsigned types
        assert!(!cs.contains(&Type::U8));
        assert!(!cs.contains(&Type::U16));
        assert!(!cs.contains(&Type::U32));
        assert!(!cs.contains(&Type::U64));
        // Fits in signed types and floats
        assert!(cs.contains(&Type::I8));
        assert!(cs.contains(&Type::I16));
        assert!(cs.contains(&Type::I32));
        assert!(cs.contains(&Type::I64));
        assert!(cs.contains(&Type::F32));
        assert!(cs.contains(&Type::F64));
        // Smallest is I8
        assert_eq!(cs.best(), Some(&Type::I8));
    }

    #[test]
    fn test_candidate_types_for_value_zero() {
        let cs = candidate_types_for_value(0);
        // 0 fits in all types
        assert_eq!(cs.0.len(), 10);
        assert_eq!(cs.best(), Some(&Type::U8));
    }

    #[test]
    fn test_candidate_types_for_unary_negate() {
        let int_cs = candidate_types_for_value(42);
        let filtered = candidate_types_for_unary(&int_cs, UnaryOperator::Negate);
        // Negate filters to signed types
        assert!(!filtered.contains(&Type::U8));
        assert!(!filtered.contains(&Type::U16));
        assert!(!filtered.contains(&Type::U32));
        assert!(!filtered.contains(&Type::U64));
        assert!(filtered.contains(&Type::I8));
        assert!(filtered.contains(&Type::I16));
        assert!(filtered.contains(&Type::I32));
        assert!(filtered.contains(&Type::I64));
        assert!(filtered.contains(&Type::F32));
        assert!(filtered.contains(&Type::F64));
    }

    #[test]
    fn test_candidate_types_for_unary_bitnot() {
        let cs = candidate_types_for_value(5);
        let filtered = candidate_types_for_unary(&cs, UnaryOperator::BitNot);
        // BitNot works on any integer type (signed or unsigned)
        assert!(filtered.contains(&Type::U8));
        assert!(filtered.contains(&Type::I8));
        assert!(filtered.contains(&Type::U16));
        assert!(filtered.contains(&Type::I16));
        assert!(filtered.contains(&Type::U32));
        assert!(filtered.contains(&Type::I32));
        assert!(filtered.contains(&Type::U64));
        assert!(filtered.contains(&Type::I64));
        // Floats are not integers
        assert!(!filtered.contains(&Type::F32));
        assert!(!filtered.contains(&Type::F64));
    }

    #[test]
    fn test_candidate_types_for_unary_logical_not() {
        let cs = candidate_types_for_value(42);
        let filtered = candidate_types_for_unary(&cs, UnaryOperator::Not);
        // Logical NOT always produces Bool
        assert_eq!(filtered, CandidateSet(vec![Type::Bool]));
    }

    #[test]
    fn test_find_common_type_sets_signed_unsigned() {
        // -42 candidates ∩ U8 candidates → I16 (smallest common)
        let signed = candidate_types_for_value(-42);
        let unsigned = CandidateSet(vec![Type::U8]);
        let common = find_common_type_sets(&signed, &unsigned);
        assert_eq!(common, Some(Type::I16));
    }

    #[test]
    fn test_find_common_type_sets_no_common() {
        // F32 and U64 have no common type (U64 cannot cast to F32, F32 cannot cast to U64)
        let float = CandidateSet(vec![Type::F32]);
        let big = CandidateSet(vec![Type::U64]);
        let common = find_common_type_sets(&float, &big);
        assert!(common.is_none());
    }

    #[test]
    fn test_find_common_type_sets_small_signed_unsigned() {
        // I8 and U8 have common type I16 (I8→I16 OK, U8→I16 OK)
        let signed = CandidateSet(vec![Type::I8]);
        let unsigned = CandidateSet(vec![Type::U8]);
        let common = find_common_type_sets(&signed, &unsigned);
        assert_eq!(common, Some(Type::I16));
    }

    #[test]
    fn test_is_cast_lossless_value() {
        use crate::machine::{FloatFmt, FloatValue, IntegerFmt, IntegerValue, StringFmt, StringValue};
        // 42 fits in U8
        assert!(is_cast_lossless_value(
            &Value::Integer(IntegerValue { value: 42, fmt: IntegerFmt::Dec }),
            &Type::U8
        ));
        // -42 does NOT fit in U8
        assert!(!is_cast_lossless_value(
            &Value::Integer(IntegerValue { value: -42, fmt: IntegerFmt::Dec }),
            &Type::U8
        ));
        // -42 fits in I8
        assert!(is_cast_lossless_value(
            &Value::Integer(IntegerValue { value: -42, fmt: IntegerFmt::Dec }),
            &Type::I8
        ));
        // Float 3.14 fits in F64
        assert!(is_cast_lossless_value(
            &Value::Float(FloatValue { value: 3.14, fmt: FloatFmt::Decimal }),
            &Type::F64
        ));
        // Float does NOT fit in integer
        assert!(!is_cast_lossless_value(
            &Value::Float(FloatValue { value: 3.14, fmt: FloatFmt::Decimal }),
            &Type::U8
        ));
        // Bool fits in numeric
        assert!(is_cast_lossless_value(
            &Value::Bool(true),
            &Type::U8
        ));
    }

    #[test]
    fn test_value_default_type() {
        use crate::machine::{FloatFmt, FloatValue, IntegerFmt, IntegerValue, StringFmt, StringValue};
        assert_eq!(value_default_type(&Value::Integer(IntegerValue { value: 42, fmt: IntegerFmt::Dec })), Type::I64);
        assert_eq!(value_default_type(&Value::Float(FloatValue { value: 3.14, fmt: FloatFmt::Decimal })), Type::F64);
        assert_eq!(value_default_type(&Value::Bool(true)), Type::Bool);
        assert_eq!(value_default_type(&Value::String(StringValue { value: "hello".into(), fmt: StringFmt::DoubleQuote })), Type::String);
    }

    #[test]
    fn test_int_to_int_lossless_match() {
        // Unsigned → larger unsigned
        assert!(int_to_int_lossless(&Type::U8, &Type::U16));
        assert!(int_to_int_lossless(&Type::U8, &Type::U32));
        assert!(int_to_int_lossless(&Type::U16, &Type::U32));
        assert!(int_to_int_lossless(&Type::U32, &Type::U64));
        // Unsigned → larger signed
        assert!(int_to_int_lossless(&Type::U8, &Type::I16));
        assert!(int_to_int_lossless(&Type::U8, &Type::I32));
        assert!(int_to_int_lossless(&Type::U16, &Type::I32));
        assert!(int_to_int_lossless(&Type::U32, &Type::I64));
        // Signed → larger signed
        assert!(int_to_int_lossless(&Type::I8, &Type::I16));
        assert!(int_to_int_lossless(&Type::I8, &Type::I32));
        assert!(int_to_int_lossless(&Type::I32, &Type::I64));
        // Never signed → unsigned
        assert!(!int_to_int_lossless(&Type::I8, &Type::U8));
        assert!(!int_to_int_lossless(&Type::I32, &Type::U32));
        assert!(!int_to_int_lossless(&Type::I64, &Type::U64));
    }
}
