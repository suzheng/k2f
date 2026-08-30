use crate::{node_text, SemanticNode};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Selection {
    pub id: String,
    pub ids: Vec<String>,
    pub role: String,
    pub variant: Option<String>,
    pub text: Option<String>,
    pub char_range: Option<[usize; 2]>,
    pub node: serde_json::Value,
}

/// Official clipboard: node id + semantic JSON. Plain text is derived, never a source.
#[derive(Debug, Clone, Serialize)]
pub struct Clipboard {
    pub id: String,
    pub node: serde_json::Value,
    pub text: Option<String>,
}

pub fn selection_of(node: &SemanticNode) -> Option<Selection> {
    selection_with_ids(node, vec![node.id.clone()])
}

pub fn selection_with_ids(node: &SemanticNode, ids: Vec<String>) -> Option<Selection> {
    let text = node_text(node).map(str::to_string);
    let char_range = text.as_ref().map(|t| [0, t.chars().count()]);
    Some(Selection {
        id: node.id.clone(),
        ids,
        role: node.role.clone(),
        variant: node.variant.clone(),
        text,
        char_range,
        node: serde_json::to_value(node).ok()?,
    })
}

pub fn clipboard_of(node: &SemanticNode) -> Option<Clipboard> {
    Some(Clipboard {
        id: node.id.clone(),
        node: serde_json::to_value(node).ok()?,
        text: node_text(node).map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeContent;

    fn text(id: &str, s: &str) -> SemanticNode {
        SemanticNode {
            id: id.into(),
            role: "body".into(),
            content: NodeContent::Text(s.into()),
            ..Default::default()
        }
    }

    #[test]
    fn clipboard_is_id_and_node_json_text_is_derived() {
        let n = text("contract.compensation.amount", "USD 100");
        let clip = clipboard_of(&n).unwrap();
        assert_eq!(clip.id, "contract.compensation.amount");
        assert_eq!(clip.text.as_deref(), Some("USD 100"));
        assert_eq!(clip.node["id"], "contract.compensation.amount");
        assert_eq!(clip.node["content"]["value"], "USD 100");
        let sel = selection_with_ids(&n, vec!["amount".into(), "root".into()]).unwrap();
        assert_eq!(sel.ids, vec!["amount", "root"]);
        assert_eq!(sel.id, "contract.compensation.amount");
    }
}
