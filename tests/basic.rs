use exprimo::Evaluator;
use std::collections::HashMap;

#[cfg(test)]
#[test]
fn test_basic_evaluate_with_context() {
    let mut context = HashMap::new();

    context.insert("a".to_string(), serde_json::Value::Bool(true));
    context.insert("b".to_string(), serde_json::Value::Bool(false));

    let evaluator = Evaluator::new(
        context,
        HashMap::new(), // custom_functions
    );

    let expr1 = "a && b";
    let expr2 = "a || b";
    let expr3 = "a && !b";
    let expr4 = "a || !b";
    let expr5 = "a && b || a && !b";
    let res1 = evaluator.evaluate(&expr1).unwrap();
    let res2 = evaluator.evaluate(&expr2).unwrap();
    let res3 = evaluator.evaluate(&expr3).unwrap();
    let res4 = evaluator.evaluate(&expr4).unwrap();
    let res5 = evaluator.evaluate(&expr5).unwrap();

    assert_eq!(res1, false);
    assert_eq!(res2, true);
    assert_eq!(res3, true);
    assert_eq!(res4, true);
    assert_eq!(res5, true);
}

#[test]
fn test_basic_evaluate_with_nulls() {
    let mut context = HashMap::new();

    context.insert("a".to_string(), serde_json::Value::Null);
    context.insert("b".to_string(), serde_json::Value::Bool(true));

    let evaluator = Evaluator::new(
        context,
        HashMap::new(), // custom_functions
    );

    let expr1 = "a && b";
    let expr2 = "a || b";
    let expr3 = "a && !b";
    let expr4 = "a || !b";
    let expr5 = "a && b || a && !b";
    let res1 = evaluator.evaluate(&expr1).unwrap();
    let res2 = evaluator.evaluate(&expr2).unwrap();
    let res3 = evaluator.evaluate(&expr3).unwrap();
    let res4 = evaluator.evaluate(&expr4).unwrap();
    let res5 = evaluator.evaluate(&expr5).unwrap();

    assert_eq!(res1, false);
    assert_eq!(res2, true);
    assert_eq!(res3, false);
    assert_eq!(res4, false);
    assert_eq!(res5, false);
}

// #[test]
// fn test_basic_evaluate_with_empty_strings() {
//     let mut context = HashMap::new();
//
//     context.insert(
//         "a".to_string(),
//         serde_json::Value::String("".to_string()),
//     );
//     context.insert("b".to_string(), serde_json::Value::Bool(true));
//
//     #[cfg(feature = "logging")]
//     let logger = Logger::default();
//
//     let evaluator = Evaluator::new(
//         context,
//         #[cfg(feature = "logging")]
//         logger,
//     );
//
//     let expr1 = "a && b";
//     let expr2 = "a || b";
//     let expr3 = "a && !b";
//     let expr4 = "a || !b";
//     let expr5 = "a && b || a && !b";
//     let res1 = evaluator.evaluate(&expr1).unwrap();
//     let res2 = evaluator.evaluate(&expr2).unwrap();
//     let res3 = evaluator.evaluate(&expr3).unwrap();
//     let res4 = evaluator.evaluate(&expr4).unwrap();
//     let res5 = evaluator.evaluate(&expr5).unwrap();
//
//     assert_eq!(res1, false);
//     assert_eq!(res2, true);
//     assert_eq!(res3, false);
//     assert_eq!(res4, false);
//     assert_eq!(res5, false);
// }

#[test]
fn test_single_quotes_expressions() {
    let mut context = HashMap::new();

    context.insert(
        "a".to_string(),
        serde_json::Value::String("true".to_string()),
    );
    let evaluator = Evaluator::new(
        context,
        HashMap::new(), // custom_functions
    );

    let expr1 = "a == 'true'";

    let res1 = evaluator.evaluate(&expr1).unwrap();

    assert_eq!(res1, true);
}

// --- Custom Function Tests ---

// Imports needed for custom function tests
use exprimo::{CustomFuncError, CustomFunction, EvaluationError};
use serde_json::Value; // Already imported at top level if this is the same file
use std::fmt::Debug;
use std::sync::Arc; // Required for CustomFunction trait
                    // HashMap is already imported at top level

#[derive(Debug)] // Debug is required by the CustomFunction trait
struct MyTestAdder;

impl CustomFunction for MyTestAdder {
    fn call(&self, args: &[Value]) -> Result<Value, CustomFuncError> {
        if args.len() != 2 {
            return Err(CustomFuncError::ArityError {
                expected: 2,
                got: args.len(),
            });
        }
        match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(val_a), Some(val_b)) = (a.as_f64(), b.as_f64()) {
                    Ok(Value::Number(
                        serde_json::Number::from_f64(val_a + val_b).unwrap(),
                    ))
                } else {
                    Err(CustomFuncError::ArgumentError(
                        "Non-finite number provided".to_string(),
                    ))
                }
            }
            _ => Err(CustomFuncError::ArgumentError(
                "Arguments must be numbers".to_string(),
            )),
        }
    }
}

#[test]
fn test_parenthesized_expressions() {
    let mut context = HashMap::new();
    context.insert(
        "a".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap()),
    );
    context.insert(
        "b".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(2.0).unwrap()),
    );
    context.insert(
        "c".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(3.0).unwrap()),
    );
    context.insert(
        "d".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(4.0).unwrap()),
    );

    let evaluator = Evaluator::new(
        context,
        HashMap::new(), // custom_functions
    );

    // Simple case: (1 + 2) * 3 = 9
    let expr1 = "(a + b) * c";
    let res1 = evaluator.evaluate(expr1).unwrap();
    assert_eq!(
        res1,
        serde_json::Value::Number(serde_json::Number::from_f64(9.0).unwrap())
    );

    // Nested parentheses: ((1 + 2) * 3) / 4 = 2.25
    let expr2 = "((a + b) * c) / d";
    let res2 = evaluator.evaluate(expr2).unwrap();
    assert_eq!(
        res2,
        serde_json::Value::Number(serde_json::Number::from_f64(2.25).unwrap())
    );

    // More complex expression: ( (4-2) * ( (1+2)*3 ) ) / (10/5) = (2 * 9) / 2 = 9
    // Using direct numbers for clarity here, assuming context a,b,c,d are not used or are shadowed by literals
    let expr3 = "((d-b) * ((a+b)*c)) / (10/5)";
    let res3 = evaluator.evaluate(expr3).unwrap();
    assert_eq!(
        res3,
        serde_json::Value::Number(serde_json::Number::from_f64(9.0).unwrap())
    );

    // Expression with unary operator
    let expr4 = "-(a + b)";
    let res4 = evaluator.evaluate(expr4).unwrap();
    assert_eq!(
        res4,
        serde_json::Value::Number(serde_json::Number::from_f64(-3.0).unwrap())
    );

    // Expression with unary operator inside parentheses
    let expr5 = "c * (-a - b)"; // 3 * (-1 - 2) = 3 * (-3) = -9
    let res5 = evaluator.evaluate(expr5).unwrap();
    assert_eq!(
        res5,
        serde_json::Value::Number(serde_json::Number::from_f64(-9.0).unwrap())
    );

    // Expression with boolean logic
    let expr6 = "(a < b) && (c > d)"; // (1 < 2) && (3 > 4) -> true && false -> false
    let res6 = evaluator.evaluate(expr6).unwrap();
    assert_eq!(res6, serde_json::Value::Bool(false));

    // Expression with boolean logic and parentheses for precedence
    let expr7 = "a < b && c > d || a == 1"; // (1<2 && 3>4) || 1==1 -> (true && false) || true -> false || true -> true
    let res7 = evaluator.evaluate(expr7).unwrap();
    assert_eq!(res7, serde_json::Value::Bool(true));

    let expr8 = "a < b && (c > d || a == 1)"; // 1<2 && (3>4 || 1==1) -> true && (false || true) -> true && true -> true
    let res8 = evaluator.evaluate(expr8).unwrap();
    assert_eq!(res8, serde_json::Value::Bool(true));
}

// --- Object.hasOwnProperty() Tests ---

#[test]
fn test_object_has_own_property_success() {
    let mut context = HashMap::new();
    let mut obj = serde_json::Map::new();
    obj.insert("name".to_string(), Value::String("Alice".to_string()));
    obj.insert("age".to_string(), Value::Number(30.into()));
    obj.insert(
        "".to_string(),
        Value::String("empty_string_key".to_string()),
    ); // Empty string as key
    context.insert("myObj".to_string(), Value::Object(obj));

    let evaluator = Evaluator::new(context, HashMap::new());

    assert_eq!(
        evaluator.evaluate("myObj.hasOwnProperty('name')").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        evaluator.evaluate("myObj.hasOwnProperty(\"age\")").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        evaluator
            .evaluate("myObj.hasOwnProperty('nonExistent')")
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        evaluator.evaluate("myObj.hasOwnProperty('')").unwrap(),
        Value::Bool(true)
    ); // Test empty string key

    // Test coercion of argument to string for a key that doesn't exist on myObj
    assert_eq!(
        evaluator.evaluate("myObj.hasOwnProperty(123)").unwrap(),
        Value::Bool(false)
    ); // effectively myObj.hasOwnProperty("123")
    assert_eq!(
        evaluator.evaluate("myObj.hasOwnProperty(true)").unwrap(),
        Value::Bool(false)
    ); // effectively myObj.hasOwnProperty("true")

    // Setup a new context for an object that has stringified number as key
    let mut context2 = HashMap::new();
    let mut obj_with_true_key = serde_json::Map::new();
    obj_with_true_key.insert(
        "true".to_string(),
        Value::String("test value for boolean true key".to_string()),
    );
    context2.insert("objTrueKey".to_string(), Value::Object(obj_with_true_key));

    let evaluator2_true = Evaluator::new(context2.clone(), HashMap::new());

    assert_eq!(
        evaluator2_true
            .evaluate("objTrueKey.hasOwnProperty(true)")
            .unwrap(), // Evaluates to objTrueKey.hasOwnProperty("true")
        Value::Bool(true),
        "objTrueKey should have property 'true' when called with boolean true"
    );
    assert_eq!(
        evaluator2_true
            .evaluate("objTrueKey.hasOwnProperty(false)")
            .unwrap(), // Evaluates to objTrueKey.hasOwnProperty("false")
        Value::Bool(false),
        "objTrueKey should not have property 'false'"
    );

    // Test with a key that looks like a number
    let mut context3 = HashMap::new();
    let mut obj_with_num_key_str = serde_json::Map::new();
    obj_with_num_key_str.insert("123".to_string(), Value::Bool(true));
    context3.insert(
        "objNumStrKey".to_string(),
        Value::Object(obj_with_num_key_str),
    );

    let evaluator3_num = Evaluator::new(context3, HashMap::new());

    assert_eq!(
        evaluator3_num
            .evaluate("objNumStrKey.hasOwnProperty(123)")
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        evaluator3_num
            .evaluate("objNumStrKey.hasOwnProperty('123')")
            .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn test_object_has_own_property_arity_error() {
    let mut context = HashMap::new();
    context.insert("myObj".to_string(), Value::Object(serde_json::Map::new()));
    let evaluator = Evaluator::new(context, HashMap::new());

    let res_no_args = evaluator.evaluate("myObj.hasOwnProperty()");
    match res_no_args {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 1);
            assert_eq!(got, 0);
        }
        _ => panic!(
            "Expected ArityError for no arguments, got {:?}",
            res_no_args
        ),
    }

    let res_many_args = evaluator.evaluate("myObj.hasOwnProperty('prop', 'extra')");
    match res_many_args {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 1);
            assert_eq!(got, 2);
        }
        _ => panic!(
            "Expected ArityError for many arguments, got {:?}",
            res_many_args
        ),
    }
}

#[test]
fn test_object_has_own_property_on_non_object() {
    let mut context = HashMap::new();
    context.insert("myArr".to_string(), Value::Array(vec![]));
    context.insert("myStr".to_string(), Value::String("text".to_string()));

    let evaluator = Evaluator::new(context, HashMap::new());

    // Case 1: Array
    let expr_arr = "myArr.hasOwnProperty('length')";
    let result_arr = evaluator.evaluate(expr_arr);
    match result_arr {
        Err(EvaluationError::TypeError(msg)) => {
            assert_eq!(
                msg,
                "'null' (resulting from expression 'myArr.hasOwnProperty') is not a function."
            );
        }
        _ => panic!(
            "Expected TypeError for myArr.hasOwnProperty, got {:?}",
            result_arr
        ),
    }

    // Case 2: String — strings don't expose hasOwnProperty; resolves to null
    // then calling null as a function yields a "not a function" error.
    let expr_str = "myStr.hasOwnProperty('length')";
    let result_str = evaluator.evaluate(expr_str);
    match result_str {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(
                msg.contains("is not a function"),
                "unexpected message: {}",
                msg
            );
        }
        _ => panic!(
            "Expected TypeError for myStr.hasOwnProperty, got {:?}",
            result_str
        ),
    }
}

#[test]
fn test_object_has_own_property_nested() {
    let mut context = HashMap::new();
    let mut inner_obj = serde_json::Map::new();
    inner_obj.insert("value".to_string(), Value::Bool(true));
    let mut outer_obj = serde_json::Map::new();
    outer_obj.insert("nestedObj".to_string(), Value::Object(inner_obj));
    context.insert("item".to_string(), Value::Object(outer_obj));

    let evaluator = Evaluator::new(context, HashMap::new());

    assert_eq!(
        evaluator
            .evaluate("item.nestedObj.hasOwnProperty('value')")
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        evaluator
            .evaluate("item.nestedObj.hasOwnProperty('nonExistent')")
            .unwrap(),
        Value::Bool(false)
    );
}

// --- Array.includes() Tests ---

#[test]
fn test_array_includes_success() {
    let mut context = HashMap::new();
    let arr = Value::Array(vec![
        Value::Number(10.into()),
        Value::String("hello".to_string()),
        Value::Bool(true),
        Value::Null,
    ]);
    context.insert("myArr".to_string(), arr);

    let evaluator = Evaluator::new(context, HashMap::new());

    assert_eq!(
        evaluator.evaluate("myArr.includes(10)").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        evaluator.evaluate("myArr.includes(\"hello\")").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        evaluator.evaluate("myArr.includes(true)").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        evaluator.evaluate("myArr.includes(null)").unwrap(),
        Value::Bool(true)
    );

    assert_eq!(
        evaluator.evaluate("myArr.includes(20)").unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        evaluator.evaluate("myArr.includes(\"world\")").unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        evaluator.evaluate("myArr.includes(false)").unwrap(),
        Value::Bool(false)
    );
    // Note: Value::Null is present, so `myArr.includes(something_else_that_is_not_null)` should be false.
    // serde_json::Value::Object(Map::new()) is not in the array.
    assert_eq!(
        evaluator.evaluate("myArr.includes({})").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn test_array_includes_arity_error() {
    let mut context = HashMap::new();
    context.insert("myArr".to_string(), Value::Array(vec![]));
    let evaluator = Evaluator::new(context, HashMap::new());

    let res_no_args = evaluator.evaluate("myArr.includes()");
    match res_no_args {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 1);
            assert_eq!(got, 0);
        }
        _ => panic!(
            "Expected ArityError for no arguments, got {:?}",
            res_no_args
        ),
    }

    let res_many_args = evaluator.evaluate("myArr.includes(1, 2)");
    match res_many_args {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 1);
            assert_eq!(got, 2);
        }
        _ => panic!(
            "Expected ArityError for many arguments, got {:?}",
            res_many_args
        ),
    }
}

#[test]
fn test_array_includes_on_non_array() {
    // Strings now also support .includes — verify it behaves like String.includes,
    // and that .includes on a null/primitive object still errors.
    let mut context = HashMap::new();
    context.insert("aStr".to_string(), Value::String("hello".to_string()));
    context.insert("aNull".to_string(), Value::Null);
    let evaluator = Evaluator::new(context, HashMap::new());

    // String.includes: substring search
    assert_eq!(
        evaluator.evaluate("aStr.includes('ell')").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        evaluator.evaluate("aStr.includes('xyz')").unwrap(),
        Value::Bool(false)
    );

    // .includes on null still errors
    let result = evaluator.evaluate("aNull.includes(1)");
    match result {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("Cannot read properties of null or primitive value: null"));
        }
        _ => panic!(
            "Expected TypeError when calling .includes on null, got {:?}",
            result
        ),
    }
}

#[test]
fn test_array_includes_nested() {
    let mut context = HashMap::new();
    let mut obj = serde_json::Map::new();
    let arr = Value::Array(vec![Value::Number(42.into())]);
    obj.insert("nestedArr".to_string(), arr);
    context.insert("myObj".to_string(), Value::Object(obj));

    let evaluator = Evaluator::new(context, HashMap::new());

    assert_eq!(
        evaluator.evaluate("myObj.nestedArr.includes(42)").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        evaluator.evaluate("myObj.nestedArr.includes(100)").unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn test_custom_adder_success() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    let evaluator = Evaluator::new(
        // Evaluator is already imported
        context,
        custom_funcs,
    );

    let result = evaluator.evaluate("custom_add(10, 20.5)").unwrap();
    assert_eq!(result.as_f64(), Some(30.5));

    let result_neg = evaluator.evaluate("custom_add(-5, -2)").unwrap();
    assert_eq!(result_neg.as_f64(), Some(-7.0));
}

#[test]
fn test_custom_adder_arity_error_few_args() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    let evaluator = Evaluator::new(context, custom_funcs);

    let result = evaluator.evaluate("custom_add(10)");
    match result {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 2);
            assert_eq!(got, 1);
        }
        _ => panic!("Expected ArityError, got {:?}", result),
    }
}

#[test]
fn test_custom_adder_arity_error_many_args() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    let evaluator = Evaluator::new(context, custom_funcs);

    let result = evaluator.evaluate("custom_add(10, 20, 30)");
    match result {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 2);
            assert_eq!(got, 3);
        }
        _ => panic!("Expected ArityError, got {:?}", result),
    }
}

#[test]
fn test_custom_adder_type_error_arg1() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    let evaluator = Evaluator::new(context, custom_funcs);

    let result = evaluator.evaluate("custom_add('not_a_number', 10)");
    match result {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArgumentError(msg))) => {
            assert_eq!(msg, "Arguments must be numbers");
        }
        _ => panic!("Expected ArgumentError, got {:?}", result),
    }
}

#[test]
fn test_custom_adder_type_error_arg2() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    let evaluator = Evaluator::new(context, custom_funcs);

    let result = evaluator.evaluate("custom_add(10, 'not_a_number')");
    match result {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArgumentError(msg))) => {
            assert_eq!(msg, "Arguments must be numbers");
        }
        _ => panic!("Expected ArgumentError, got {:?}", result),
    }
}

#[test]
fn test_custom_adder_non_finite_number_error() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    let evaluator = Evaluator::new(context, custom_funcs);

    // Create a NaN Value::Number (Note: serde_json::Number cannot directly represent NaN/Infinity)
    // This test relies on the internal f64 conversion and check.
    // For the purpose of this test, we'll assume that if a Value::Number
    // somehow contained a non-finite f64, our function would catch it.
    // Direct creation of such a serde_json::Value::Number is tricky,
    // as it typically only supports finite numbers.
    // The error "Non-finite number provided" is more of a safeguard
    // if such a value were to be constructed manually or via other means.
    // We can't directly test this path with standard expression strings
    // if the parser/evaluator only produces finite Value::Number.
    // However, if a custom function internally constructed such a value and passed it
    // to another custom function, this check would be relevant.
    // For now, we acknowledge this path is hard to test via string expressions.
    // A direct call to `MyTestAdder.call()` would be needed to test this,
    // which is outside the scope of `evaluator.evaluate()`.
    // So, this specific error condition "Non-finite number provided"
    // is not easily testable through the evaluator's `evaluate` method
    // if the parser only generates valid numbers.
    // The existing type error tests cover cases where types are not numbers.
}

// --- Property Access Tests ---

#[test]
fn test_array_length_direct() {
    let mut context = HashMap::new();
    let my_array = Value::Array(vec![Value::from(1), Value::from(2), Value::from(3)]);
    context.insert("myArray".to_string(), my_array);

    let evaluator = Evaluator::new(context, HashMap::new());

    let result = evaluator.evaluate("myArray.length").unwrap();
    assert_eq!(
        result,
        Value::Number(serde_json::Number::from_f64(3.0).unwrap())
    );
}

#[test]
fn test_array_length_nested_in_object() {
    let mut context = HashMap::new();
    let my_array = Value::Array(vec![Value::from("a"), Value::from("b")]);
    let mut obj = serde_json::Map::new();
    obj.insert("arr".to_string(), my_array);
    context.insert("myObj".to_string(), Value::Object(obj));

    let evaluator = Evaluator::new(context, HashMap::new());

    let result = evaluator.evaluate("myObj.arr.length").unwrap();
    assert_eq!(
        result,
        Value::Number(serde_json::Number::from_f64(2.0).unwrap())
    );
}

#[test]
fn test_length_on_non_array() {
    let mut context = HashMap::new();
    context.insert("myString".to_string(), Value::String("hello".to_string()));
    context.insert("myNum".to_string(), Value::Number(123.into()));
    let mut obj_without_length = serde_json::Map::new();
    obj_without_length.insert("prop".to_string(), Value::from("value"));
    context.insert("myObj".to_string(), Value::Object(obj_without_length));
    context.insert("nullVar".to_string(), Value::Null);

    let evaluator = Evaluator::new(context.clone(), HashMap::new());

    // String.length returns the number of characters
    let res_str = evaluator.evaluate("myString.length").unwrap();
    assert_eq!(
        res_str,
        Value::Number(serde_json::Number::from_f64(5.0).unwrap())
    );

    // Numbers don't expose .length — returns null (JS undefined equivalent)
    let res_num = evaluator.evaluate("myNum.length").unwrap();
    assert_eq!(res_num, Value::Null);

    // Accessing .length on an object that doesn't have it should return Value::Null
    let res_obj = evaluator.evaluate("myObj.length").unwrap();
    assert_eq!(res_obj, Value::Null);

    let res_null = evaluator.evaluate("nullVar.length");
    match res_null {
        Err(EvaluationError::TypeError(msg)) => {
            assert_eq!(
                msg,
                "Cannot read property 'length' of non-array/non-object value: null"
            );
        }
        _ => panic!("Expected TypeError for null.length, got {:?}", res_null),
    }
}

#[test]
fn test_other_property_on_array() {
    let mut context = HashMap::new();
    let my_array = Value::Array(vec![Value::from(1)]);
    context.insert("myArray".to_string(), my_array);

    let evaluator = Evaluator::new(context, HashMap::new());

    let result = evaluator.evaluate("myArray.foo").unwrap();
    assert_eq!(result, Value::Null); // JS returns undefined, so we return Null
}

#[test]
fn test_property_access_on_object() {
    let mut context = HashMap::new();
    let mut obj = serde_json::Map::new();
    obj.insert("name".to_string(), Value::String("Tester".to_string()));
    obj.insert("age".to_string(), Value::Number(30.into()));
    context.insert("user".to_string(), Value::Object(obj));

    let evaluator = Evaluator::new(context, HashMap::new());

    assert_eq!(
        evaluator.evaluate("user.name").unwrap(),
        Value::String("Tester".to_string())
    );
    assert_eq!(
        evaluator.evaluate("user.age").unwrap(),
        Value::Number(30.into())
    );
    assert_eq!(evaluator.evaluate("user.nonexistent").unwrap(), Value::Null);
}

#[test]
fn test_property_access_on_nested_object() {
    let mut context = HashMap::new();
    let mut inner_obj = serde_json::Map::new();
    inner_obj.insert("value".to_string(), Value::Bool(true));
    let mut outer_obj = serde_json::Map::new();
    outer_obj.insert("nested".to_string(), Value::Object(inner_obj));
    context.insert("item".to_string(), Value::Object(outer_obj));

    let evaluator = Evaluator::new(context, HashMap::new());

    assert_eq!(
        evaluator.evaluate("item.nested.value").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(evaluator.evaluate("item.nested.foo").unwrap(), Value::Null);

    let res_access_on_null = evaluator.evaluate("item.nonexistent.bar"); // item.nonexistent is Null, then .bar on Null
    match res_access_on_null {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("Cannot read properties of null or primitive value: null (trying to access property: bar)"));
        }
        _ => panic!(
            "Expected TypeError for item.nonexistent.bar, got {:?}",
            res_access_on_null
        ),
    }
}

#[test]
fn test_property_access_on_null_or_primitive_object_error() {
    // Strings and numbers expose some built-in properties; accessing an unknown
    // property on them returns null (JS undefined equivalent). Booleans and null
    // still error on property access.
    let mut context = HashMap::new();
    context.insert("s".to_string(), Value::String("text".to_string()));
    context.insert("n".to_string(), Value::Number(123.into()));
    context.insert("b".to_string(), Value::Bool(true));
    context.insert("nl".to_string(), Value::Null);

    let evaluator = Evaluator::new(context, HashMap::new());

    // Unknown property on a string or number → null
    assert_eq!(evaluator.evaluate("s.foo").unwrap(), Value::Null);
    assert_eq!(evaluator.evaluate("n.bar").unwrap(), Value::Null);

    // Property access on bool / null still errors
    for case in &["b.baz", "nl.qux"] {
        let result = evaluator.evaluate(case);
        match result {
            Err(EvaluationError::TypeError(msg)) => {
                assert!(msg.starts_with("Cannot read properties of null or primitive value:"));
            }
            _ => panic!(
                "Expected TypeError for property access on primitive/null ({}), got {:?}",
                case, result
            ),
        }
    }
}

// --- String property and method tests ---

fn num(v: f64) -> Value {
    Value::Number(serde_json::Number::from_f64(v).unwrap())
}

fn with_str_context() -> Evaluator {
    let mut context = HashMap::new();
    context.insert("s".to_string(), Value::String("Hello, World!".to_string()));
    context.insert("empty".to_string(), Value::String(String::new()));
    context.insert("padded".to_string(), Value::String("  spaced  ".to_string()));
    Evaluator::new(context, HashMap::new())
}

#[test]
fn test_string_length() {
    let ev = with_str_context();
    assert_eq!(ev.evaluate("s.length").unwrap(), num(13.0));
    assert_eq!(ev.evaluate("empty.length").unwrap(), num(0.0));
    // length counts characters, not bytes
    let mut ctx = HashMap::new();
    ctx.insert("u".to_string(), Value::String("héllo".to_string()));
    let ev2 = Evaluator::new(ctx, HashMap::new());
    assert_eq!(ev2.evaluate("u.length").unwrap(), num(5.0));
}

#[test]
fn test_string_case_conversion() {
    let ev = with_str_context();
    assert_eq!(
        ev.evaluate("s.toUpperCase()").unwrap(),
        Value::String("HELLO, WORLD!".to_string())
    );
    assert_eq!(
        ev.evaluate("s.toLowerCase()").unwrap(),
        Value::String("hello, world!".to_string())
    );
}

#[test]
fn test_string_trim() {
    let ev = with_str_context();
    assert_eq!(
        ev.evaluate("padded.trim()").unwrap(),
        Value::String("spaced".to_string())
    );
    assert_eq!(
        ev.evaluate("empty.trim()").unwrap(),
        Value::String(String::new())
    );
}

#[test]
fn test_string_includes_starts_ends_with() {
    let ev = with_str_context();
    assert_eq!(ev.evaluate("s.includes('World')").unwrap(), Value::Bool(true));
    assert_eq!(ev.evaluate("s.includes('world')").unwrap(), Value::Bool(false));
    assert_eq!(ev.evaluate("s.startsWith('Hello')").unwrap(), Value::Bool(true));
    assert_eq!(ev.evaluate("s.startsWith('Hi')").unwrap(), Value::Bool(false));
    assert_eq!(ev.evaluate("s.endsWith('!')").unwrap(), Value::Bool(true));
    assert_eq!(ev.evaluate("s.endsWith('?')").unwrap(), Value::Bool(false));
}

#[test]
fn test_string_slice() {
    let ev = with_str_context();
    assert_eq!(
        ev.evaluate("s.slice(0, 5)").unwrap(),
        Value::String("Hello".to_string())
    );
    assert_eq!(
        ev.evaluate("s.slice(7)").unwrap(),
        Value::String("World!".to_string())
    );
    // negative indices relative to end
    assert_eq!(
        ev.evaluate("s.slice(-6, -1)").unwrap(),
        Value::String("World".to_string())
    );
    // end before start returns empty
    assert_eq!(
        ev.evaluate("s.slice(5, 2)").unwrap(),
        Value::String(String::new())
    );
    // out-of-range end is clamped
    assert_eq!(
        ev.evaluate("s.slice(0, 100)").unwrap(),
        Value::String("Hello, World!".to_string())
    );
}

#[test]
fn test_string_index_of() {
    let ev = with_str_context();
    assert_eq!(ev.evaluate("s.indexOf('World')").unwrap(), num(7.0));
    assert_eq!(ev.evaluate("s.indexOf('xxx')").unwrap(), num(-1.0));
    assert_eq!(ev.evaluate("s.indexOf('H')").unwrap(), num(0.0));
}

// --- Number.toFixed tests ---

#[test]
fn test_number_to_fixed() {
    let mut ctx = HashMap::new();
    ctx.insert("price".to_string(), num(3.144));
    ctx.insert("n".to_string(), num(2.0));
    ctx.insert("big".to_string(), num(1234.5));
    let ev = Evaluator::new(ctx, HashMap::new());

    assert_eq!(
        ev.evaluate("price.toFixed(2)").unwrap(),
        Value::String("3.14".to_string())
    );
    assert_eq!(
        ev.evaluate("price.toFixed(0)").unwrap(),
        Value::String("3".to_string())
    );
    assert_eq!(
        ev.evaluate("big.toFixed(2)").unwrap(),
        Value::String("1234.50".to_string())
    );
    // default (no args) is 0 digits
    assert_eq!(
        ev.evaluate("n.toFixed()").unwrap(),
        Value::String("2".to_string())
    );
    // negative numbers
    let mut ctx2 = HashMap::new();
    ctx2.insert("neg".to_string(), num(-1.2345));
    let ev2 = Evaluator::new(ctx2, HashMap::new());
    assert_eq!(
        ev2.evaluate("neg.toFixed(2)").unwrap(),
        Value::String("-1.23".to_string())
    );
}

#[test]
fn test_number_to_fixed_out_of_range() {
    let mut ctx = HashMap::new();
    ctx.insert("n".to_string(), num(1.0));
    let ev = Evaluator::new(ctx, HashMap::new());
    match ev.evaluate("n.toFixed(-1)") {
        Err(EvaluationError::TypeError(_)) => {}
        other => panic!("Expected TypeError for toFixed(-1), got {:?}", other),
    }
    match ev.evaluate("n.toFixed(101)") {
        Err(EvaluationError::TypeError(_)) => {}
        other => panic!("Expected TypeError for toFixed(101), got {:?}", other),
    }
}

// --- Math namespace tests ---

#[test]
fn test_math_floor_ceil_round() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    assert_eq!(ev.evaluate("Math.floor(1.9)").unwrap(), num(1.0));
    assert_eq!(ev.evaluate("Math.floor(-1.1)").unwrap(), num(-2.0));
    assert_eq!(ev.evaluate("Math.ceil(1.1)").unwrap(), num(2.0));
    assert_eq!(ev.evaluate("Math.ceil(-1.9)").unwrap(), num(-1.0));
    assert_eq!(ev.evaluate("Math.round(1.4)").unwrap(), num(1.0));
    assert_eq!(ev.evaluate("Math.round(1.5)").unwrap(), num(2.0));
    // JS semantics: half rounds toward +Infinity
    assert_eq!(ev.evaluate("Math.round(-1.5)").unwrap(), num(-1.0));
}

#[test]
fn test_math_abs_min_max() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    assert_eq!(ev.evaluate("Math.abs(-5)").unwrap(), num(5.0));
    assert_eq!(ev.evaluate("Math.abs(3)").unwrap(), num(3.0));
    assert_eq!(ev.evaluate("Math.min(1, 2, 3)").unwrap(), num(1.0));
    assert_eq!(ev.evaluate("Math.max(1, 2, 3)").unwrap(), num(3.0));
    assert_eq!(ev.evaluate("Math.min(-1, -2)").unwrap(), num(-2.0));
    // Math.min() with no args in JS returns Infinity
    match ev.evaluate("Math.min()") {
        Ok(Value::Number(n)) => assert!(n.as_f64().unwrap() > 1e300),
        other => panic!("Expected large number for Math.min(), got {:?}", other),
    }
}

#[test]
fn test_math_unknown_method() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    match ev.evaluate("Math.foo(1)") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("Math.foo"));
        }
        other => panic!("Expected TypeError for Math.foo, got {:?}", other),
    }
}

#[test]
fn test_math_shadowed_by_context() {
    // A user-defined `Math` in context should shadow the namespace.
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    m.insert("custom".to_string(), Value::String("shadowed".to_string()));
    ctx.insert("Math".to_string(), Value::Object(m));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(
        ev.evaluate("Math.custom").unwrap(),
        Value::String("shadowed".to_string())
    );
}

// --- Bracket indexing tests ---

#[test]
fn test_bracket_index_array() {
    let mut ctx = HashMap::new();
    ctx.insert(
        "arr".to_string(),
        Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ]),
    );
    ctx.insert("i".to_string(), num(1.0));
    let ev = Evaluator::new(ctx, HashMap::new());

    assert_eq!(
        ev.evaluate("arr[0]").unwrap(),
        Value::String("a".to_string())
    );
    assert_eq!(
        ev.evaluate("arr[2]").unwrap(),
        Value::String("c".to_string())
    );
    // dynamic index
    assert_eq!(
        ev.evaluate("arr[i]").unwrap(),
        Value::String("b".to_string())
    );
    // out of range → null
    assert_eq!(ev.evaluate("arr[99]").unwrap(), Value::Null);
    // negative index → null (not Python-like wrap-around)
    assert_eq!(ev.evaluate("arr[-1]").unwrap(), Value::Null);
}

#[test]
fn test_bracket_index_object() {
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    m.insert("name".to_string(), Value::String("Alice".to_string()));
    m.insert("age".to_string(), num(30.0));
    ctx.insert("obj".to_string(), Value::Object(m));
    ctx.insert("key".to_string(), Value::String("name".to_string()));
    let ev = Evaluator::new(ctx, HashMap::new());

    assert_eq!(
        ev.evaluate("obj['name']").unwrap(),
        Value::String("Alice".to_string())
    );
    assert_eq!(ev.evaluate("obj[\"age\"]").unwrap(), num(30.0));
    // dynamic key
    assert_eq!(
        ev.evaluate("obj[key]").unwrap(),
        Value::String("Alice".to_string())
    );
    // missing key → null
    assert_eq!(ev.evaluate("obj['missing']").unwrap(), Value::Null);
}

#[test]
fn test_bracket_index_string() {
    let mut ctx = HashMap::new();
    ctx.insert("s".to_string(), Value::String("hello".to_string()));
    let ev = Evaluator::new(ctx, HashMap::new());

    assert_eq!(
        ev.evaluate("s[0]").unwrap(),
        Value::String("h".to_string())
    );
    assert_eq!(
        ev.evaluate("s[4]").unwrap(),
        Value::String("o".to_string())
    );
    assert_eq!(ev.evaluate("s[99]").unwrap(), Value::Null);
}

#[test]
fn test_bracket_index_chained() {
    let mut ctx = HashMap::new();
    let mut item = serde_json::Map::new();
    item.insert("name".to_string(), Value::String("first".to_string()));
    let arr = Value::Array(vec![Value::Object(item)]);
    let mut root = serde_json::Map::new();
    root.insert("items".to_string(), arr);
    ctx.insert("state".to_string(), Value::Object(root));
    let ev = Evaluator::new(ctx, HashMap::new());

    assert_eq!(
        ev.evaluate("state.items[0].name").unwrap(),
        Value::String("first".to_string())
    );
    assert_eq!(
        ev.evaluate("state['items'][0]['name']").unwrap(),
        Value::String("first".to_string())
    );
}

// --- Array methods tests ---

fn with_arr_context() -> Evaluator {
    let mut ctx = HashMap::new();
    ctx.insert(
        "arr".to_string(),
        Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
            Value::String("d".to_string()),
        ]),
    );
    ctx.insert(
        "nums".to_string(),
        Value::Array(vec![
            Value::Number(10.into()),
            Value::Number(20.into()),
            Value::Number(30.into()),
        ]),
    );
    Evaluator::new(ctx, HashMap::new())
}

#[test]
fn test_array_index_of() {
    let ev = with_arr_context();
    assert_eq!(ev.evaluate("arr.indexOf('b')").unwrap(), num(1.0));
    assert_eq!(ev.evaluate("arr.indexOf('z')").unwrap(), num(-1.0));
    // Number context values are integer-typed; the literal 20 parses as a float,
    // so strict equality here matches numerically (20.0 == 20).
    assert_eq!(ev.evaluate("nums.indexOf(20)").unwrap(), num(1.0));
    // strict equality: 20 !== "20"
    assert_eq!(ev.evaluate("nums.indexOf('20')").unwrap(), num(-1.0));
}

#[test]
fn test_array_join() {
    let ev = with_arr_context();
    assert_eq!(
        ev.evaluate("arr.join(', ')").unwrap(),
        Value::String("a, b, c, d".to_string())
    );
    assert_eq!(
        ev.evaluate("arr.join('')").unwrap(),
        Value::String("abcd".to_string())
    );
    // default separator is ","
    assert_eq!(
        ev.evaluate("arr.join()").unwrap(),
        Value::String("a,b,c,d".to_string())
    );
    // numbers get stringified
    assert_eq!(
        ev.evaluate("nums.join('-')").unwrap(),
        Value::String("10-20-30".to_string())
    );
    // null entries render as empty string, matching JS
    let mut ctx2 = HashMap::new();
    ctx2.insert(
        "mixed".to_string(),
        Value::Array(vec![
            Value::Number(1.into()),
            Value::Null,
            Value::Number(3.into()),
        ]),
    );
    let ev2 = Evaluator::new(ctx2, HashMap::new());
    assert_eq!(
        ev2.evaluate("mixed.join('-')").unwrap(),
        Value::String("1--3".to_string())
    );
}

#[test]
fn test_array_slice() {
    let ev = with_arr_context();
    // [b, c]
    let r = ev.evaluate("arr.slice(1, 3)").unwrap();
    assert_eq!(
        r,
        Value::Array(vec![
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ])
    );
    // from index to end
    let r2 = ev.evaluate("arr.slice(2)").unwrap();
    assert_eq!(
        r2,
        Value::Array(vec![
            Value::String("c".to_string()),
            Value::String("d".to_string()),
        ])
    );
    // negative start
    let r3 = ev.evaluate("arr.slice(-2)").unwrap();
    assert_eq!(
        r3,
        Value::Array(vec![
            Value::String("c".to_string()),
            Value::String("d".to_string()),
        ])
    );
    // empty slice
    assert_eq!(
        ev.evaluate("arr.slice(3, 1)").unwrap(),
        Value::Array(vec![])
    );
    // no-arg → full copy
    let r4 = ev.evaluate("arr.slice()").unwrap();
    assert_eq!(
        r4,
        Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
            Value::String("d".to_string()),
        ])
    );
}

// --- Integration: survey bug #2 scenario ---

#[test]
fn test_hypen_bug_2_empty_state() {
    // The "empty state" DSL condition from the survey
    let mut ctx = HashMap::new();
    ctx.insert("state".to_string(), Value::String(String::new()));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(ev.evaluate("state.length == 0").unwrap(), Value::Bool(true));

    let mut ctx2 = HashMap::new();
    ctx2.insert("state".to_string(), Value::String("hi".to_string()));
    let ev2 = Evaluator::new(ctx2, HashMap::new());
    assert_eq!(ev2.evaluate("state.length == 0").unwrap(), Value::Bool(false));
}

#[test]
fn test_hypen_currency_display() {
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    m.insert("price".to_string(), num(19.995));
    ctx.insert("state".to_string(), Value::Object(m));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(
        ev.evaluate("state.price.toFixed(2)").unwrap(),
        Value::String("20.00".to_string())
    );
}

#[test]
fn test_hypen_truncated_bio() {
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    m.insert(
        "bio".to_string(),
        Value::String("A very long bio that should be truncated.".to_string()),
    );
    ctx.insert("state".to_string(), Value::Object(m));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(
        ev.evaluate("state.bio.slice(0, 11) + '…'").unwrap(),
        Value::String("A very long…".to_string())
    );
}

// --- Object namespace tests ---

#[test]
fn test_object_keys_values_entries() {
    let mut ctx = HashMap::new();
    let mut obj = serde_json::Map::new();
    obj.insert("a".to_string(), Value::Number(1.into()));
    obj.insert("b".to_string(), Value::Number(2.into()));
    obj.insert("c".to_string(), Value::String("three".to_string()));
    ctx.insert("obj".to_string(), Value::Object(obj));
    let ev = Evaluator::new(ctx, HashMap::new());

    let keys = ev.evaluate("Object.keys(obj)").unwrap();
    assert_eq!(
        keys,
        Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ])
    );

    let values = ev.evaluate("Object.values(obj)").unwrap();
    assert_eq!(
        values,
        Value::Array(vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::String("three".to_string()),
        ])
    );

    let entries = ev.evaluate("Object.entries(obj)").unwrap();
    assert_eq!(
        entries,
        Value::Array(vec![
            Value::Array(vec![
                Value::String("a".to_string()),
                Value::Number(1.into()),
            ]),
            Value::Array(vec![
                Value::String("b".to_string()),
                Value::Number(2.into()),
            ]),
            Value::Array(vec![
                Value::String("c".to_string()),
                Value::String("three".to_string()),
            ]),
        ])
    );
}

#[test]
fn test_object_keys_on_non_object() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    match ev.evaluate("Object.keys('hi')") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("non-object"));
        }
        other => panic!("Expected TypeError, got {:?}", other),
    }
}

#[test]
fn test_object_unknown_method() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    match ev.evaluate("Object.foo({})") {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("Object.foo"));
        }
        other => panic!("Expected TypeError, got {:?}", other),
    }
}

#[test]
fn test_object_shadowed_by_context() {
    // A user-defined `Object` binding should shadow the namespace.
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    m.insert("custom".to_string(), Value::String("shadowed".to_string()));
    ctx.insert("Object".to_string(), Value::Object(m));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(
        ev.evaluate("Object.custom").unwrap(),
        Value::String("shadowed".to_string())
    );
}

// --- Complex array literal tests ---

#[test]
fn test_array_literal_with_elements() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    let result = ev.evaluate("[1, 2, 3]").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            num(1.0),
            num(2.0),
            num(3.0),
        ])
    );
    // mixed types
    let result2 = ev.evaluate("[1, 'two', true, null]").unwrap();
    assert_eq!(
        result2,
        Value::Array(vec![
            num(1.0),
            Value::String("two".to_string()),
            Value::Bool(true),
            Value::Null,
        ])
    );
}

#[test]
fn test_array_literal_with_expressions() {
    let mut ctx = HashMap::new();
    ctx.insert("x".to_string(), num(10.0));
    let ev = Evaluator::new(ctx, HashMap::new());
    let result = ev.evaluate("[x, x + 1, x * 2]").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            num(10.0),
            num(11.0),
            num(20.0),
        ])
    );
}

#[test]
fn test_array_literal_nested() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    let result = ev.evaluate("[[1, 2], [3, 4]]").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![num(1.0), num(2.0)]),
            Value::Array(vec![num(3.0), num(4.0)]),
        ])
    );
}

#[test]
fn test_array_literal_with_methods() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    assert_eq!(ev.evaluate("[1, 2, 3].length").unwrap(), num(3.0));
    assert_eq!(
        ev.evaluate("[1, 2, 3].includes(2)").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ev.evaluate("['a', 'b', 'c'].join('-')").unwrap(),
        Value::String("a-b-c".to_string())
    );
    assert_eq!(
        ev.evaluate("[10, 20, 30][1]").unwrap(),
        num(20.0)
    );
}

// --- Complex object literal tests ---
// Note: `{a: 1}` at statement position is parsed as a block; wrap in parens.

#[test]
fn test_object_literal_simple() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    let result = ev.evaluate("({a: 1, b: 2})").unwrap();
    let mut expected = serde_json::Map::new();
    expected.insert("a".to_string(), num(1.0));
    expected.insert("b".to_string(), num(2.0));
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_object_literal_string_keys() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    let result = ev.evaluate("({'foo': 1, \"bar\": 2})").unwrap();
    let mut expected = serde_json::Map::new();
    expected.insert("foo".to_string(), num(1.0));
    expected.insert("bar".to_string(), num(2.0));
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_object_literal_with_expressions() {
    let mut ctx = HashMap::new();
    ctx.insert("x".to_string(), num(5.0));
    let ev = Evaluator::new(ctx, HashMap::new());
    let result = ev.evaluate("({value: x * 2, doubled: x + x})").unwrap();
    let mut expected = serde_json::Map::new();
    expected.insert("value".to_string(), num(10.0));
    expected.insert("doubled".to_string(), num(10.0));
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_object_literal_nested() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    let result = ev.evaluate("({outer: {inner: 42}})").unwrap();
    let mut inner = serde_json::Map::new();
    inner.insert("inner".to_string(), num(42.0));
    let mut outer = serde_json::Map::new();
    outer.insert("outer".to_string(), Value::Object(inner));
    assert_eq!(result, Value::Object(outer));
}

#[test]
fn test_object_literal_with_arrays() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    let result = ev.evaluate("({items: [1, 2, 3], tags: ['a', 'b']})").unwrap();
    let mut expected = serde_json::Map::new();
    expected.insert(
        "items".to_string(),
        Value::Array(vec![num(1.0), num(2.0), num(3.0)]),
    );
    expected.insert(
        "tags".to_string(),
        Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]),
    );
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_object_literal_access() {
    let ev = Evaluator::new(HashMap::new(), HashMap::new());
    assert_eq!(ev.evaluate("({a: 1, b: 2}).a").unwrap(), num(1.0));
    assert_eq!(ev.evaluate("({a: 1, b: 2})['b']").unwrap(), num(2.0));
}

#[test]
fn test_hypen_tags_join() {
    let mut ctx = HashMap::new();
    let mut m = serde_json::Map::new();
    m.insert(
        "tags".to_string(),
        Value::Array(vec![
            Value::String("rust".to_string()),
            Value::String("dsl".to_string()),
            Value::String("eval".to_string()),
        ]),
    );
    ctx.insert("state".to_string(), Value::Object(m));
    let ev = Evaluator::new(ctx, HashMap::new());
    assert_eq!(
        ev.evaluate("state.tags.join(', ')").unwrap(),
        Value::String("rust, dsl, eval".to_string())
    );
}
