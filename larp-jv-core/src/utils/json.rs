//! JSON utilities

use crate::error::LarpResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parse JSON from string
pub fn parse_json<T: for<'de> Deserialize<'de>>(json: &str) -> LarpResult<T> {
    Ok(serde_json::from_str(json)?)
}

/// Serialize to JSON string (pretty)
pub fn to_json_pretty<T: Serialize>(value: &T) -> LarpResult<String> {
    Ok(serde_json::to_string_pretty(value)?)
}

/// Serialize to JSON string (compact)
pub fn to_json_compact<T: Serialize>(value: &T) -> LarpResult<String> {
    Ok(serde_json::to_string(value)?)
}

/// Merge two JSON values (b overwrites a)
pub fn merge_json(a: &mut Value, b: Value) {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            for (k, v) in b {
                merge_json(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

/// Get value by JSON pointer (RFC 6901)
pub fn get_by_pointer<'a>(value: &'a Value, pointer: &str) -> Option<&'a Value> {
    value.pointer(pointer)
}

/// Set value by JSON pointer
pub fn set_by_pointer(value: &mut Value, pointer: &str, new_value: Value) -> bool {
    if let Some(target) = value.pointer_mut(pointer) {
        *target = new_value;
        true
    } else {
        false
    }
}

/// Deep clone a JSON value
pub fn deep_clone(value: &Value) -> Value {
    value.clone()
}

/// Check if JSON value is "empty" (null, empty object, empty array, empty string)
pub fn is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(_) => false,
        Value::Number(n) => n.is_finite() && n.as_f64() == Some(0.0),
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_and_serialize() {
        let json = r#"{"name": "test", "value": 42}"#;
        let parsed: Value = parse_json(json).unwrap();
        assert_eq!(parsed["name"], "test");
        assert_eq!(parsed["value"], 42);

        let compact = to_json_compact(&parsed).unwrap();
        assert!(compact.contains("test"));
    }

    #[test]
    fn test_merge_json() {
        let mut a = json!({"a": 1, "b": {"x": 10}});
        let b = json!({"b": {"y": 20}, "c": 3});
        merge_json(&mut a, b);
        assert_eq!(a["a"], 1);
        assert_eq!(a["b"]["x"], 10);
        assert_eq!(a["b"]["y"], 20);
        assert_eq!(a["c"], 3);
    }
}
