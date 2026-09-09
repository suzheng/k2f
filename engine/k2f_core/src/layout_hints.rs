use crate::{LayoutHint, Pt};
use serde::{Deserialize, Deserializer, Serialize};

/// Missing `type` means stack — same default as omitting `layout` entirely.
pub(crate) fn deserialize_layout_hint<'de, D>(
    deserializer: D,
) -> Result<Option<LayoutHint>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(mut value) = Option::<serde_json::Value>::deserialize(deserializer)? else {
        return Ok(None);
    };
    if let serde_json::Value::Object(map) = &mut value {
        map.entry("type")
            .or_insert_with(|| serde_json::json!("stack"));
    }
    serde_json::from_value(value)
        .map(Some)
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

impl Default for Align {
    fn default() -> Self {
        Align::Stretch
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JustifyContent {
    Start,
    Center,
    End,
}

impl Default for JustifyContent {
    fn default() -> Self {
        JustifyContent::Start
    }
}

/// Default per-cell alignment for grid children.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CellAlign {
    #[serde(default)]
    pub x: Align,
    #[serde(default)]
    pub y: Align,
}

/// Optional deterministic fixed outer size for container layout hints.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FixedSizeHint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<Pt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<Pt>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayoutHint, NodeContent, SemanticNode};

    #[test]
    fn omitted_layout_type_deserializes_as_stack() {
        let node: SemanticNode = serde_json::from_value(serde_json::json!({
            "id": "r",
            "role": "rule",
            "content": { "type": "container", "value": {} },
            "layout": { "height": 1500 }
        }))
        .unwrap();
        match node.layout {
            Some(LayoutHint::Stack { size, .. }) => {
                assert_eq!(size.height, Some(Pt(1500)));
            }
            other => panic!("expected stack, got {other:?}"),
        }
        assert!(matches!(node.content, NodeContent::Container { children } if children.is_empty()));
    }
}
