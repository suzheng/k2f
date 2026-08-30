use crate::{K2FError, NodeContent, SemanticNode};

/// Semantic role for display math formulas.
pub const ROLE_MATH: &str = "math";

/// Validates invariants for semantic math nodes.
///
/// This is intentionally validation-only: it does not mutate the node.
pub fn validate_math_invariants(node: &SemanticNode) -> Result<(), K2FError> {
    let is_math_role = node.role == ROLE_MATH;
    let is_math_content = matches!(&node.content, NodeContent::Math(_));

    if is_math_role && !is_math_content {
        return Err(K2FError::MathRoleRequiresMathContent {
            node_id: node.id.clone(),
            got: content_kind(&node.content).to_string(),
        });
    }
    if is_math_content && !is_math_role {
        return Err(K2FError::MathContentRequiresMathRole {
            node_id: node.id.clone(),
            role: node.role.clone(),
        });
    }

    if !is_math_role {
        return Ok(());
    }

    if node.layout.is_some() {
        return Err(K2FError::MathLayoutNotAllowed {
            node_id: node.id.clone(),
        });
    }

    if !node.modifiers.is_empty() {
        return Err(K2FError::MathModifiersNotAllowed {
            node_id: node.id.clone(),
        });
    }

    match &node.content {
        NodeContent::Math(tex) if tex.trim().is_empty() => Err(K2FError::MathEmpty {
            node_id: node.id.clone(),
        }),
        _ => Ok(()),
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
