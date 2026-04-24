//! Edge case coverage for String built-in properties and methods.

use exprimo::{CustomFuncError, EvaluationError, Evaluator};
use serde_json::Value;
use std::collections::HashMap;

fn ev() -> Evaluator {
    Evaluator::new(HashMap::new(), HashMap::new())
}

fn ev_with(key: &str, value: Value) -> Evaluator {
    let mut ctx = HashMap::new();
    ctx.insert(key.to_string(), value);
    Evaluator::new(ctx, HashMap::new())
}

fn num(v: f64) -> Value {
    Value::Number(serde_json::Number::from_f64(v).unwrap())
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

// --- .length ---

#[test]
fn length_empty() {
    assert_eq!(ev().evaluate("''.length").unwrap(), num(0.0));
}

#[test]
fn length_ascii() {
    assert_eq!(ev().evaluate("'hello'.length").unwrap(), num(5.0));
}

#[test]
fn length_multibyte_characters() {
    // Length counts Unicode characters, not bytes.
    assert_eq!(ev().evaluate("'héllo'.length").unwrap(), num(5.0));
}

#[test]
fn length_emoji() {
    // A single emoji counts as 1 character in exprimo (char iterator).
    assert_eq!(ev().evaluate("'🎉'.length").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("'🎉abc'.length").unwrap(), num(4.0));
}

#[test]
fn length_with_escapes() {
    assert_eq!(ev().evaluate("'a\\nb'.length").unwrap(), num(3.0));
}

#[test]
fn length_from_context() {
    let ev = ev_with("name", s("Alice"));
    assert_eq!(ev.evaluate("name.length").unwrap(), num(5.0));
}

// --- case conversion ---

#[test]
fn to_upper_empty() {
    assert_eq!(ev().evaluate("''.toUpperCase()").unwrap(), s(""));
}

#[test]
fn to_upper_already_upper() {
    assert_eq!(ev().evaluate("'ABC'.toUpperCase()").unwrap(), s("ABC"));
}

#[test]
fn to_upper_mixed() {
    assert_eq!(ev().evaluate("'Hello World'.toUpperCase()").unwrap(), s("HELLO WORLD"));
}

#[test]
fn to_upper_numbers_preserved() {
    assert_eq!(ev().evaluate("'abc123!@#'.toUpperCase()").unwrap(), s("ABC123!@#"));
}

#[test]
fn to_upper_non_ascii() {
    assert_eq!(ev().evaluate("'héllo'.toUpperCase()").unwrap(), s("HÉLLO"));
}

#[test]
fn to_lower_empty() {
    assert_eq!(ev().evaluate("''.toLowerCase()").unwrap(), s(""));
}

#[test]
fn to_lower_already_lower() {
    assert_eq!(ev().evaluate("'abc'.toLowerCase()").unwrap(), s("abc"));
}

#[test]
fn to_lower_mixed() {
    assert_eq!(ev().evaluate("'Hello World'.toLowerCase()").unwrap(), s("hello world"));
}

#[test]
fn to_lower_non_ascii() {
    assert_eq!(ev().evaluate("'HÉLLO'.toLowerCase()").unwrap(), s("héllo"));
}

#[test]
fn case_methods_reject_args() {
    // toUpperCase / toLowerCase take zero args.
    match ev().evaluate("'x'.toUpperCase('y')") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected: 0, got: 1 })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// --- .trim() ---

#[test]
fn trim_empty() {
    assert_eq!(ev().evaluate("''.trim()").unwrap(), s(""));
}

#[test]
fn trim_only_whitespace() {
    assert_eq!(ev().evaluate("'   '.trim()").unwrap(), s(""));
    assert_eq!(ev().evaluate("'\\t\\n '.trim()").unwrap(), s(""));
}

#[test]
fn trim_internal_whitespace_preserved() {
    assert_eq!(ev().evaluate("'  a b c  '.trim()").unwrap(), s("a b c"));
}

#[test]
fn trim_no_whitespace() {
    assert_eq!(ev().evaluate("'hello'.trim()").unwrap(), s("hello"));
}

#[test]
fn trim_rejects_args() {
    match ev().evaluate("'x'.trim('y')") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// --- includes / startsWith / endsWith ---

#[test]
fn includes_empty_always_true() {
    // Matches JS: "".includes("") === true
    assert_eq!(ev().evaluate("''.includes('')").unwrap(), Value::Bool(true));
    assert_eq!(ev().evaluate("'abc'.includes('')").unwrap(), Value::Bool(true));
}

#[test]
fn includes_empty_haystack() {
    assert_eq!(ev().evaluate("''.includes('x')").unwrap(), Value::Bool(false));
}

#[test]
fn includes_case_sensitive() {
    assert_eq!(ev().evaluate("'Hello'.includes('hello')").unwrap(), Value::Bool(false));
    assert_eq!(ev().evaluate("'Hello'.includes('Hello')").unwrap(), Value::Bool(true));
}

#[test]
fn includes_coerces_non_string_arg() {
    // Numeric literals stringify with trailing ".0" (exprimo f64 quirk),
    // so `1` searches for "1.0" — not found in "a1b".
    assert_eq!(ev().evaluate("'abc'.includes(1)").unwrap(), Value::Bool(false));
    // The coerced form does match when the haystack contains "1.0".
    assert_eq!(ev().evaluate("'v=1.0 ok'.includes(1)").unwrap(), Value::Bool(true));
    // null coerces to "null"
    assert_eq!(ev().evaluate("'is null'.includes(null)").unwrap(), Value::Bool(true));
    // booleans coerce to "true"/"false"
    assert_eq!(ev().evaluate("'was true today'.includes(true)").unwrap(), Value::Bool(true));
}

#[test]
fn starts_with_empty_always_true() {
    assert_eq!(ev().evaluate("''.startsWith('')").unwrap(), Value::Bool(true));
    assert_eq!(ev().evaluate("'abc'.startsWith('')").unwrap(), Value::Bool(true));
}

#[test]
fn starts_with_exact() {
    assert_eq!(ev().evaluate("'hello'.startsWith('hello')").unwrap(), Value::Bool(true));
}

#[test]
fn starts_with_longer_than_haystack() {
    assert_eq!(ev().evaluate("'hi'.startsWith('hello')").unwrap(), Value::Bool(false));
}

#[test]
fn starts_with_case_sensitive() {
    assert_eq!(ev().evaluate("'Hello'.startsWith('hello')").unwrap(), Value::Bool(false));
}

#[test]
fn ends_with_empty_always_true() {
    assert_eq!(ev().evaluate("''.endsWith('')").unwrap(), Value::Bool(true));
    assert_eq!(ev().evaluate("'abc'.endsWith('')").unwrap(), Value::Bool(true));
}

#[test]
fn ends_with_exact() {
    assert_eq!(ev().evaluate("'hello'.endsWith('hello')").unwrap(), Value::Bool(true));
}

#[test]
fn ends_with_longer_than_haystack() {
    assert_eq!(ev().evaluate("'hi'.endsWith('hello')").unwrap(), Value::Bool(false));
}

#[test]
fn includes_requires_one_arg() {
    match ev().evaluate("'x'.includes()") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
    match ev().evaluate("'x'.includes('a', 'b')") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// --- slice(start, end?) ---

#[test]
fn slice_no_args_returns_full_copy() {
    assert_eq!(ev().evaluate("'hello'.slice()").unwrap(), s("hello"));
}

#[test]
fn slice_start_only() {
    assert_eq!(ev().evaluate("'hello'.slice(2)").unwrap(), s("llo"));
    assert_eq!(ev().evaluate("'hello'.slice(0)").unwrap(), s("hello"));
}

#[test]
fn slice_start_end() {
    assert_eq!(ev().evaluate("'hello'.slice(1, 4)").unwrap(), s("ell"));
}

#[test]
fn slice_negative_start() {
    assert_eq!(ev().evaluate("'hello'.slice(-2)").unwrap(), s("lo"));
    assert_eq!(ev().evaluate("'hello'.slice(-3, -1)").unwrap(), s("ll"));
}

#[test]
fn slice_out_of_range_start() {
    // Start past the end → empty string
    assert_eq!(ev().evaluate("'hello'.slice(100)").unwrap(), s(""));
}

#[test]
fn slice_negative_start_beyond_len_clamps_to_zero() {
    assert_eq!(ev().evaluate("'abc'.slice(-100)").unwrap(), s("abc"));
}

#[test]
fn slice_end_clamps_to_len() {
    assert_eq!(ev().evaluate("'hello'.slice(0, 100)").unwrap(), s("hello"));
}

#[test]
fn slice_end_before_start_returns_empty() {
    assert_eq!(ev().evaluate("'hello'.slice(4, 2)").unwrap(), s(""));
}

#[test]
fn slice_same_start_end() {
    assert_eq!(ev().evaluate("'hello'.slice(2, 2)").unwrap(), s(""));
}

#[test]
fn slice_on_empty_string() {
    assert_eq!(ev().evaluate("''.slice()").unwrap(), s(""));
    assert_eq!(ev().evaluate("''.slice(0, 5)").unwrap(), s(""));
    assert_eq!(ev().evaluate("''.slice(-1)").unwrap(), s(""));
}

#[test]
fn slice_on_multibyte_chars() {
    // char-based indexing, not byte-based
    assert_eq!(ev().evaluate("'🎉abc'.slice(0, 1)").unwrap(), s("🎉"));
    assert_eq!(ev().evaluate("'🎉abc'.slice(1, 3)").unwrap(), s("ab"));
    assert_eq!(ev().evaluate("'🎉abc'.slice(-3)").unwrap(), s("abc"));
}

#[test]
fn slice_rejects_more_than_two_args() {
    match ev().evaluate("'x'.slice(0, 1, 2)") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected: 2, got: 3 })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// --- indexOf ---

#[test]
fn index_of_found() {
    assert_eq!(ev().evaluate("'hello'.indexOf('l')").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("'hello'.indexOf('h')").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("'hello'.indexOf('o')").unwrap(), num(4.0));
}

#[test]
fn index_of_not_found() {
    assert_eq!(ev().evaluate("'hello'.indexOf('x')").unwrap(), num(-1.0));
}

#[test]
fn index_of_empty_string_returns_zero() {
    // Matches JS: any string .indexOf('') is 0.
    assert_eq!(ev().evaluate("'hello'.indexOf('')").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("''.indexOf('')").unwrap(), num(0.0));
}

#[test]
fn index_of_in_empty_haystack() {
    assert_eq!(ev().evaluate("''.indexOf('x')").unwrap(), num(-1.0));
}

#[test]
fn index_of_multibyte_reports_char_index() {
    // The 'a' appears after a 4-byte emoji but at char index 1.
    assert_eq!(ev().evaluate("'🎉abc'.indexOf('abc')").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("'🎉abc'.indexOf('a')").unwrap(), num(1.0));
    assert_eq!(ev().evaluate("'🎉abc'.indexOf('c')").unwrap(), num(3.0));
}

#[test]
fn index_of_coerces_non_string() {
    // Numeric literals stringify as "1.0" (f64 quirk), so we search for that form.
    assert_eq!(ev().evaluate("'v=1.0 ok'.indexOf(1)").unwrap(), num(2.0));
    assert_eq!(ev().evaluate("'null is null'.indexOf(null)").unwrap(), num(0.0));
    assert_eq!(ev().evaluate("'trueval'.indexOf(true)").unwrap(), num(0.0));
}

#[test]
fn index_of_requires_one_arg() {
    match ev().evaluate("'x'.indexOf()") {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { .. })) => {}
        other => panic!("expected arity error, got {:?}", other),
    }
}

// --- cross-method combinations ---

#[test]
fn chained_operations() {
    assert_eq!(ev().evaluate("'  Hello  '.trim().toUpperCase()").unwrap(), s("HELLO"));
    assert_eq!(
        ev().evaluate("'hello, world'.slice(0, 5).toUpperCase()").unwrap(),
        s("HELLO")
    );
}

#[test]
fn length_after_trim() {
    assert_eq!(ev().evaluate("'   hi   '.trim().length").unwrap(), num(2.0));
}

#[test]
fn slice_in_concatenation() {
    assert_eq!(
        ev().evaluate("'prefix:' + 'abcdef'.slice(0, 3)").unwrap(),
        s("prefix:abc")
    );
}

#[test]
fn methods_work_on_literal_and_context() {
    let mut ctx = HashMap::new();
    ctx.insert("name".to_string(), s("alice"));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(ev.evaluate("name.toUpperCase()").unwrap(), s("ALICE"));
    assert_eq!(ev.evaluate("name.length").unwrap(), num(5.0));
}

#[test]
fn unknown_string_method_yields_null_then_not_function() {
    // Unknown method resolves to null, then calling null errors with "not a function".
    match ev().evaluate("'x'.unknown()") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("is not a function"), "got: {}", msg);
        }
        other => panic!("expected TypeError, got {:?}", other),
    }
}

#[test]
fn unknown_string_property_returns_null() {
    assert_eq!(ev().evaluate("'x'.foo").unwrap(), Value::Null);
}

#[test]
fn string_in_ternary_via_length() {
    let ev = ev_with("s", s(""));
    assert_eq!(
        ev.evaluate("s.length == 0 ? 'empty' : 'nonempty'").unwrap(),
        s("empty")
    );
    let ev2 = ev_with("s", s("hi"));
    assert_eq!(
        ev2.evaluate("s.length == 0 ? 'empty' : 'nonempty'").unwrap(),
        s("nonempty")
    );
}
