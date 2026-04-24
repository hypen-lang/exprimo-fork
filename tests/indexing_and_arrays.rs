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
    // Integer-valued f64s now render canonically (no trailing ".0").
    assert_eq!(ev2.evaluate("a.join(',')").unwrap(), s("1,,3"));
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

// =============================================================================
// Bracket indexing — coercion of unusual index types
// =============================================================================

fn arr3() -> Value {
    Value::Array(vec![s("a"), s("b"), s("c")])
}

#[test]
fn array_index_with_boolean_true_becomes_one() {
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a[true]").unwrap(), s("b"));
}

#[test]
fn array_index_with_boolean_false_becomes_zero() {
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a[false]").unwrap(), s("a"));
}

#[test]
fn array_index_with_null_becomes_zero() {
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a[null]").unwrap(), s("a"));
}

#[test]
fn array_index_with_string_numeric() {
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a['0']").unwrap(), s("a"));
    assert_eq!(ev.evaluate("a['1']").unwrap(), s("b"));
    assert_eq!(ev.evaluate("a['2']").unwrap(), s("c"));
}

#[test]
fn array_index_with_infinity_out_of_range() {
    // Infinity sentinel is f64::MAX which saturates as usize::MAX on cast.
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a[Infinity]").unwrap(), Value::Null);
}

#[test]
fn array_index_with_negative_infinity() {
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a[-Infinity]").unwrap(), Value::Null);
}

#[test]
fn array_index_with_huge_number_does_not_panic() {
    // 1e20 > usize::MAX on 64-bit; the `as usize` cast saturates.
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a[1e20]").unwrap(), Value::Null);
    assert_eq!(ev.evaluate("a[1e300]").unwrap(), Value::Null);
}

#[test]
fn array_index_with_negative_zero_behaves_as_zero() {
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a[-0]").unwrap(), s("a"));
}

#[test]
fn array_index_with_expression_result() {
    let ev = ev_with(&[("a", arr3())]);
    assert_eq!(ev.evaluate("a[1 + 0]").unwrap(), s("b"));
    assert_eq!(ev.evaluate("a[2 * 1]").unwrap(), s("c"));
    // 1 - 2 = -1 → null
    assert_eq!(ev.evaluate("a[1 - 2]").unwrap(), Value::Null);
}

#[test]
fn object_index_with_boolean_matches_stringified_key() {
    // The literal keys "true" and "false" (as stored in the map) are found
    // when indexing with Boolean, which coerces to "true"/"false".
    let mut m = serde_json::Map::new();
    m.insert("true".into(), s("T"));
    m.insert("false".into(), s("F"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o[true]").unwrap(), s("T"));
    assert_eq!(ev.evaluate("o[false]").unwrap(), s("F"));
}

#[test]
fn object_index_with_null_matches_null_key() {
    let mut m = serde_json::Map::new();
    m.insert("null".into(), s("was-null"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o[null]").unwrap(), s("was-null"));
}

#[test]
fn object_index_with_numeric_literal_canonicalizes_key() {
    // JS ToPropertyKey: the number 1 stringifies to "1", matching the
    // JSON-standard key.
    let mut m = serde_json::Map::new();
    m.insert("1".into(), s("one"));
    m.insert("1.0".into(), s("one-point-zero"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o[1]").unwrap(), s("one"));
    // The "1.0" key is still only reachable via the explicit string form.
    assert_eq!(ev.evaluate("o['1.0']").unwrap(), s("one-point-zero"));
}

#[test]
fn object_index_with_numeric_expression_canonicalizes_key() {
    // 1 + 1 = 2 — canonical "2" key, not "2.0".
    let mut m = serde_json::Map::new();
    m.insert("2".into(), s("two"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o[1 + 1]").unwrap(), s("two"));
}

#[test]
fn object_index_with_fractional_uses_decimal_form() {
    let mut m = serde_json::Map::new();
    m.insert("1.5".into(), s("found"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o[1.5]").unwrap(), s("found"));
}

#[test]
fn string_index_with_boolean() {
    let ev = ev_with(&[("s", s("abc"))]);
    assert_eq!(ev.evaluate("s[true]").unwrap(), s("b"));
    assert_eq!(ev.evaluate("s[false]").unwrap(), s("a"));
}

#[test]
fn string_index_with_fractional_returns_null() {
    let ev = ev_with(&[("s", s("abc"))]);
    assert_eq!(ev.evaluate("s[0.5]").unwrap(), Value::Null);
    assert_eq!(ev.evaluate("s[1.7]").unwrap(), Value::Null);
}

#[test]
fn string_index_with_string_numeric() {
    let ev = ev_with(&[("s", s("abc"))]);
    assert_eq!(ev.evaluate("s['1']").unwrap(), s("b"));
}

#[test]
fn string_index_with_infinity_returns_null() {
    let ev = ev_with(&[("s", s("abc"))]);
    assert_eq!(ev.evaluate("s[Infinity]").unwrap(), Value::Null);
}

// =============================================================================
// join — recursive stringification of nested arrays and objects
// =============================================================================
//
// JS: Array.prototype.toString is equivalent to .join(','), and join
// stringifies each element via toString (recursive for arrays, static
// "[object Object]" for objects). exprimo now matches this exactly,
// including canonical number stringification (integer-valued floats
// render without trailing ".0").

#[test]
fn join_nested_arrays_recursively() {
    // JS: [[1,2], [3,4]].join('-') === "1,2-3,4"
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![
            Value::Array(vec![num(1.0), num(2.0)]),
            Value::Array(vec![num(3.0), num(4.0)]),
        ]),
    )]);
    assert_eq!(ev.evaluate("a.join('-')").unwrap(), s("1,2-3,4"));
}

#[test]
fn join_nested_arrays_with_integer_valued_numbers() {
    // Integer-typed numbers (from context, not literals) stringify cleanly.
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![
            Value::Array(vec![Value::Number(1.into()), Value::Number(2.into())]),
            Value::Array(vec![Value::Number(3.into()), Value::Number(4.into())]),
        ]),
    )]);
    assert_eq!(ev.evaluate("a.join('-')").unwrap(), s("1,2-3,4"));
}

#[test]
fn join_single_nested_array() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![Value::Array(vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(3.into()),
        ])]),
    )]);
    // One outer element (a 3-element array). Default separator never used.
    assert_eq!(ev.evaluate("a.join(',')").unwrap(), s("1,2,3"));
}

#[test]
fn join_array_containing_object() {
    // Objects stringify as "[object Object]" (JS: same).
    let mut m = serde_json::Map::new();
    m.insert("a".into(), Value::Number(1.into()));
    let ev = ev_with(&[("a", Value::Array(vec![Value::Object(m)]))]);
    assert_eq!(ev.evaluate("a.join(',')").unwrap(), s("[object Object]"));
}

#[test]
fn join_mixed_nesting() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![
            Value::Number(1.into()),
            Value::Array(vec![Value::Number(2.into()), Value::Number(3.into())]),
            Value::Number(4.into()),
        ]),
    )]);
    assert_eq!(ev.evaluate("a.join('|')").unwrap(), s("1|2,3|4"));
}

#[test]
fn join_of_all_nulls() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![Value::Null, Value::Null, Value::Null]),
    )]);
    assert_eq!(ev.evaluate("a.join(',')").unwrap(), s(",,"));
}

#[test]
fn concatenation_stringifies_arrays_recursively() {
    // 'x' + [1,2] in JS becomes "x1,2" (canonical ToString).
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![Value::Number(1.into()), Value::Number(2.into())]),
    )]);
    assert_eq!(ev.evaluate("'x' + a").unwrap(), s("x1,2"));
    assert_eq!(ev.evaluate("a + 'x'").unwrap(), s("1,2x"));
}

#[test]
fn concatenation_stringifies_objects() {
    let mut m = serde_json::Map::new();
    m.insert("k".into(), s("v"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o + ''").unwrap(), s("[object Object]"));
    assert_eq!(ev.evaluate("'pre:' + o").unwrap(), s("pre:[object Object]"));
}

#[test]
fn concatenation_stringifies_nested_structures() {
    // Deep array inside a string concat still recurses.
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![
            Value::Number(1.into()),
            Value::Array(vec![Value::Number(2.into()), Value::Number(3.into())]),
        ]),
    )]);
    assert_eq!(ev.evaluate("'[' + a + ']'").unwrap(), s("[1,2,3]"));
}

// =============================================================================
// Array.indexOf — reference inequality of compound values
// =============================================================================

#[test]
fn array_index_of_nested_array_literal_not_found() {
    // JS: array literals are new objects each time; strict equality is by ref.
    assert_eq!(ev().evaluate("[[1, 2]].indexOf([1, 2])").unwrap(), num(-1.0));
}

#[test]
fn array_index_of_nested_object_literal_not_found() {
    assert_eq!(
        ev().evaluate("[({a: 1})].indexOf({a: 1})").unwrap(),
        num(-1.0)
    );
}

#[test]
fn array_index_of_returns_first_of_multiple_occurrences() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![
            s("x"),
            s("y"),
            s("x"),
            s("y"),
            s("x"),
        ]),
    )]);
    assert_eq!(ev.evaluate("a.indexOf('x')").unwrap(), num(0.0));
    assert_eq!(ev.evaluate("a.indexOf('y')").unwrap(), num(1.0));
}

// =============================================================================
// Array.slice — more permutations
// =============================================================================

#[test]
fn array_slice_fractional_indices_truncate() {
    // JS ToIntegerOrInfinity truncates toward zero; exprimo does the same.
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![Value::Number(1.into()), Value::Number(2.into()), Value::Number(3.into())]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice(0.5, 2.5)").unwrap(),
        Value::Array(vec![Value::Number(1.into()), Value::Number(2.into())])
    );
}

#[test]
fn array_slice_string_coerced_indices() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![Value::Number(1.into()), Value::Number(2.into()), Value::Number(3.into())]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice('1', '3')").unwrap(),
        Value::Array(vec![Value::Number(2.into()), Value::Number(3.into())])
    );
}

#[test]
fn array_slice_negative_zero() {
    // -0 is treated as 0; slice(-0) returns a full copy.
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![Value::Number(1.into()), Value::Number(2.into()), Value::Number(3.into())]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice(-0)").unwrap(),
        Value::Array(vec![Value::Number(1.into()), Value::Number(2.into()), Value::Number(3.into())])
    );
}

#[test]
fn array_slice_null_and_boolean_indices() {
    // null → 0; true → 1; false → 0
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![Value::Number(1.into()), Value::Number(2.into()), Value::Number(3.into()), Value::Number(4.into())]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice(null, 2)").unwrap(),
        Value::Array(vec![Value::Number(1.into()), Value::Number(2.into())])
    );
    assert_eq!(
        ev.evaluate("a.slice(true, 3)").unwrap(),
        Value::Array(vec![Value::Number(2.into()), Value::Number(3.into())])
    );
    assert_eq!(
        ev.evaluate("a.slice(false, true)").unwrap(),
        Value::Array(vec![Value::Number(1.into())])
    );
}

// =============================================================================
// Indexing on function-call results (systematic)
// =============================================================================

#[test]
fn index_into_object_keys_array() {
    let mut m = serde_json::Map::new();
    m.insert("alpha".into(), Value::Number(1.into()));
    m.insert("beta".into(), Value::Number(2.into()));
    m.insert("gamma".into(), Value::Number(3.into()));
    let ev = ev_with(&[("o", Value::Object(m))]);
    // BTreeMap: alphabetical, so [0] === "alpha"
    assert_eq!(ev.evaluate("Object.keys(o)[0]").unwrap(), s("alpha"));
    assert_eq!(ev.evaluate("Object.keys(o)[2]").unwrap(), s("gamma"));
}

#[test]
fn chained_function_call_then_index_then_method() {
    let mut m = serde_json::Map::new();
    m.insert("first".into(), s("alice"));
    m.insert("second".into(), s("bob"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    // Object.keys → ["first", "second"]; [0] === "first"; .toUpperCase() === "FIRST"
    assert_eq!(
        ev.evaluate("Object.keys(o)[0].toUpperCase()").unwrap(),
        s("FIRST")
    );
}

#[test]
fn index_into_slice_result() {
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![
            Value::Number(10.into()),
            Value::Number(20.into()),
            Value::Number(30.into()),
            Value::Number(40.into()),
        ]),
    )]);
    assert_eq!(
        ev.evaluate("a.slice(1, 3)[0]").unwrap(),
        Value::Number(20.into())
    );
    assert_eq!(
        ev.evaluate("a.slice(-2)[1]").unwrap(),
        Value::Number(40.into())
    );
}

#[test]
fn index_into_string_slice_result() {
    assert_eq!(ev().evaluate("'hello world'.slice(6)[0]").unwrap(), s("w"));
    assert_eq!(ev().evaluate("'hello world'.slice(6)[4]").unwrap(), s("d"));
    assert_eq!(ev().evaluate("'hello'.toUpperCase()[0]").unwrap(), s("H"));
}

#[test]
fn index_into_join_result_char_by_char() {
    // join returns a string; bracket index returns single char.
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![s("ab"), s("cd")]),
    )]);
    // "ab,cd"[2] === ","
    assert_eq!(ev.evaluate("a.join(',')[2]").unwrap(), s(","));
}

#[test]
fn index_result_of_object_values_chained() {
    let mut m = serde_json::Map::new();
    m.insert("a".into(), Value::Number(100.into()));
    m.insert("b".into(), Value::Number(200.into()));
    let ev = ev_with(&[("o", Value::Object(m))]);
    // Values are alphabetical by key → [100, 200]; [1] === 200
    assert_eq!(
        ev.evaluate("Object.values(o)[1]").unwrap(),
        Value::Number(200.into())
    );
}
