use serde::{Deserialize, Serialize};

use crate::{K2FError, NodeContent, SemanticNode};

/// Semantic role for flat list items.
pub const ROLE_LIST_ITEM: &str = "list_item";

/// Marker style for list items.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListMarkerType {
    Bullet,
    Number,
}

/// Validates invariants for flat list items.
///
/// This is intentionally validation-only: it does not mutate the node.
pub fn validate_list_item_invariants(node: &SemanticNode) -> Result<(), K2FError> {
    if node.role != ROLE_LIST_ITEM {
        return Ok(());
    }

    match node.list_id.as_deref() {
        None => {
            return Err(K2FError::ListItemMissingListId {
                node_id: node.id.clone(),
            })
        }
        Some(id) if id.trim().is_empty() => {
            return Err(K2FError::ListItemEmptyListId {
                node_id: node.id.clone(),
            })
        }
        Some(_) => {}
    }

    match &node.content {
        NodeContent::Text(_) => Ok(()),
        other => Err(K2FError::ListItemRequiresTextContent {
            node_id: node.id.clone(),
            got: content_kind(other).to_string(),
        }),
    }
}

fn content_kind(content: &NodeContent) -> &'static str {
    match content {
        NodeContent::Text(_) => "text",
        NodeContent::CodeBlock(_) => "code_block",
        NodeContent::Math(_) => "math",
        NodeContent::Image { .. } => "image",
        NodeContent::Container { .. } => "container",
        NodeContent::Table(_) => "table",
        NodeContent::TableReference { .. } => "table_reference",
    }
}
