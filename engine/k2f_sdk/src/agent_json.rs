use crate::error::{AgentError, INVALID_ARGUMENT};
use k2f_core::SemanticNode;
use serde_json::Value;

/// Agent-facing JSON: engine defaults (`layout: null`, empty `modifiers`) stripped.
pub(crate) fn to_agent_value(node: &SemanticNode) -> Result<Value, AgentError> {
    let mut v = serde_json::to_value(node)
        .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("serialize node: {e}")))?;
    strip_engine_defaults(&mut v);
    Ok(v)
}

fn strip_engine_defaults(v: &mut Value) {
    match v {
        Value::Object(map) => {
            if map.get("layout") == Some(&Value::Null) {
                map.remove("layout");
            }
            if map
                .get("modifiers")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
            {
                map.remove("modifiers");
            }
            for child in map.values_mut() {
                strip_engine_defaults(child);
            }
        }
        Value::Array(items) => {
            for child in items {
                strip_engine_defaults(child);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::text_node;

    #[test]
    fn omits_null_layout_and_empty_modifiers() {
        let v = to_agent_value(&text_node("n", "body", "hi")).unwrap();
        let map = v.as_object().unwrap();
        assert!(!map.contains_key("layout"));
        assert!(!map.contains_key("modifiers"));
        assert_eq!(map["content"]["value"], "hi");
    }
}
