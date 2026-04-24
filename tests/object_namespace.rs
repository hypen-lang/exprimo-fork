//! Edge case coverage for the Object namespace methods (keys/values/entries).
//!
//! Note: serde_json::Map is BTreeMap-backed (alphabetical), so iteration is
//! always sorted. Tests rely on that deterministic ordering.

use exprimo::{CustomFuncError, EvaluationError, Evaluator};
use serde_json::Value;
use std::collections::HashMap;

fn ev() -> Evaluator {
    Evaluator::new(HashMap::new(), HashMap::new())
}

fn ev_with_obj(name: &str, pairs: &[(&str, Value)]) -> Evaluator {
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    for (k, v) in pairs {
        m.insert(k.to_string(), v.clone());
    }
    ctx.insert(name.to_string(), Value::Object(m));
    Evaluator::new(ctx, HashMap::new())
}

fn num(v: f64) -> Value {
    Value::Number(serde_json::Number::from_f64(v).unwrap())
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

// --- Object.keys ---

#[test]
fn keys_empty_object_from_literal() {
    assert_eq!(
        ev().evaluate("Object.keys({})").unwrap(),
        Value::Array(vec![])
    );
}

#[test]
fn keys_empty_object_from_context() {
    let ev = ev_with_obj("o", &[]);
    assert_eq!(
        ev.evaluate("Object.keys(o)").unwrap(),
        Value::Array(vec![])
    );
}

#[test]
fn keys_single_pair() {
    assert_eq!(
        ev().evaluate("Object.keys({a: 1})").unwrap(),
        Value::Array(vec![s("a")])
    );
}

#[test]
fn keys_multiple_pairs_sorted_alphabetically() {
    // BTreeMap backing sorts keys alphabetically regardless of insertion order.
    assert_eq!(
        ev().evaluate("Object.keys({b: 1, a: 2, c: 3})").unwrap(),
        Value::Array(vec![s("a"), s("b"), s("c")])
    );
}

#[test]
fn keys_with_string_keys() {
    assert_eq!(
        ev().evaluate("Object.keys({'hello world': 1, 'foo': 2})").unwrap(),
        Value::Array(vec![s("foo"), s("hello world")])
    );
}

#[test]
fn keys_with_empty_string_key() {
    let ev = ev_with_obj("o", &[("", s("val"))]);
    assert_eq!(
        ev.evaluate("Object.keys(o)").unwrap(),
        Value::Array(vec![s("")])
    );
}

#[test]
fn keys_on_nested_object_only_returns_top_level() {
    let ev = ev_with_obj(
        "o",
        &[
            ("a", num(1.0)),
            ("nested", {
                let mut m = serde_json::Map::new();
                m.insert("inner".to_string(), num(2.0));
                Value::Object(m)
            }),
        ],
    );
    assert_eq!(
        ev.evaluate("Object.keys(o)").unwrap(),
        Value::Array(vec![s("a"), s("nested")])
    );
}

#[test]
fn keys_on_non_object_errors() {
    for expr in &[
        "Object.keys('hello')",
        "Object.keys(42)",
        "Object.keys(true)",
        "Object.keys(null)",
        "Object.keys([1, 2, 3])",
    ] {
        match ev().evaluate(expr) {
            Err(EvaluationError::TypeError(msg)) => {
                assert!(msg.contains("non-object"), "{} → {}", expr, msg);
            }
            other => panic!("Expected TypeError for {}, got {:?}", expr, other),
        }
    }
}

#[test]
fn keys_arity_error_no_args() {
    match ev().evaluate("Object.keys()") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected: 1, got: 0 })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

#[test]
fn keys_arity_error_too_many_args() {
    match ev().evaluate("Object.keys({a: 1}, {b: 2})") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected: 1, got: 2 })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// --- Object.values ---

#[test]
fn values_empty() {
    assert_eq!(
        ev().evaluate("Object.values({})").unwrap(),
        Value::Array(vec![])
    );
}

#[test]
fn values_preserves_types() {
    // values ordered by their keys (alphabetical).
    let result = ev()
        .evaluate("Object.values({c: true, a: 1, b: 'x'})")
        .unwrap();
    assert_eq!(
        result,
        Value::Array(vec![num(1.0), s("x"), Value::Bool(true)])
    );
}

#[test]
fn values_sorted_by_keys_alphabetically() {
    // Values come back in the alphabetical order of their keys, NOT insertion order.
    let result = ev().evaluate("Object.values({z: 1, a: 2, m: 3})").unwrap();
    assert_eq!(result, Value::Array(vec![num(2.0), num(3.0), num(1.0)]));
}

#[test]
fn values_with_null_and_nested() {
    let ev = ev_with_obj(
        "o",
        &[
            ("a", Value::Null),
            ("b", Value::Array(vec![num(1.0), num(2.0)])),
            ("c", {
                let mut m = serde_json::Map::new();
                m.insert("x".to_string(), num(10.0));
                Value::Object(m)
            }),
        ],
    );
    let result = ev.evaluate("Object.values(o)").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Null,
            Value::Array(vec![num(1.0), num(2.0)]),
            {
                let mut m = serde_json::Map::new();
                m.insert("x".to_string(), num(10.0));
                Value::Object(m)
            }
        ])
    );
}

#[test]
fn values_on_non_object_errors() {
    match ev().evaluate("Object.values('hello')") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("non-object"));
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn values_arity_error() {
    match ev().evaluate("Object.values()") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// --- Object.entries ---

#[test]
fn entries_empty() {
    assert_eq!(
        ev().evaluate("Object.entries({})").unwrap(),
        Value::Array(vec![])
    );
}

#[test]
fn entries_structure_key_value_pairs() {
    let result = ev().evaluate("Object.entries({a: 1, b: 'x'})").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![s("a"), num(1.0)]),
            Value::Array(vec![s("b"), s("x")]),
        ])
    );
}

#[test]
fn entries_first_element_is_key_second_is_value() {
    // Spot-check individual entry shape via indexing.
    assert_eq!(
        ev().evaluate("Object.entries({foo: 42})[0][0]").unwrap(),
        s("foo")
    );
    assert_eq!(
        ev().evaluate("Object.entries({foo: 42})[0][1]").unwrap(),
        num(42.0)
    );
}

#[test]
fn entries_on_non_object_errors() {
    match ev().evaluate("Object.entries(42)") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("non-object"));
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn entries_length_matches_keys() {
    let ev = ev_with_obj(
        "o",
        &[("a", num(1.0)), ("b", num(2.0)), ("c", num(3.0)), ("d", num(4.0))],
    );
    assert_eq!(ev.evaluate("Object.entries(o).length").unwrap(), num(4.0));
}

// --- Object namespace misc ---

#[test]
fn object_unknown_method_errors() {
    match ev().evaluate("Object.freeze({a: 1})") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("Object.freeze"), "got: {}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn object_accessing_property_without_call_errors() {
    match ev().evaluate("Object.foo") {
        Err(EvaluationError::TypeError(_)) => {}
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn object_shadowed_by_context() {
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    m.insert("keys".to_string(), s("shadowed"));
    ctx.insert("Object".to_string(), Value::Object(m));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(ev.evaluate("Object.keys").unwrap(), s("shadowed"));
}

// --- Combinations with other features ---

#[test]
fn keys_combined_with_length() {
    let ev = ev_with_obj("o", &[("a", num(1.0)), ("b", num(2.0))]);
    assert_eq!(ev.evaluate("Object.keys(o).length").unwrap(), num(2.0));
}

#[test]
fn keys_combined_with_includes() {
    let ev = ev_with_obj("o", &[("a", num(1.0)), ("b", num(2.0))]);
    assert_eq!(
        ev.evaluate("Object.keys(o).includes('a')").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev.evaluate("Object.keys(o).includes('missing')").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn keys_combined_with_join() {
    let ev = ev_with_obj("o", &[("a", num(1.0)), ("b", num(2.0)), ("c", num(3.0))]);
    assert_eq!(ev.evaluate("Object.keys(o).join(', ')").unwrap(), s("a, b, c"));
}

#[test]
fn keys_with_bracket_indexing() {
    let ev = ev_with_obj("o", &[("first", num(1.0)), ("second", num(2.0))]);
    // Object.keys is alphabetical: ["first", "second"][0] === "first"
    assert_eq!(ev.evaluate("Object.keys(o)[0]").unwrap(), s("first"));
}

#[test]
fn values_combined_with_indexOf() {
    let ev = ev_with_obj("o", &[("a", num(10.0)), ("b", num(20.0)), ("c", num(30.0))]);
    // IDs are alphabetical by key, so values come back [10, 20, 30].
    assert_eq!(ev.evaluate("Object.values(o).indexOf(20)").unwrap(), num(1.0));
}

#[test]
fn entries_combined_with_length_and_slice() {
    let ev = ev_with_obj("o", &[("a", num(1.0)), ("b", num(2.0)), ("c", num(3.0))]);
    assert_eq!(ev.evaluate("Object.entries(o).slice(0, 1).length").unwrap(), num(1.0));
}

#[test]
fn object_methods_on_inline_literal() {
    // Passing an inline object literal with computed values.
    let mut ctx = HashMap::new();
    ctx.insert("x".to_string(), num(5.0));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(
        ev.evaluate("Object.keys({p: x + 1, q: x * 2})").unwrap(),
        Value::Array(vec![s("p"), s("q")])
    );
    assert_eq!(
        ev.evaluate("Object.values({p: x + 1, q: x * 2})").unwrap(),
        Value::Array(vec![num(6.0), num(10.0)])
    );
}
