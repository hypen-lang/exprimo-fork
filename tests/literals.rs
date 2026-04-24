//! Edge case coverage for complex array and object literals.

use exprimo::{EvaluationError, Evaluator};
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

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = serde_json::Map::new();
    for (k, v) in pairs {
        m.insert(k.to_string(), v.clone());
    }
    Value::Object(m)
}

// =============================================================================
// Array literals
// =============================================================================

#[test]
fn empty_array_literal() {
    assert_eq!(ev().evaluate("[]").unwrap(), Value::Array(vec![]));
}

#[test]
fn single_element_array() {
    assert_eq!(
        ev().evaluate("[1]").unwrap(),
        Value::Array(vec![num(1.0)])
    );
}

#[test]
fn array_all_numbers() {
    assert_eq!(
        ev().evaluate("[1, 2, 3]").unwrap(),
        Value::Array(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn array_all_strings() {
    assert_eq!(
        ev().evaluate("['a', 'b', 'c']").unwrap(),
        Value::Array(vec![s("a"), s("b"), s("c")])
    );
}

#[test]
fn array_mixed_primitives() {
    assert_eq!(
        ev().evaluate("[1, 'two', true, false, null]").unwrap(),
        Value::Array(vec![
            num(1.0),
            s("two"),
            Value::Bool(true),
            Value::Bool(false),
            Value::Null,
        ])
    );
}

#[test]
fn array_with_trailing_comma() {
    // Parser accepts the trailing comma and drops it.
    assert_eq!(
        ev().evaluate("[1, 2, 3,]").unwrap(),
        Value::Array(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn array_with_sparse_holes_collapses() {
    // rslint_parser does not expose sparse holes, so [,] and [1,,3] collapse.
    // This is a documented deviation from JS where holes produce `undefined`.
    assert_eq!(ev().evaluate("[,]").unwrap(), Value::Array(vec![]));
    assert_eq!(
        ev().evaluate("[1,,3]").unwrap(),
        Value::Array(vec![num(1.0), num(3.0)])
    );
}

#[test]
fn array_with_expressions_as_elements() {
    assert_eq!(
        ev().evaluate("[1 + 1, 2 * 2, 'a' + 'b']").unwrap(),
        Value::Array(vec![num(2.0), num(4.0), s("ab")])
    );
}

#[test]
fn array_with_function_calls() {
    assert_eq!(
        ev().evaluate("[Math.floor(1.5), Math.ceil(1.5), Math.abs(-3)]").unwrap(),
        Value::Array(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn array_with_ternary() {
    let ev = ev_with(&[("x", num(5.0))]);
    assert_eq!(
        ev.evaluate("[x > 0 ? 'pos' : 'neg', x]").unwrap(),
        Value::Array(vec![s("pos"), num(5.0)])
    );
}

#[test]
fn array_with_variable_references() {
    let ev = ev_with(&[("a", num(1.0)), ("b", num(2.0)), ("c", num(3.0))]);
    assert_eq!(
        ev.evaluate("[a, b, c]").unwrap(),
        Value::Array(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn array_nested_two_levels() {
    assert_eq!(
        ev().evaluate("[[1, 2], [3, 4]]").unwrap(),
        Value::Array(vec![
            Value::Array(vec![num(1.0), num(2.0)]),
            Value::Array(vec![num(3.0), num(4.0)]),
        ])
    );
}

#[test]
fn array_deeply_nested() {
    assert_eq!(
        ev().evaluate("[1, [2, [3, [4]]]]").unwrap(),
        Value::Array(vec![
            num(1.0),
            Value::Array(vec![
                num(2.0),
                Value::Array(vec![num(3.0), Value::Array(vec![num(4.0)])])
            ])
        ])
    );
}

#[test]
fn array_of_objects() {
    assert_eq!(
        ev().evaluate("[({a: 1}), ({a: 2})]").unwrap(),
        Value::Array(vec![obj(&[("a", num(1.0))]), obj(&[("a", num(2.0))])])
    );
}

#[test]
fn array_of_mixed_collections() {
    assert_eq!(
        ev().evaluate("[[1], ({a: 1}), [], ({})]").unwrap(),
        Value::Array(vec![
            Value::Array(vec![num(1.0)]),
            obj(&[("a", num(1.0))]),
            Value::Array(vec![]),
            obj(&[]),
        ])
    );
}

#[test]
fn array_literal_length() {
    assert_eq!(ev().evaluate("[1, 2, 3].length").unwrap(), num(3.0));
    assert_eq!(ev().evaluate("[].length").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("[[1, 2]].length").unwrap(), num(1.0));
}

#[test]
fn array_literal_indexing() {
    assert_eq!(ev().evaluate("[10, 20, 30][0]").unwrap(), num(10.0));
    assert_eq!(ev().evaluate("[10, 20, 30][1]").unwrap(), num(20.0));
    assert_eq!(ev().evaluate("[10, 20, 30][5]").unwrap(), Value::Null);
}

#[test]
fn array_literal_method_calls() {
    assert_eq!(
        ev().evaluate("[1, 2, 3].includes(2)").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(ev().evaluate("[1, 2, 3].indexOf(3)").unwrap(), num(2.0));
    assert_eq!(
        ev().evaluate("['a', 'b', 'c'].join('-')").unwrap(),
        s("a-b-c")
    );
    assert_eq!(
        ev().evaluate("[1, 2, 3].slice(0, 2)").unwrap(),
        Value::Array(vec![num(1.0), num(2.0)])
    );
}

#[test]
fn array_literal_in_ternary() {
    let ev = ev_with(&[("flag", Value::Bool(true))]);
    assert_eq!(
        ev.evaluate("flag ? [1, 2] : [3, 4]").unwrap(),
        Value::Array(vec![num(1.0), num(2.0)])
    );
}

#[test]
fn array_literal_in_equality() {
    // JS: array equality is always false for distinct instances.
    assert_eq!(
        ev().evaluate("[1, 2] == [1, 2]").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn array_literal_is_truthy() {
    assert_eq!(
        ev().evaluate("[] ? 'truthy' : 'falsy'").unwrap(),
        s("truthy")
    );
    assert_eq!(
        ev().evaluate("[0] ? 'truthy' : 'falsy'").unwrap(),
        s("truthy")
    );
}

// =============================================================================
// Object literals
// =============================================================================

#[test]
fn empty_object_literal_needs_parens() {
    assert_eq!(ev().evaluate("({})").unwrap(), obj(&[]));
}

#[test]
fn single_pair_object() {
    assert_eq!(ev().evaluate("({a: 1})").unwrap(), obj(&[("a", num(1.0))]));
}

#[test]
fn object_multiple_pairs_alphabetical_output() {
    // Input insertion order doesn't matter — BTreeMap sorts alphabetically.
    assert_eq!(
        ev().evaluate("({c: 3, a: 1, b: 2})").unwrap(),
        obj(&[("a", num(1.0)), ("b", num(2.0)), ("c", num(3.0))])
    );
}

#[test]
fn object_trailing_comma_accepted() {
    assert_eq!(
        ev().evaluate("({a: 1, b: 2,})").unwrap(),
        obj(&[("a", num(1.0)), ("b", num(2.0))])
    );
}

#[test]
fn object_string_keys_single_quoted() {
    assert_eq!(
        ev().evaluate("({'a': 1, 'b': 2})").unwrap(),
        obj(&[("a", num(1.0)), ("b", num(2.0))])
    );
}

#[test]
fn object_string_keys_double_quoted() {
    assert_eq!(
        ev().evaluate("({\"a\": 1, \"b\": 2})").unwrap(),
        obj(&[("a", num(1.0)), ("b", num(2.0))])
    );
}

#[test]
fn object_string_keys_with_spaces() {
    assert_eq!(
        ev().evaluate("({'hello world': 1})").unwrap(),
        obj(&[("hello world", num(1.0))])
    );
}

#[test]
fn object_empty_string_key() {
    assert_eq!(
        ev().evaluate("({'': 'val'})").unwrap(),
        obj(&[("", s("val"))])
    );
}

#[test]
fn object_value_types() {
    assert_eq!(
        ev().evaluate("({n: 1, s: 'x', b: true, nil: null})").unwrap(),
        obj(&[
            ("b", Value::Bool(true)),
            ("n", num(1.0)),
            ("nil", Value::Null),
            ("s", s("x")),
        ])
    );
}

#[test]
fn object_value_is_expression() {
    let ev = ev_with(&[("x", num(3.0))]);
    assert_eq!(
        ev.evaluate("({sum: x + 2, product: x * x})").unwrap(),
        obj(&[("product", num(9.0)), ("sum", num(5.0))])
    );
}

#[test]
fn object_value_is_variable() {
    let ev = ev_with(&[("foo", s("bar"))]);
    assert_eq!(
        ev.evaluate("({x: foo})").unwrap(),
        obj(&[("x", s("bar"))])
    );
}

#[test]
fn object_value_is_function_call() {
    assert_eq!(
        ev().evaluate("({x: Math.abs(-5), y: Math.floor(3.7)})").unwrap(),
        obj(&[("x", num(5.0)), ("y", num(3.0))])
    );
}

#[test]
fn object_nested_object() {
    assert_eq!(
        ev().evaluate("({outer: {inner: 42}})").unwrap(),
        obj(&[("outer", obj(&[("inner", num(42.0))]))])
    );
}

#[test]
fn object_nested_deep() {
    assert_eq!(
        ev().evaluate("({a: {b: {c: 'deep'}}})").unwrap(),
        obj(&[("a", obj(&[("b", obj(&[("c", s("deep"))]))]))])
    );
}

#[test]
fn object_with_array_value() {
    assert_eq!(
        ev().evaluate("({items: [1, 2, 3]})").unwrap(),
        obj(&[("items", Value::Array(vec![num(1.0), num(2.0), num(3.0)]))])
    );
}

#[test]
fn object_combining_arrays_and_nested_objects() {
    assert_eq!(
        ev().evaluate("({list: [1, 2], data: {k: 'v'}, flag: true})").unwrap(),
        obj(&[
            ("data", obj(&[("k", s("v"))])),
            ("flag", Value::Bool(true)),
            ("list", Value::Array(vec![num(1.0), num(2.0)])),
        ])
    );
}

#[test]
fn object_duplicate_keys_last_wins() {
    // Matches JS: {a:1, a:2} → {a: 2}
    assert_eq!(
        ev().evaluate("({a: 1, a: 2})").unwrap(),
        obj(&[("a", num(2.0))])
    );
}

#[test]
fn object_property_dot_access() {
    assert_eq!(ev().evaluate("({a: 1, b: 2}).a").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("({a: 1, b: 2}).b").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("({a: 1}).missing").unwrap(), Value::Null);
}

#[test]
fn object_property_bracket_access() {
    assert_eq!(ev().evaluate("({a: 1, b: 2})['a']").unwrap(), num(1.0));
    assert_eq!(
        ev().evaluate("({a: 1, b: 2})['missing']").unwrap(),
        Value::Null
    );
}

#[test]
fn object_nested_chain_access() {
    assert_eq!(
        ev().evaluate("({a: ({b: 42})}).a.b").unwrap(),
        num(42.0)
    );
    assert_eq!(
        ev().evaluate("({a: ({b: ({c: 'x'})})})['a']['b']['c']").unwrap(),
        s("x")
    );
}

#[test]
fn object_literal_length_hasOwnProperty() {
    // Objects are iterable via Object.keys, and hasOwnProperty works on literals.
    assert_eq!(
        ev().evaluate("({a: 1, b: 2}).hasOwnProperty('a')").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev().evaluate("({a: 1, b: 2}).hasOwnProperty('x')").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn object_literal_with_namespace_methods() {
    assert_eq!(
        ev().evaluate("Object.keys({a: 1, b: 2, c: 3}).length").unwrap(),
        num(3.0)
    );
    assert_eq!(
        ev().evaluate("Object.values({a: 'x', b: 'y'})").unwrap(),
        Value::Array(vec![s("x"), s("y")])
    );
}

#[test]
fn object_literal_in_ternary() {
    let ev = ev_with(&[("flag", Value::Bool(true))]);
    assert_eq!(
        ev.evaluate("flag ? ({a: 1}) : ({a: 2})").unwrap(),
        obj(&[("a", num(1.0))])
    );
}

#[test]
fn object_literal_is_truthy() {
    assert_eq!(
        ev().evaluate("({}) ? 'yes' : 'no'").unwrap(),
        s("yes")
    );
    assert_eq!(
        ev().evaluate("({a: 1}) ? 'yes' : 'no'").unwrap(),
        s("yes")
    );
}

#[test]
fn object_equality_is_false_for_distinct_instances() {
    assert_eq!(
        ev().evaluate("({a: 1}) == ({a: 1})").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn object_shorthand_property_is_rejected() {
    // `{foo}` shorthand not supported.
    match ev().evaluate("({foo})") {
        Err(EvaluationError::Node(_)) => {}
        other => panic!("expected Node error for shorthand, got {:?}", other),
    }
}

// =============================================================================
// Interop: array of objects, object of arrays
// =============================================================================

#[test]
fn array_of_objects_indexed_and_keyed() {
    assert_eq!(
        ev().evaluate("[({name: 'a'}), ({name: 'b'})][0].name").unwrap(),
        s("a")
    );
    assert_eq!(
        ev().evaluate("[({name: 'a'}), ({name: 'b'})][1]['name']").unwrap(),
        s("b")
    );
}

#[test]
fn object_of_arrays_indexing() {
    assert_eq!(
        ev().evaluate("({nums: [10, 20, 30]}).nums[1]").unwrap(),
        num(20.0)
    );
    assert_eq!(
        ev().evaluate("({nums: [10, 20, 30]})['nums'].length").unwrap(),
        num(3.0)
    );
}

#[test]
fn deeply_nested_literal_access_chain() {
    assert_eq!(
        ev()
            .evaluate("({a: [({b: [({c: 'found'})]})]}).a[0].b[0].c")
            .unwrap(),
        s("found")
    );
}

#[test]
fn array_literal_method_on_object_literal_values() {
    // Object.values on inline object, then array method on the result.
    assert_eq!(
        ev()
            .evaluate("Object.values({a: 1, b: 2, c: 3}).includes(2)")
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev()
            .evaluate("Object.keys({z: 1, a: 2}).join('-')")
            .unwrap(),
        s("a-z")
    );
}
