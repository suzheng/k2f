use crate::theme_vocab::ThemeVocab;
use crate::{K2FError, NodeContent, SemanticNode, TableDataSource};

/// Validate that every node's `role` exists in the theme and that `variant` (if present)
/// is allowed for that role.
pub fn validate_semantic_tree_with_theme_vocab(
    root: &SemanticNode,
    theme_vocab: &ThemeVocab,
) -> Result<(), K2FError> {
    validate_node_role_variant(root, theme_vocab)?;
    Ok(())
}

fn validate_node_role_variant(
    node: &SemanticNode,
    theme_vocab: &ThemeVocab,
) -> Result<(), K2FError> {
    let role_entry = theme_vocab
        .roles
        .get(&node.role)
        .ok_or_else(|| K2FError::UnknownRole {
            node_id: node.id.clone(),
            role: node.role.clone(),
        })?;

    if let Some(variant) = node.variant.as_deref() {
        if !role_entry.variants.contains_key(variant) {
            return Err(K2FError::UnknownVariant {
                node_id: node.id.clone(),
                role: node.role.clone(),
                variant: variant.to_string(),
            });
        }
    }

    match &node.content {
        NodeContent::Container { children } => {
            for c in children {
                validate_node_role_variant(c, theme_vocab)?;
            }
        }
        NodeContent::Table(spec) => match &spec.data {
            TableDataSource::Inline { rows } => {
                for row in rows {
                    for cell in row {
                        validate_node_role_variant(cell, theme_vocab)?;
                    }
                }
            }
            TableDataSource::Asset { source } => {
                return Err(K2FError::TableAssetDataRequiresAssets {
                    node_id: node.id.clone(),
                    asset_source: source.clone(),
                });
            }
        },
        _ => {}
    }

    Ok(())
}
