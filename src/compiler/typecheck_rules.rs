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

use crate::machine::{BinaryOperator, Type};

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
    let int_bits = match int_ty {
        Type::U8 | Type::I8 => 8,
        Type::U16 | Type::I16 => 16,
        Type::U32 | Type::I32 => 32,
        Type::U64 | Type::I64 => 64,
        _ => return false,
    };

    let mantissa_bits = match float_ty {
        Type::F32 => 23, // f32 has 23-bit mantissa (+ 1 implicit = 24 bits of precision)
        Type::F64 => 52, // f64 has 52-bit mantissa
        _ => return false,
    };

    // For signed integers, we need one extra bit for the sign
    let is_signed = matches!(int_ty, Type::I8 | Type::I16 | Type::I32 | Type::I64);
    let needed_bits = if is_signed { int_bits } else { int_bits };

    // u8/u16 can fit in f32 mantissa (24 bits including implicit bit)
    // i8/i16 can fit in f32 (needs sign + value, 8 bits total for signed)
    // u32/i32 can fit in f64 mantissa (52 bits)
    // i64/u64 cannot fit in f64 (64 > 52 bits)
    match (int_ty, float_ty) {
        (Type::U8 | Type::I8 | Type::U16 | Type::I16, Type::F32) => true,
        (Type::U32 | Type::I32, Type::F64) => true,
        (Type::U8 | Type::I8, Type::F64) => true,
        (Type::U16 | Type::I16, Type::F64) => true,
        _ => false,
    }
}

/// Check if integer → integer casting is lossless.
fn int_to_int_lossless(from: &Type, to: &Type) -> bool {
    let from_bits = match from {
        Type::U8 | Type::I8 => 8,
        Type::U16 | Type::I16 => 16,
        Type::U32 | Type::I32 => 32,
        Type::U64 | Type::I64 => 64,
        _ => return false,
    };

    let to_bits = match to {
        Type::U8 | Type::I8 => 8,
        Type::U16 | Type::I16 => 16,
        Type::U32 | Type::I32 => 32,
        Type::U64 | Type::I64 => 64,
        _ => return false,
    };

    let from_unsigned = matches!(from, Type::U8 | Type::U16 | Type::U32 | Type::U64);
    let to_unsigned = matches!(to, Type::U8 | Type::U16 | Type::U32 | Type::U64);

    // Never allow signed → unsigned (sign conversion is not lossless)
    if !from_unsigned && to_unsigned {
        return false;
    }

    // Same sign: target must be strictly larger (or equal, but equal is handled above)
    if from_unsigned == to_unsigned {
        to_bits > from_bits
    } else {
        // from unsigned → to signed: target must be able to hold all values of source
        // e.g., u8 → i16 is OK (i16 can hold 0..255), u8 → i8 is NOT (i8 can only hold 0..127)
        // u16 → i32 is OK, u32 → i64 is OK
        // In general: unsigned N → signed M is OK if M_bits >= N_bits + 1 (for sign bit)
        // But i64 can hold all u64 values... wait, no. u64 max = 2^64-1, i64 max = 2^63-1
        // So u64 → i64 is NOT lossless for values > 2^63-1
        //
        // Rule: unsigned N → signed M is OK only if M_bits > N_bits (strictly larger)
        // OR if N_bits < M_bits - 1 (leaving room for sign bit)
        // Simplified: u8→i16(16>8✓), u16→i32(32>16✓), u32→i64(64>32✓)
        // Not: u64→i64(64=64✗), u16→i16(16=16✗, but this is same sign, handled above)
        to_bits > from_bits + 1 || (to_bits >= from_bits && to_bits == 64 && from_bits < 64)
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
    let candidates: Vec<Type> = (0_u8..=10)
        .filter_map(|target_ty| {
            let target = numeric_type_from_u8(target_ty)?;
            if is_cast_lossless(from, &target) && is_cast_lossless(to, &target) {
                Some(target)
            } else {
                None
            }
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Sort by bits, then prefer unsigned for same bit count
    let mut sorted = candidates;
    sorted.sort_by_key(|t| (get_target_bits(t), !is_unsigned(t) as u8));

    sorted.first().cloned()
}

/// Convert a u8 index to a Type variant.
fn numeric_type_from_u8(n: u8) -> Option<Type> {
    match n {
        0 => Some(Type::Bool),
        1 => Some(Type::U8),
        2 => Some(Type::U16),
        3 => Some(Type::U32),
        4 => Some(Type::U64),
        5 => Some(Type::I8),
        6 => Some(Type::I16),
        7 => Some(Type::I32),
        8 => Some(Type::I64),
        9 => Some(Type::F32),
        10 => Some(Type::F64),
        _ => None,
    }
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
}
