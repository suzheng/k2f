use crate::{K2FError, NodeContent, SemanticNode};

/// Semantic role for code blocks.
pub const ROLE_CODE_BLOCK: &str = "code_block";

/// Allowed modifier type inside code blocks.
pub const MOD_TYPE_SYNTAX_HIGHLIGHT: &str = "syntax_highlight";

/// Validates invariants for semantic code blocks.
///
/// This is intentionally validation-only: it does not mutate the node.
pub fn validate_code_block_invariants(node: &SemanticNode) -> Result<(), K2FError> {
    let is_code_role = node.role == ROLE_CODE_BLOCK;
    let is_code_content = matches!(&node.content, NodeContent::CodeBlock(_));

    // Enforce semantic intent: role and content must agree.
    if is_code_role && !is_code_content {
        return Err(K2FError::CodeBlockRoleRequiresCodeBlockContent {
            node_id: node.id.clone(),
            got: content_kind(&node.content).to_string(),
        });
    }
    if is_code_content && !is_code_role {
        return Err(K2FError::CodeBlockContentRequiresCodeBlockRole {
            node_id: node.id.clone(),
            role: node.role.clone(),
        });
    }

    if !is_code_role {
        return Ok(());
    }

    if matches!(node.preserve_whitespace, Some(false)) {
        return Err(K2FError::CodeBlockPreserveWhitespaceMustBeTrue {
            node_id: node.id.clone(),
        });
    }

    if node.layout.is_some() {
        return Err(K2FError::CodeBlockLayoutNotAllowed {
            node_id: node.id.clone(),
        });
    }

    validate_code_block_modifiers(node)?;

    Ok(())
}

fn validate_code_block_modifiers(node: &SemanticNode) -> Result<(), K2FError> {
    for m in &node.modifiers {
        if m.mod_type != MOD_TYPE_SYNTAX_HIGHLIGHT {
            return Err(K2FError::CodeBlockDisallowedModifierType {
                node_id: node.id.clone(),
                mod_type: m.mod_type.clone(),
            });
        }
    }
    Ok(())
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

/// Code block payload value. Supports either a single string (may include '\n') or
/// a flat array of lines.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CodeBlockValue {
    Text(String),
    Lines(Vec<String>),
}

impl CodeBlockValue {
    /// Returns a borrowed view when the value is already a single string, otherwise
    /// returns an owned canonical string for processing.
    pub fn to_canonical_text(&self) -> std::borrow::Cow<'_, str> {
        match self {
            CodeBlockValue::Text(s) => std::borrow::Cow::Borrowed(s.as_str()),
            CodeBlockValue::Lines(lines) => std::borrow::Cow::Owned(lines.join("\n")),
        }
    }
}
