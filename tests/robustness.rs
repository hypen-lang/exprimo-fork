use exprimo::Evaluator;
use serde_json::Value;
use std::collections::HashMap;

#[test]
fn test_overflow_addition() {
    let evaluator = Evaluator::new(HashMap::new(), HashMap::new());
    // 1.7976931348623157e308 is approx f64::MAX
    let result = evaluator.evaluate("1.7976931348623157e308 + 1.7976931348623157e308");
    match result {
        Ok(Value::Number(n)) => {
            // Should be f64::MAX (Infinity approximation)
            assert!(n.as_f64().unwrap() > 1.0e308);
        }
        Ok(val) => panic!("Expected Number, got {:?}", val),
        Err(e) => panic!("Evaluation failed: {:?}", e),
    }
}

#[test]
fn test_divide_by_zero() {
    let evaluator = Evaluator::new(HashMap::new(), HashMap::new());
    let result = evaluator.evaluate("1 / 0");
    match result {
        Ok(Value::Number(n)) => {
            // Should be f64::MAX (Infinity approximation) or Infinity
            // Currently it might be returning 0.0 based on previous analysis, we want it to be MAX
            let val = n.as_f64().unwrap();
            assert!(val > 1.0e308 || val.is_infinite());
        }
        Ok(val) => panic!("Expected Number, got {:?}", val),
        Err(e) => panic!("Evaluation failed: {:?}", e),
    }
}

#[test]
fn test_nan_creation() {
    let evaluator = Evaluator::new(HashMap::new(), HashMap::new());
    // 0/0 triggers NaN
    let result = evaluator.evaluate("0 / 0");
    match result {
        Ok(Value::Null) => {} // This is what we WANT (null for NaN)
        Ok(Value::Number(n)) => {
            // Current behavior might be 0.0
            println!("Got number: {:?}", n);
            // Verify it is NOT 0.0 if we want to fail before fix (but let's just assert our desired behavior)
            // We want it to be Null
            panic!("Expected Null for NaN, got Number: {:?}", n);
        }
        Ok(val) => panic!("Expected Null, got {:?}", val),
        Err(e) => panic!("Evaluation failed: {:?}", e),
    }
}

#[test]
fn test_escape_sequence() {
    let evaluator = Evaluator::new(HashMap::new(), HashMap::new());
    let result = evaluator.evaluate("'\\n'");
    match result {
        Ok(Value::String(s)) => assert_eq!(s, "\n"),
        Ok(val) => panic!("Expected String, got {:?}", val),
        Err(e) => panic!("Evaluation failed: {:?}", e),
    }
}
