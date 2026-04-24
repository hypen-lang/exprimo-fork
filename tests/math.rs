//! Edge case coverage for the Math namespace and Number.toFixed.

use exprimo::{CustomFuncError, EvaluationError, Evaluator};
use serde_json::Value;
use std::collections::HashMap;

fn ev() -> Evaluator {
    Evaluator::new(HashMap::new(), HashMap::new())
}

fn num(v: f64) -> Value {
    Value::Number(serde_json::Number::from_f64(v).unwrap())
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

// --- Math.floor ---

#[test]
fn floor_positive() {
    assert_eq!(ev().evaluate("Math.floor(1.9)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.floor(1.0)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.floor(1.1)").unwrap(), num(1.0));
}

#[test]
fn floor_zero() {
    assert_eq!(ev().evaluate("Math.floor(0)").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("Math.floor(0.0)").unwrap(), num(0.0));
}

#[test]
fn floor_negative() {
    assert_eq!(ev().evaluate("Math.floor(-1.1)").unwrap(), num(-2.0));
    assert_eq!(ev().evaluate("Math.floor(-0.5)").unwrap(), num(-1.0));
    assert_eq!(ev().evaluate("Math.floor(-1.0)").unwrap(), num(-1.0));
}

#[test]
fn floor_coerces_string_arg() {
    // JS coerces string to number.
    assert_eq!(ev().evaluate("Math.floor('2.7')").unwrap(), num(2.0));
}

#[test]
fn floor_arity_error() {
    match ev().evaluate("Math.floor()") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected: 1, got: 0 })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
    match ev().evaluate("Math.floor(1, 2)") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected: 1, got: 2 })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// --- Math.ceil ---

#[test]
fn ceil_positive() {
    assert_eq!(ev().evaluate("Math.ceil(1.1)").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("Math.ceil(1.0)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.ceil(1.9)").unwrap(), num(2.0));
}

#[test]
fn ceil_zero() {
    assert_eq!(ev().evaluate("Math.ceil(0)").unwrap(), num(0.0));
}

#[test]
fn ceil_negative() {
    assert_eq!(ev().evaluate("Math.ceil(-1.9)").unwrap(), num(-1.0));
    assert_eq!(ev().evaluate("Math.ceil(-0.5)").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("Math.ceil(-1.0)").unwrap(), num(-1.0));
}

// --- Math.round ---

#[test]
fn round_halves_to_plus_infinity() {
    // JS: Math.round(0.5) === 1, Math.round(-0.5) === 0 (half toward +Infinity)
    assert_eq!(ev().evaluate("Math.round(0.5)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.round(1.5)").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("Math.round(-0.5)").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("Math.round(-1.5)").unwrap(), num(-1.0));
}

#[test]
fn round_below_and_above_half() {
    assert_eq!(ev().evaluate("Math.round(1.4)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.round(1.6)").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("Math.round(-1.4)").unwrap(), num(-1.0));
    assert_eq!(ev().evaluate("Math.round(-1.6)").unwrap(), num(-2.0));
}

#[test]
fn round_integer_unchanged() {
    assert_eq!(ev().evaluate("Math.round(5)").unwrap(), num(5.0));
    assert_eq!(ev().evaluate("Math.round(-5)").unwrap(), num(-5.0));
    assert_eq!(ev().evaluate("Math.round(0)").unwrap(), num(0.0));
}

// --- Math.abs ---

#[test]
fn abs_basic() {
    assert_eq!(ev().evaluate("Math.abs(5)").unwrap(), num(5.0));
    assert_eq!(ev().evaluate("Math.abs(-5)").unwrap(), num(5.0));
    assert_eq!(ev().evaluate("Math.abs(0)").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("Math.abs(-0)").unwrap(), num(0.0));
}

#[test]
fn abs_fractions() {
    assert_eq!(ev().evaluate("Math.abs(-1.5)").unwrap(), num(1.5));
    assert_eq!(ev().evaluate("Math.abs(1.5)").unwrap(), num(1.5));
}

#[test]
fn abs_coerces_string() {
    assert_eq!(ev().evaluate("Math.abs('-7')").unwrap(), num(7.0));
}

// --- Math.min / Math.max ---

#[test]
fn min_two_args() {
    assert_eq!(ev().evaluate("Math.min(1, 2)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.min(-1, -2)").unwrap(), num(-2.0));
}

#[test]
fn min_many_args() {
    assert_eq!(ev().evaluate("Math.min(5, 3, 8, 1, 4)").unwrap(), num(1.0));
}

#[test]
fn min_single_arg() {
    assert_eq!(ev().evaluate("Math.min(42)").unwrap(), num(42.0));
}

#[test]
fn min_no_args_returns_infinity() {
    // JS Math.min() === Infinity. exprimo represents Infinity as f64::MAX.
    match ev().evaluate("Math.min()").unwrap() {
        Value::Number(n) => {
            let v = n.as_f64().unwrap();
            assert!(v > 1e300, "expected very large number, got {}", v);
        }
        other => panic!("expected Number, got {:?}", other),
    }
}

#[test]
fn min_coerces_string() {
    assert_eq!(ev().evaluate("Math.min('5', 2)").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("Math.min('5', '2')").unwrap(), num(2.0));
}

#[test]
fn max_two_args() {
    assert_eq!(ev().evaluate("Math.max(1, 2)").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("Math.max(-1, -2)").unwrap(), num(-1.0));
}

#[test]
fn max_many_args() {
    assert_eq!(ev().evaluate("Math.max(5, 3, 8, 1, 4)").unwrap(), num(8.0));
}

#[test]
fn max_single_arg() {
    assert_eq!(ev().evaluate("Math.max(42)").unwrap(), num(42.0));
}

#[test]
fn max_no_args_returns_negative_infinity() {
    match ev().evaluate("Math.max()").unwrap() {
        Value::Number(n) => {
            let v = n.as_f64().unwrap();
            assert!(v < -1e300, "expected very negative number, got {}", v);
        }
        other => panic!("expected Number, got {:?}", other),
    }
}

#[test]
fn min_max_with_mixed_fractions() {
    assert_eq!(ev().evaluate("Math.min(1.5, 1.4)").unwrap(), num(1.4));
    assert_eq!(ev().evaluate("Math.max(1.5, 1.4)").unwrap(), num(1.5));
}

// --- Math nesting / combinations ---

#[test]
fn math_nested_calls() {
    assert_eq!(ev().evaluate("Math.abs(Math.floor(-1.5))").unwrap(), num(2.0));
    assert_eq!(
        ev().evaluate("Math.min(Math.abs(-5), Math.abs(3))").unwrap(),
        num(3.0)
    );
    assert_eq!(
        ev().evaluate("Math.max(Math.floor(1.7), Math.ceil(2.3))").unwrap(),
        num(3.0)
    );
}

#[test]
fn math_with_arithmetic_args() {
    assert_eq!(ev().evaluate("Math.floor(5 / 2)").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("Math.ceil(5 / 2)").unwrap(), num(3.0));
    assert_eq!(ev().evaluate("Math.abs(3 - 10)").unwrap(), num(7.0));
}

#[test]
fn math_in_ternary() {
    assert_eq!(
        ev().evaluate("Math.abs(-5) > 3 ? 'big' : 'small'").unwrap(),
        s("big")
    );
}

#[test]
fn math_with_context() {
    let mut ctx = HashMap::new();
    ctx.insert("x".to_string(), num(-3.7));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(ev.evaluate("Math.floor(x)").unwrap(), num(-4.0));
    assert_eq!(ev.evaluate("Math.abs(x)").unwrap(), num(3.7));
}

#[test]
fn math_unknown_method() {
    match ev().evaluate("Math.unknown(1)") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("Math.unknown"), "got: {}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn math_not_a_method_call_returns_namespace_error() {
    // Accessing Math.floor without calling it returns the unknown method error
    // because Math.<anything> always resolves as a method.
    match ev().evaluate("Math.foo") {
        Err(EvaluationError::TypeError(_)) => {}
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn math_shadowed_uses_context_value() {
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    m.insert("floor".to_string(), Value::String("custom".to_string()));
    ctx.insert("Math".to_string(), Value::Object(m));
    let ev = Evaluator::new(ctx, HashMap::new());
    // Now Math.floor looks up the key on the context object.
    assert_eq!(ev.evaluate("Math.floor").unwrap(), s("custom"));
}

// --- Number.toFixed ---

#[test]
fn to_fixed_basic() {
    let ev = ev();
    assert_eq!(ev.evaluate("(3.14).toFixed(1)").unwrap(), s("3.1"));
    assert_eq!(ev.evaluate("(3.14).toFixed(2)").unwrap(), s("3.14"));
    assert_eq!(ev.evaluate("(3.14).toFixed(3)").unwrap(), s("3.140"));
}

#[test]
fn to_fixed_default_zero_digits() {
    assert_eq!(ev().evaluate("(3.7).toFixed()").unwrap(), s("4"));
    assert_eq!(ev().evaluate("(3.4).toFixed()").unwrap(), s("3"));
}

#[test]
fn to_fixed_zero() {
    assert_eq!(ev().evaluate("(0).toFixed(2)").unwrap(), s("0.00"));
    assert_eq!(ev().evaluate("(-0).toFixed(2)").unwrap(), s("0.00"));
}

#[test]
fn to_fixed_integers_get_trailing_zeros() {
    assert_eq!(ev().evaluate("(5).toFixed(2)").unwrap(), s("5.00"));
    assert_eq!(ev().evaluate("(100).toFixed(3)").unwrap(), s("100.000"));
}

#[test]
fn to_fixed_negative_numbers() {
    assert_eq!(ev().evaluate("(-1.234).toFixed(2)").unwrap(), s("-1.23"));
    // Rust uses round-half-to-even, so -0.5 rounds to -0 which we then
    // normalize to "0" (matching `(-0.5).toFixed(0) === "0"` in V8).
    assert_eq!(ev().evaluate("(-0.5).toFixed(0)").unwrap(), s("0"));
    assert_eq!(ev().evaluate("(-1.5).toFixed(0)").unwrap(), s("-2"));
    assert_eq!(ev().evaluate("(-99.99).toFixed(1)").unwrap(), s("-100.0"));
}

#[test]
fn to_fixed_rejects_negative_digits() {
    match ev().evaluate("(1).toFixed(-1)") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("between 0 and 100"), "got: {}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn to_fixed_rejects_digits_over_100() {
    match ev().evaluate("(1).toFixed(101)") {
        Err(EvaluationError::TypeError(_)) => {}
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn to_fixed_accepts_boundary_digits() {
    assert_eq!(ev().evaluate("(1).toFixed(0)").unwrap(), s("1"));
    // digit count of 100 should be accepted
    let r = ev().evaluate("(1).toFixed(100)").unwrap();
    match r {
        Value::String(out) => {
            assert!(out.starts_with("1."), "{}", out);
            assert_eq!(out.len(), 1 + 1 + 100);
        }
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn to_fixed_too_many_args() {
    match ev().evaluate("(1).toFixed(1, 2)") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

#[test]
fn to_fixed_truncates_float_digits_to_int() {
    // JS truncates the digits arg toward zero: (1).toFixed(2.9) === "1.00"
    assert_eq!(ev().evaluate("(1).toFixed(2.9)").unwrap(), s("1.00"));
}

#[test]
fn to_fixed_on_context_number() {
    let mut ctx = HashMap::new();
    ctx.insert("price".to_string(), num(19.995));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(ev.evaluate("price.toFixed(2)").unwrap(), s("20.00"));
}

#[test]
fn to_fixed_in_concatenation() {
    let mut ctx = HashMap::new();
    ctx.insert("price".to_string(), num(9.5));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(
        ev.evaluate("'$' + price.toFixed(2)").unwrap(),
        s("$9.50")
    );
}

// =============================================================================
// Math with coerced non-number arguments
// =============================================================================

#[test]
fn math_floor_coerces_null_to_zero() {
    assert_eq!(ev().evaluate("Math.floor(null)").unwrap(), num(0.0));
}

#[test]
fn math_floor_coerces_booleans() {
    assert_eq!(ev().evaluate("Math.floor(true)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.floor(false)").unwrap(), num(0.0));
}

#[test]
fn math_ceil_coerces_booleans() {
    assert_eq!(ev().evaluate("Math.ceil(true)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.ceil(false)").unwrap(), num(0.0));
}

#[test]
fn math_round_coerces_null_and_booleans() {
    assert_eq!(ev().evaluate("Math.round(null)").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("Math.round(true)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.round(false)").unwrap(), num(0.0));
}

#[test]
fn math_abs_coerces_null_and_booleans() {
    assert_eq!(ev().evaluate("Math.abs(null)").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("Math.abs(true)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math.abs(false)").unwrap(), num(0.0));
}

#[test]
fn math_floor_of_non_numeric_string_yields_null() {
    // 'abc' → NaN → floor(NaN) returns Null via f64_to_value.
    assert_eq!(ev().evaluate("Math.floor('abc')").unwrap(), Value::Null);
    assert_eq!(ev().evaluate("Math.abs('xyz')").unwrap(), Value::Null);
}

#[test]
fn math_min_mixed_coercion() {
    // null→0, true→1, false→0, '3'→3. Minimum of {0, 1, 0, 3} is 0.
    assert_eq!(
        ev().evaluate("Math.min(null, true, false, '3')").unwrap(),
        num(0.0)
    );
}

#[test]
fn math_max_mixed_coercion() {
    // {0, 1, 0, 2} → max 2
    assert_eq!(
        ev().evaluate("Math.max(null, true, false, 2)").unwrap(),
        num(2.0)
    );
}

#[test]
fn math_min_max_with_boolean_only() {
    assert_eq!(ev().evaluate("Math.min(true, false)").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("Math.max(true, false)").unwrap(), num(1.0));
}

#[test]
fn math_min_with_unparseable_string_propagates_nan() {
    // NaN from 'abc' → min returns Null (f64_to_value converts NaN to Null).
    assert_eq!(ev().evaluate("Math.min('abc', 1)").unwrap(), Value::Null);
    assert_eq!(ev().evaluate("Math.max('abc', 1)").unwrap(), Value::Null);
}

// =============================================================================
// Math with Infinity arguments
// =============================================================================
//
// exprimo represents Infinity as f64::MAX because serde_json::Number cannot
// store actual f64::INFINITY. Thus Math.*(Infinity) returns the same sentinel.

fn is_positive_infinity_sentinel(v: Value) -> bool {
    match v {
        Value::Number(n) => n.as_f64().unwrap() > 1.0e300,
        _ => false,
    }
}

fn is_negative_infinity_sentinel(v: Value) -> bool {
    match v {
        Value::Number(n) => n.as_f64().unwrap() < -1.0e300,
        _ => false,
    }
}

#[test]
fn math_floor_of_infinity_returns_infinity_sentinel() {
    assert!(is_positive_infinity_sentinel(
        ev().evaluate("Math.floor(Infinity)").unwrap()
    ));
}

#[test]
fn math_ceil_of_infinity_returns_infinity_sentinel() {
    assert!(is_positive_infinity_sentinel(
        ev().evaluate("Math.ceil(Infinity)").unwrap()
    ));
}

#[test]
fn math_round_of_infinity_returns_infinity_sentinel() {
    assert!(is_positive_infinity_sentinel(
        ev().evaluate("Math.round(Infinity)").unwrap()
    ));
}

#[test]
fn math_abs_of_negative_infinity_returns_positive_infinity() {
    assert!(is_positive_infinity_sentinel(
        ev().evaluate("Math.abs(-Infinity)").unwrap()
    ));
}

#[test]
fn math_min_with_negative_infinity() {
    assert!(is_negative_infinity_sentinel(
        ev().evaluate("Math.min(-Infinity, 1)").unwrap()
    ));
}

#[test]
fn math_max_with_positive_infinity() {
    assert!(is_positive_infinity_sentinel(
        ev().evaluate("Math.max(Infinity, 1)").unwrap()
    ));
}

#[test]
fn math_with_very_large_numbers() {
    // Very large finite number floors/ceils to itself.
    let v = ev().evaluate("Math.floor(1e300)").unwrap();
    match v {
        Value::Number(n) => assert!(n.as_f64().unwrap() > 1e299),
        other => panic!("expected Number, got {:?}", other),
    }
}

// =============================================================================
// Number.toFixed with coerced digit arguments
// =============================================================================

#[test]
fn to_fixed_digit_arg_is_string() {
    // JS coerces the digit argument using ToIntegerOrInfinity.
    assert_eq!(ev().evaluate("(1).toFixed('2')").unwrap(), s("1.00"));
    assert_eq!(ev().evaluate("(3.14).toFixed('1')").unwrap(), s("3.1"));
}

#[test]
fn to_fixed_digit_arg_is_boolean() {
    // true → 1 digit, false → 0 digits
    assert_eq!(ev().evaluate("(1).toFixed(true)").unwrap(), s("1.0"));
    assert_eq!(ev().evaluate("(1).toFixed(false)").unwrap(), s("1"));
}

#[test]
fn to_fixed_digit_arg_is_null() {
    // null → 0 digits
    assert_eq!(ev().evaluate("(1.7).toFixed(null)").unwrap(), s("2"));
    assert_eq!(ev().evaluate("(3.14).toFixed(null)").unwrap(), s("3"));
}

#[test]
fn to_fixed_digit_arg_unparseable_string_becomes_zero() {
    // 'abc' → NaN → arg_to_int returns 0 (NaN short-circuit).
    assert_eq!(ev().evaluate("(3.14).toFixed('abc')").unwrap(), s("3"));
}

#[test]
fn to_fixed_on_infinity_produces_long_string() {
    // exprimo's Infinity is f64::MAX; toFixed(2) produces the literal expansion.
    let v = ev().evaluate("Infinity.toFixed(2)").unwrap();
    match v {
        Value::String(out) => {
            assert!(out.ends_with(".00"));
            // f64::MAX is ~309 digits before the decimal.
            assert!(out.len() > 300, "expected a very long number, got len={}", out.len());
        }
        other => panic!("expected String, got {:?}", other),
    }
}

// =============================================================================
// Math namespace edge cases
// =============================================================================

#[test]
fn math_nested_with_mixed_types() {
    // Math.floor(Math.abs(true) + Math.abs(null)) === 1
    assert_eq!(
        ev().evaluate("Math.floor(Math.abs(true) + Math.abs(null))").unwrap(),
        num(1.0)
    );
}

#[test]
fn math_deep_chained_in_expression() {
    // Math.min(Math.abs(-5), Math.ceil(2.3), Math.floor(10.9), 7) === 3
    assert_eq!(
        ev().evaluate("Math.min(Math.abs(-5), Math.ceil(2.3), Math.floor(10.9), 7)").unwrap(),
        num(3.0)
    );
}
