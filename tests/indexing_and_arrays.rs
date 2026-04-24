//! Edge case coverage for bracket indexing (arr[i], obj[k], str[i]) and
//! additional array methods (indexOf, join, slice).

use exprimo::{CustomFuncError, EvaluationError, Evaluator};
use serde_json::Value;
use std::collections::HashMap;

fn ev() -> Evaluator {
    Evaluator::new(HashMap::new(), HashMap::new())
}

fn ev_with(pairs: &[(&str, Value)]) -> Evaluator {
    let mut ctx = HashMap::new();
    for (k, v) in pairs {
        ctx.insert(k.to_string(), v.clone());
    }
    Evaluator::new(ctx, HashMap::new())
}

fn num(v: f64) -> Value {
    Value::Number(serde_json::Number::from_f64(v).unwrap())
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

// =============================================================================
// Bracket indexing — arrays
// =============================================================================

#[test]
fn array_index_zero() {
    let ev = ev_with(&[("a", Value::Array(vec![num(10.0), num(20.0), num(30.0)]))]);
    assert_eq!(ev.evaluate("a[0]").unwrap(), num(10.0));
}

#[test]
fn array_index_last() {
    let ev = ev_with(&[("a", Value::Array(vec![num(10.0), num(20.0), num(30.0)]))]);
    assert_eq!(ev.evaluate("a[2]").unwrap(), num(30.0));
}

#[test]
fn array_index_out_of_range_returns_null() {
    let ev = ev_with(&[("a", Value::Array(vec![num(10.0)]))]);
    assert_eq!(ev.evaluate("a[1]").unwrap(), Value::Null);
    assert_eq!(ev.evaluate("a[99]").unwrap(), Value::Null);
}

#[test]
fn array_negative_index_returns_null() {
    // JS: arr[-1] === undefined (not Python-style wrap).
    let ev = ev_with(&[("a", Value::Array(vec![num(10.0), num(20.0)]))]);
    assert_eq!(ev.evaluate("a[-1]").unwrap(), Value::Null);
    assert_eq!(ev.evaluate("a[-100]").unwrap(), Value::Null);
}

#[test]
fn array_non_integer_index_returns_null() {
    let ev = ev_with(&[("a", Value::Array(vec![num(10.0), num(20.0)]))]);
    // 0.5 isn't a valid array index.
    assert_eq!(ev.evaluate("a[0.5]").unwrap(), Value::Null);
    assert_eq!(ev.evaluate("a[1.7]").unwrap(), Value::Null);
}

#[test]
fn array_nan_index_returns_null() {
    // A non-numeric string coerces to NaN during index_value's to_number call,
    // and NaN indices return null.
    let ev = ev_with(&[("a", Value::Array(vec![num(10.0)]))]);
    assert_eq!(ev.evaluate("a['abc']").unwrap(), Value::Null);
}

#[test]
fn array_index_on_empty_returns_null() {
    let ev = ev_with(&[("a", Value::Array(vec![]))]);
    assert_eq!(ev.evaluate("a[0]").unwrap(), Value::Null);
}

#[test]
fn array_dynamic_index_from_context() {
    let ev = ev_with(&[
        ("a", Value::Array(vec![num(1.0), num(2.0), num(3.0)])),
        ("i", num(2.0)),
    ]);
    assert_eq!(ev.evaluate("a[i]").unwrap(), num(3.0));
}

#[test]
fn array_index_from_expression() {
    let ev = ev_with(&[("a", Value::Array(vec![num(10.0), num(20.0), num(30.0)]))]);
    assert_eq!(ev.evaluate("a[1 + 1]").unwrap(), num(30.0));
    assert_eq!(ev.evaluate("a[a.length - 1]").unwrap(), num(30.0));
}

#[test]
fn array_index_chained() {
    let ev = ev_with(&[(
        "nested",
        Value::Array(vec![
            Value::Array(vec![num(1.0), num(2.0)]),
            Value::Array(vec![num(3.0), num(4.0)]),
        ]),
    )]);
    assert_eq!(ev.evaluate("nested[0][0]").unwrap(), num(1.0));
    assert_eq!(ev.evaluate("nested[0][1]").unwrap(), num(2.0));
    assert_eq!(ev.evaluate("nested[1][1]").unwrap(), num(4.0));
}

#[test]
fn array_index_on_literal() {
    let ev = ev();
    assert_eq!(ev.evaluate("[10, 20, 30][1]").unwrap(), num(20.0));
}

// =============================================================================
// Bracket indexing — objects
// =============================================================================

#[test]
fn object_index_string_key() {
    let mut m = serde_json::Map::new();
    m.insert("name".to_string(), s("Alice"));
    m.insert("age".to_string(), num(30.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o['name']").unwrap(), s("Alice"));
    assert_eq!(ev.evaluate("o['age']").unwrap(), num(30.0));
}

#[test]
fn object_index_double_quoted_string() {
    let mut m = serde_json::Map::new();
    m.insert("key".to_string(), s("val"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o[\"key\"]").unwrap(), s("val"));
}

#[test]
fn object_index_missing_key_returns_null() {
    let mut m = serde_json::Map::new();
    m.insert("a".to_string(), num(1.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o['missing']").unwrap(), Value::Null);
}

#[test]
fn object_index_dynamic_key() {
    let mut m = serde_json::Map::new();
    m.insert("foo".to_string(), s("bar"));
    let ev = ev_with(&[("o", Value::Object(m)), ("k", s("foo"))]);
    assert_eq!(ev.evaluate("o[k]").unwrap(), s("bar"));
}

#[test]
fn object_index_with_computed_key() {
    let mut m = serde_json::Map::new();
    m.insert("prefix_123".to_string(), s("match"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    // '123' stringifies directly; verify dynamic key concatenation.
    assert_eq!(ev.evaluate("o['prefix_' + '123']").unwrap(), s("match"));
}

#[test]
fn object_index_coerces_non_string_key_to_string() {
    // null/true/bool keys coerce to "null"/"true"/etc. and look up those keys.
    let mut m = serde_json::Map::new();
    m.insert("null".to_string(), s("found-null"));
    m.insert("true".to_string(), s("found-true"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o[null]").unwrap(), s("found-null"));
    assert_eq!(ev.evaluate("o[true]").unwrap(), s("found-true"));
}

#[test]
fn object_index_on_empty_returns_null() {
    let ev = ev_with(&[("o", Value::Object(serde_json::Map::new()))]);
    assert_eq!(ev.evaluate("o['anything']").unwrap(), Value::Null);
}

#[test]
fn object_index_chained_with_dot() {
    // obj[key].nestedProp
    let mut inner = serde_json::Map::new();
    inner.insert("x".to_string(), num(42.0));
    let mut outer = serde_json::Map::new();
    outer.insert("inner".to_string(), Value::Object(inner));
    let ev = ev_with(&[("o", Value::Object(outer))]);
    assert_eq!(ev.evaluate("o['inner'].x").unwrap(), num(42.0));
    assert_eq!(ev.evaluate("o.inner['x']").unwrap(), num(42.0));
}

// =============================================================================
// Bracket indexing — strings
// =============================================================================

#[test]
fn string_char_index() {
    let ev = ev_with(&[("s", s("hello"))]);
    assert_eq!(ev.evaluate("s[0]").unwrap(), s("h"));
    assert_eq!(ev.evaluate("s[4]").unwrap(), s("o"));
}

#[test]
fn string_index_out_of_range_returns_null() {
    let ev = ev_with(&[("s", s("ab"))]);
    assert_eq!(ev.evaluate("s[2]").unwrap(), Value::Null);
    assert_eq!(ev.evaluate("s[99]").unwrap(), Value::Null);
}

#[test]
fn string_negative_index_returns_null() {
    let ev = ev_with(&[("s", s("ab"))]);
    assert_eq!(ev.evaluate("s[-1]").unwrap(), Value::Null);
}

#[test]
fn string_multibyte_indexing_is_char_based() {
    let ev = ev_with(&[("s", s("🎉abc"))]);
    assert_eq!(ev.evaluate("s[0]").unwrap(), s("🎉"));
    assert_eq!(ev.evaluate("s[1]").unwrap(), s("a"));
    assert_eq!(ev.evaluate("s[3]").unwrap(), s("c"));
}

#[test]
fn string_empty_index_returns_null() {
    let ev = ev_with(&[("s", s(""))]);
    assert_eq!(ev.evaluate("s[0]").unwrap(), Value::Null);
}

#[test]
fn string_index_on_literal() {
    assert_eq!(ev().evaluate("'hello'[1]").unwrap(), s("e"));
}

// =============================================================================
// Bracket indexing — errors on non-indexable
// =============================================================================

#[test]
fn index_on_null_errors() {
    let ev = ev_with(&[("n", Value::Null)]);
    match ev.evaluate("n[0]") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("non-indexable"), "{}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn index_on_number_errors() {
    let ev = ev_with(&[("n", num(42.0))]);
    match ev.evaluate("n[0]") {
        Err(EvaluationError::TypeError(_)) => {}
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn index_on_boolean_errors() {
    let ev = ev_with(&[("b", Value::Bool(true))]);
    match ev.evaluate("b[0]") {
        Err(EvaluationError::TypeError(_)) => {}
        other => panic!("expected TypeError, got {:?}", other),
    }
}

// =============================================================================
// Array.indexOf
// =============================================================================

#[test]
fn index_of_first_occurrence_only() {
    let ev = ev_with(&[("a", Value::Array(vec![s("x"), s("y"), s("x")]))]);
    // Returns first occurrence.
    assert_eq!(ev.evaluate("a.indexOf('x')").unwrap(), num(0.0));
}

#[test]
fn index_of_not_found_returns_minus_one() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0), num(2.0)]))]);
    assert_eq!(ev.evaluate("a.indexOf(99)").unwrap(), num(-1.0));
}

#[test]
fn index_of_empty_array() {
    let ev = ev_with(&[("a", Value::Array(vec![]))]);
    assert_eq!(ev.evaluate("a.indexOf('anything')").unwrap(), num(-1.0));
}

#[test]
fn index_of_strict_type_match() {
    // indexOf uses strict equality: number 1 !== string "1".
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0), num(2.0), num(3.0)]))]);
    assert_eq!(ev.evaluate("a.indexOf(2)").unwrap(), num(1.0));
    assert_eq!(ev.evaluate("a.indexOf('2')").unwrap(), num(-1.0));
}

#[test]
fn index_of_null() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![num(1.0), Value::Null, num(3.0)]),
    )]);
    assert_eq!(ev.evaluate("a.indexOf(null)").unwrap(), num(1.0));
}

#[test]
fn index_of_boolean() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![Value::Bool(true), Value::Bool(false)]),
    )]);
    assert_eq!(ev.evaluate("a.indexOf(true)").unwrap(), num(0.0));
    assert_eq!(ev.evaluate("a.indexOf(false)").unwrap(), num(1.0));
}

#[test]
fn index_of_requires_one_arg() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0)]))]);
    match ev.evaluate("a.indexOf()") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
    match ev.evaluate("a.indexOf(1, 2)") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// =============================================================================
// Array.join
// =============================================================================

#[test]
fn join_empty_array() {
    let ev = ev_with(&[("a", Value::Array(vec![]))]);
    assert_eq!(ev.evaluate("a.join(',')").unwrap(), s(""));
    assert_eq!(ev.evaluate("a.join()").unwrap(), s(""));
}

#[test]
fn join_single_element() {
    let ev = ev_with(&[("a", Value::Array(vec![s("x")]))]);
    assert_eq!(ev.evaluate("a.join(',')").unwrap(), s("x"));
    assert_eq!(ev.evaluate("a.join('--')").unwrap(), s("x"));
}

#[test]
fn join_default_separator_is_comma() {
    let ev = ev_with(&[("a", Value::Array(vec![s("a"), s("b"), s("c")]))]);
    assert_eq!(ev.evaluate("a.join()").unwrap(), s("a,b,c"));
}

#[test]
fn join_empty_separator_concatenates() {
    let ev = ev_with(&[("a", Value::Array(vec![s("a"), s("b"), s("c")]))]);
    assert_eq!(ev.evaluate("a.join('')").unwrap(), s("abc"));
}

#[test]
fn join_null_renders_as_empty_string() {
    // JS: [null, null].join(',') === ","
    let ev = ev_with(&[("a", Value::Array(vec![Value::Null, Value::Null]))]);
    assert_eq!(ev.evaluate("a.join(',')").unwrap(), s(","));
    let ev2 = ev_with(&[(
        "a",
        Value::Array(vec![num(1.0), Value::Null, num(3.0)]),
    )]);
    assert_eq!(ev2.evaluate("a.join(',')").unwrap(), s("1.0,,3.0"));
}

#[test]
fn join_mixed_types() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![s("x"), Value::Bool(true), Value::Number(1.into())]),
    )]);
    assert_eq!(ev.evaluate("a.join('-')").unwrap(), s("x-true-1"));
}

#[test]
fn join_with_integer_valued_number_context() {
    // Numbers from context that are integer-typed stringify cleanly.
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![
            Value::Number(10.into()),
            Value::Number(20.into()),
            Value::Number(30.into()),
        ]),
    )]);
    assert_eq!(ev.evaluate("a.join('-')").unwrap(), s("10-20-30"));
}

#[test]
fn join_too_many_args_errors() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0)]))]);
    match ev.evaluate("a.join(',', ';')") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

#[test]
fn join_coerces_separator_arg() {
    let ev = ev_with(&[("a", Value::Array(vec![s("a"), s("b")]))]);
    // non-string separators get stringified
    assert_eq!(ev.evaluate("a.join(null)").unwrap(), s("anullb"));
    assert_eq!(ev.evaluate("a.join(true)").unwrap(), s("atrueb"));
}

// =============================================================================
// Array.slice
// =============================================================================

#[test]
fn slice_no_args_copies() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0), num(2.0), num(3.0)]))]);
    assert_eq!(
        ev.evaluate("a.slice()").unwrap(),
        Value::Array(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn slice_zero_zero_returns_empty() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0), num(2.0), num(3.0)]))]);
    assert_eq!(ev.evaluate("a.slice(0, 0)").unwrap(), Value::Array(vec![]));
}

#[test]
fn slice_start_only_returns_tail() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![num(1.0), num(2.0), num(3.0), num(4.0)]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice(2)").unwrap(),
        Value::Array(vec![num(3.0), num(4.0)])
    );
}

#[test]
fn slice_negative_start() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![num(1.0), num(2.0), num(3.0), num(4.0)]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice(-2)").unwrap(),
        Value::Array(vec![num(3.0), num(4.0)])
    );
    assert_eq!(
        ev.evaluate("a.slice(-3, -1)").unwrap(),
        Value::Array(vec![num(2.0), num(3.0)])
    );
}

#[test]
fn slice_negative_beyond_len_clamps_to_start() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0), num(2.0), num(3.0)]))]);
    assert_eq!(
        ev.evaluate("a.slice(-100)").unwrap(),
        Value::Array(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn slice_end_clamps_to_len() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0), num(2.0), num(3.0)]))]);
    assert_eq!(
        ev.evaluate("a.slice(0, 100)").unwrap(),
        Value::Array(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn slice_end_before_start_returns_empty() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0), num(2.0), num(3.0)]))]);
    assert_eq!(ev.evaluate("a.slice(2, 1)").unwrap(), Value::Array(vec![]));
}

#[test]
fn slice_start_past_end() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0), num(2.0), num(3.0)]))]);
    assert_eq!(ev.evaluate("a.slice(100)").unwrap(), Value::Array(vec![]));
}

#[test]
fn slice_on_empty_array() {
    let ev = ev_with(&[("a", Value::Array(vec![]))]);
    assert_eq!(ev.evaluate("a.slice()").unwrap(), Value::Array(vec![]));
    assert_eq!(ev.evaluate("a.slice(0, 5)").unwrap(), Value::Array(vec![]));
    assert_eq!(ev.evaluate("a.slice(-1)").unwrap(), Value::Array(vec![]));
}

#[test]
fn slice_preserves_element_types() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![s("x"), Value::Bool(false), Value::Null, num(1.5)]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice(1, 3)").unwrap(),
        Value::Array(vec![Value::Bool(false), Value::Null])
    );
}

#[test]
fn slice_too_many_args_errors() {
    let ev = ev_with(&[("a", Value::Array(vec![num(1.0)]))]);
    match ev.evaluate("a.slice(0, 1, 2)") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// =============================================================================
// Combinations
// =============================================================================

#[test]
fn chained_methods() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![s("alpha"), s("beta"), s("gamma"), s("delta")]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice(0, 2).join(', ')").unwrap(),
        s("alpha, beta")
    );
    assert_eq!(ev.evaluate("a.slice(1).length").unwrap(), num(3.0));
    assert_eq!(
        ev.evaluate("a.slice(0, 2).includes('beta')").unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn indexOf_with_slice() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![s("a"), s("b"), s("c"), s("d")]),
    )]);
    // after slicing, indexOf restarts from 0
    assert_eq!(ev.evaluate("a.slice(1).indexOf('b')").unwrap(), num(0.0));
    assert_eq!(ev.evaluate("a.slice(1).indexOf('a')").unwrap(), num(-1.0));
}

#[test]
fn index_into_object_values_array() {
    let mut m = serde_json::Map::new();
    m.insert("a".to_string(), num(100.0));
    m.insert("b".to_string(), num(200.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("Object.values(o)[1]").unwrap(), num(200.0));
}

#[test]
fn string_slice_result_is_indexable() {
    assert_eq!(ev().evaluate("'hello world'.slice(6)[0]").unwrap(), s("w"));
}

#[test]
fn dynamic_access_pattern_item_at_index() {
    // state.items[i].name — the canonical DSL pattern from the survey.
    let mut item0 = serde_json::Map::new();
    item0.insert("name".to_string(), s("alice"));
    let mut item1 = serde_json::Map::new();
    item1.insert("name".to_string(), s("bob"));
    let state = {
        let mut m = serde_json::Map::new();
        m.insert(
            "items".to_string(),
            Value::Array(vec![Value::Object(item0), Value::Object(item1)]),
        );
        Value::Object(m)
    };
    let ev = ev_with(&[("state", state), ("i", num(1.0))]);
    assert_eq!(ev.evaluate("state.items[i].name").unwrap(), s("bob"));
    assert_eq!(ev.evaluate("state.items[0].name").unwrap(), s("alice"));
    assert_eq!(
        ev.evaluate("state['items'][1]['name']").unwrap(),
        s("bob")
    );
}
