//! Tests targeting the three JS-compliance fixes:
//!   1. Bracket-member calls resolve like dot-member calls
//!      (arr['join'](','), Math['floor'](x), etc.).
//!   2. Canonical JS ToString/ToPropertyKey for numbers: 1 → "1" (no trailing
//!      ".0"). This fixes property keys, concatenation, and join output.
//!   3. Own properties on an object shadow the hasOwnProperty built-in, per
//!      JS prototype-chain semantics.

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

// =============================================================================
// Fix 1 — bracket-member calls
// =============================================================================

#[test]
fn bracket_call_array_join() {
    assert_eq!(
        ev().evaluate("['a','b','c']['join'](',')").unwrap(),
        s("a,b,c")
    );
}

#[test]
fn bracket_call_array_includes() {
    assert_eq!(
        ev().evaluate("[1, 2, 3]['includes'](2)").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev().evaluate("[1, 2, 3]['includes'](99)").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn bracket_call_array_indexof_and_slice() {
    assert_eq!(
        ev().evaluate("['a','b','c']['indexOf']('b')").unwrap(),
        num(1.0)
    );
    assert_eq!(
        ev().evaluate("['a','b','c']['slice'](1)").unwrap(),
        Value::Array(vec![s("b"), s("c")])
    );
}

#[test]
fn bracket_call_string_methods() {
    assert_eq!(
        ev().evaluate("'hello'['toUpperCase']()").unwrap(),
        s("HELLO")
    );
    assert_eq!(
        ev().evaluate("'  hi  '['trim']()").unwrap(),
        s("hi")
    );
    assert_eq!(
        ev().evaluate("'hello'['slice'](1, 4)").unwrap(),
        s("ell")
    );
    assert_eq!(
        ev().evaluate("'hello'['indexOf']('ll')").unwrap(),
        num(2.0)
    );
    assert_eq!(
        ev().evaluate("'hello'['includes']('ell')").unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn bracket_call_number_to_fixed() {
    assert_eq!(
        ev().evaluate("(3.14)['toFixed'](1)").unwrap(),
        s("3.1")
    );
}

#[test]
fn bracket_call_math_namespace() {
    assert_eq!(ev().evaluate("Math['floor'](1.9)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math['ceil'](1.1)").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("Math['abs'](-5)").unwrap(), num(5.0));
    assert_eq!(ev().evaluate("Math['min'](3, 7, 1)").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("Math['max'](3, 7, 1)").unwrap(), num(7.0));
}

#[test]
fn bracket_call_object_namespace() {
    let mut m = serde_json::Map::new();
    m.insert("a".into(), num(1.0));
    m.insert("b".into(), num(2.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(
        ev.evaluate("Object['keys'](o)").unwrap(),
        Value::Array(vec![s("a"), s("b")])
    );
    assert_eq!(
        ev.evaluate("Object['values'](o)").unwrap(),
        Value::Array(vec![num(1.0), num(2.0)])
    );
}

#[test]
fn bracket_call_object_has_own_property() {
    let mut m = serde_json::Map::new();
    m.insert("x".into(), num(1.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(
        ev.evaluate("o['hasOwnProperty']('x')").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev.evaluate("o['hasOwnProperty']('y')").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn bracket_call_with_dynamic_method_name() {
    // The property expression can itself be dynamic.
    let ev = ev_with(&[("method", s("join"))]);
    assert_eq!(
        ev.evaluate("['a','b','c'][method](',')").unwrap(),
        s("a,b,c")
    );
}

#[test]
fn bracket_call_with_computed_method_name() {
    // Concatenation inside the bracket resolves the method name.
    assert_eq!(
        ev().evaluate("['a','b'][('j' + 'oin')](',')").unwrap(),
        s("a,b")
    );
}

#[test]
fn bracket_call_parity_with_dot_call() {
    // Every dot-call should have an equivalent bracket-call form.
    let cases: &[(&str, &str)] = &[
        ("[1,2,3].join('-')", "[1,2,3]['join']('-')"),
        ("[1,2,3].length", "[1,2,3]['length']"),
        ("'hi'.length", "'hi'['length']"),
        ("'hi'.toUpperCase()", "'hi'['toUpperCase']()"),
        ("Math.floor(1.5)", "Math['floor'](1.5)"),
        (
            "({a: 1}).hasOwnProperty('a')",
            "({a: 1})['hasOwnProperty']('a')",
        ),
    ];
    let ev = ev();
    for (dot, bracket) in cases {
        let dot_result = ev.evaluate(dot).expect(dot);
        let bracket_result = ev.evaluate(bracket).expect(bracket);
        assert_eq!(
            dot_result, bracket_result,
            "dot vs bracket differ for {} vs {}",
            dot, bracket
        );
    }
}

#[test]
fn bracket_call_on_non_function_errors() {
    let mut m = serde_json::Map::new();
    m.insert("x".into(), num(1.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    match ev.evaluate("o['x']('arg')") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("is not a function"), "got: {}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn bracket_call_unknown_method_returns_null_then_not_function() {
    // arr['unknown'] resolves to null, then calling null errors.
    match ev().evaluate("[1,2,3]['unknown']()") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("is not a function"), "got: {}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn bracket_call_unknown_math_method_errors() {
    // Math['foo'](1) hits the same namespace resolver that dot does.
    match ev().evaluate("Math['foo'](1)") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("Math.foo"), "got: {}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn bracket_call_chains_with_dot_call() {
    // Mixed bracket/dot chaining.
    assert_eq!(
        ev().evaluate("'  hello  '['trim']().toUpperCase()").unwrap(),
        s("HELLO")
    );
    assert_eq!(
        ev().evaluate("'  hello  '.trim()['toUpperCase']()").unwrap(),
        s("HELLO")
    );
}

// =============================================================================
// Fix 2 — canonical number stringification (ToString / ToPropertyKey)
// =============================================================================

#[test]
fn canonical_concat_with_integer_literal() {
    // Was "x=1.0" before the fix.
    assert_eq!(ev().evaluate("'x=' + 1").unwrap(), s("x=1"));
    assert_eq!(ev().evaluate("'x=' + 42").unwrap(), s("x=42"));
    assert_eq!(ev().evaluate("'x=' + (-5)").unwrap(), s("x=-5"));
}

#[test]
fn canonical_concat_with_fractional_literal() {
    // Fractional forms keep their decimal.
    assert_eq!(ev().evaluate("'x=' + 1.5").unwrap(), s("x=1.5"));
    assert_eq!(ev().evaluate("'x=' + (-0.25)").unwrap(), s("x=-0.25"));
}

#[test]
fn canonical_concat_with_arithmetic_result() {
    // Integer-valued arithmetic output: no trailing ".0".
    assert_eq!(ev().evaluate("'n=' + (2 + 3)").unwrap(), s("n=5"));
    assert_eq!(ev().evaluate("'n=' + (10 / 2)").unwrap(), s("n=5"));
    // Fractional arithmetic keeps decimal.
    assert_eq!(ev().evaluate("'n=' + (10 / 4)").unwrap(), s("n=2.5"));
}

#[test]
fn canonical_array_join_integer_literals() {
    // Was "1.0,2.0,3.0" before the fix.
    assert_eq!(
        ev().evaluate("[1, 2, 3].join(',')").unwrap(),
        s("1,2,3")
    );
    assert_eq!(
        ev().evaluate("[10, 20, 30].join(' + ')").unwrap(),
        s("10 + 20 + 30")
    );
}

#[test]
fn canonical_array_to_string_coercion_via_concat() {
    // Pure concatenation path — verified elsewhere too, but pin the canonical
    // form of the empty-element case and the integer-list case here.
    assert_eq!(ev().evaluate("'' + [1, 2, 3]").unwrap(), s("1,2,3"));
    assert_eq!(ev().evaluate("'' + []").unwrap(), s(""));
    // Note: abstract equality `[1,2,3] == '1,2,3'` currently returns false
    // because exprimo does not coerce arrays via ToPrimitive in `==`. That's
    // a deeper semantic gap not covered by this fix pass.
}

#[test]
fn canonical_infinity_and_nan_stringify_correctly() {
    assert_eq!(ev().evaluate("'v=' + Infinity").unwrap(), s("v=Infinity"));
    assert_eq!(ev().evaluate("'v=' + (-Infinity)").unwrap(), s("v=-Infinity"));
    // The `NaN` identifier in exprimo resolves to 0 (serde_json can't store
    // NaN), so "NaN" itself is unreachable via the identifier path. Verify
    // the coerced-from-unparseable-string path instead, which produces NaN
    // → Null during arithmetic.
    assert_eq!(ev().evaluate("'v=' + ('abc' * 1)").unwrap(), s("v=null"));
}

#[test]
fn canonical_object_key_from_numeric_literal() {
    // `({1: 'a'})[1]` should find the key "1" (canonical), not "1.0".
    assert_eq!(ev().evaluate("({1: 'a'})[1]").unwrap(), s("a"));
    assert_eq!(ev().evaluate("({1: 'a', 2: 'b'})[2]").unwrap(), s("b"));
}

#[test]
fn canonical_object_key_lookup_from_context() {
    // Canonical lookup on a JSON-style map — the stored key is "1", and
    // `o[1]` should now find it (previously looked up "1.0").
    let mut m = serde_json::Map::new();
    m.insert("1".into(), s("one"));
    m.insert("2".into(), s("two"));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o[1]").unwrap(), s("one"));
    assert_eq!(ev.evaluate("o[2]").unwrap(), s("two"));
    assert_eq!(ev.evaluate("o[3]").unwrap(), Value::Null);
}

#[test]
fn canonical_hasOwnProperty_with_numeric_arg() {
    let mut m = serde_json::Map::new();
    m.insert("1".into(), num(1.0));
    m.insert("2".into(), num(2.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    // hasOwnProperty(1) coerces to "1" (canonical), matches.
    assert_eq!(
        ev.evaluate("o.hasOwnProperty(1)").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev.evaluate("o.hasOwnProperty(2)").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev.evaluate("o.hasOwnProperty(99)").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn canonical_property_name_in_object_literal() {
    // Numeric literal keys in an object literal canonicalize to "1", not "1.0".
    let result = ev().evaluate("Object.keys({1: 'a', 2: 'b'})").unwrap();
    assert_eq!(result, Value::Array(vec![s("1"), s("2")]));
}

#[test]
fn canonical_nested_collection_stringifies_without_decimal() {
    // Integer-valued f64s in nested arrays also render cleanly.
    let ev = ev_with(&[(
        "a",
        Value::Array(vec![
            Value::Array(vec![num(1.0), num(2.0)]),
            Value::Array(vec![num(3.0), num(4.0)]),
        ]),
    )]);
    // .join('-') uses '-' between top-level elements, ',' inside nested ones
    // (nested arrays always use the default ',' separator).
    assert_eq!(ev.evaluate("a.join('-')").unwrap(), s("1,2-3,4"));
    // The concat path uses the default Array.toString (join with ',') for the
    // outer array, which flattens: [[1,2],[3,4]] → "1,2,3,4".
    assert_eq!(ev.evaluate("'arr=' + a").unwrap(), s("arr=1,2,3,4"));
}

// =============================================================================
// Fix 3 — hasOwnProperty shadowing
// =============================================================================

#[test]
fn own_hasOwnProperty_shadows_builtin_as_value() {
    // JS: ({hasOwnProperty: 42}).hasOwnProperty === 42
    assert_eq!(
        ev().evaluate("({hasOwnProperty: 42}).hasOwnProperty").unwrap(),
        num(42.0)
    );
    assert_eq!(
        ev()
            .evaluate("({hasOwnProperty: 'shadowed'}).hasOwnProperty")
            .unwrap(),
        s("shadowed")
    );
}

#[test]
fn own_hasOwnProperty_shadows_via_bracket_access() {
    assert_eq!(
        ev()
            .evaluate("({hasOwnProperty: 'v'})['hasOwnProperty']")
            .unwrap(),
        s("v")
    );
}

#[test]
fn own_hasOwnProperty_when_not_callable_errors_on_call() {
    // ({hasOwnProperty: 1}).hasOwnProperty('x') — own prop wins, but 1 is
    // not a function, so the call errors.
    match ev().evaluate("({hasOwnProperty: 1}).hasOwnProperty('x')") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("is not a function"), "got: {}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn hasOwnProperty_still_builtin_when_not_shadowed() {
    // Normal case: object without an own `hasOwnProperty` property uses the
    // built-in.
    assert_eq!(
        ev().evaluate("({a: 1}).hasOwnProperty('a')").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev().evaluate("({a: 1}).hasOwnProperty('x')").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn own_hasOwnProperty_from_context_shadows_builtin() {
    // Shadowing still applies when the object comes from context.
    let mut m = serde_json::Map::new();
    m.insert("hasOwnProperty".into(), s("mine"));
    m.insert("a".into(), num(1.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o.hasOwnProperty").unwrap(), s("mine"));
    assert_eq!(ev.evaluate("o['hasOwnProperty']").unwrap(), s("mine"));
}

#[test]
fn own_property_lookup_precedes_builtin_resolution() {
    // The fix specifically moves `map.get` before the builtin check. Verify
    // that non-hasOwnProperty keys still flow through the existing path:
    // regular lookup returns the stored value and unknown keys return null.
    let mut m = serde_json::Map::new();
    m.insert("a".into(), num(10.0));
    let ev = ev_with(&[("o", Value::Object(m))]);
    assert_eq!(ev.evaluate("o.a").unwrap(), num(10.0));
    assert_eq!(ev.evaluate("o.b").unwrap(), Value::Null);
}

#[test]
fn callable_own_hasOwnProperty_is_not_invoked_as_method() {
    // Even if the own value happens to be something callable-like (e.g. a
    // custom function reference is not supported in literals, so use a
    // string), it still just returns the value — the call will error.
    let ev = ev_with(&[(
        "o",
        Value::Object({
            let mut m = serde_json::Map::new();
            m.insert("hasOwnProperty".into(), s("not-a-function"));
            m
        }),
    )]);
    // Getting the value works:
    assert_eq!(ev.evaluate("o.hasOwnProperty").unwrap(), s("not-a-function"));
    // Calling it errors:
    match ev.evaluate("o.hasOwnProperty('x')") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("is not a function"));
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}
