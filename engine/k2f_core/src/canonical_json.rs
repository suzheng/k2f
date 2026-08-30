use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// Canonical JSON (sorted object keys) used for hashing and lock serialization.
pub fn canonical_json_string<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    serde_json::to_string(&canonicalize_json_value(v))
}

pub fn canonicalize_json_value(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut sorted: BTreeMap<String, Value> = BTreeMap::new();
            for (k, v2) in map {
                sorted.insert(k, canonicalize_json_value(v2));
            }
            let mut out = serde_json::Map::new();
            for (k, v2) in sorted {
                out.insert(k, v2);
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(canonicalize_json_value).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorts_object_keys() {
        let v = json!({"b": 1, "a": 2});
        assert_eq!(canonical_json_string(&v).unwrap(), r#"{"a":2,"b":1}"#);
    }
}
