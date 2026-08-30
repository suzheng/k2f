use crate::{
    validate_semantic_tree, validate_semantic_tree_with_theme_vocab, CanvasMode, K2FError,
    Manifest, NodeContent, SemanticNode, TableDataSource, ThemeVocab,
};

pub fn validate_manifest_running_blocks(
    manifest: &Manifest,
    theme_vocab: &ThemeVocab,
) -> Result<(), K2FError> {
    if manifest.canvas_mode == CanvasMode::Infinite && !manifest.running_blocks.is_empty() {
        return Err(K2FError::RunningBlocksRequirePagedMode);
    }

    if manifest.running_blocks.is_empty() {
        return Ok(());
    }

    // Validate each running block node using the same structural + theme validation as the main tree.
    for rb in &manifest.running_blocks {
        validate_semantic_tree(&rb.node)?;
        validate_semantic_tree_with_theme_vocab(&rb.node, theme_vocab)?;
        validate_running_block_placeholders(&rb.node)?;
    }

    crate::validate_manifest_node_ids(manifest)?;

    Ok(())
}

fn validate_running_block_placeholders(node: &SemanticNode) -> Result<(), K2FError> {
    match &node.content {
        NodeContent::Text(text) => validate_text_placeholders(&node.id, text)?,
        NodeContent::Container { children } => {
            for c in children {
                validate_running_block_placeholders(c)?;
            }
        }
        NodeContent::Table(spec) => match &spec.data {
            TableDataSource::Inline { rows } => {
                for row in rows {
                    for cell in row {
                        validate_running_block_placeholders(cell)?;
                    }
                }
            }
            TableDataSource::Asset { .. } => {}
        },
        _ => {}
    }
    Ok(())
}

fn validate_text_placeholders(node_id: &str, text: &str) -> Result<(), K2FError> {
    let bytes = text.as_bytes();
    let mut i: usize = 0;

    while i < bytes.len() {
        // Reject any closing token without a prior opening token.
        if bytes[i] == b'}' && i + 1 < bytes.len() && bytes[i + 1] == b'}' {
            return Err(K2FError::RunningBlockUnbalancedPlaceholders {
                node_id: node_id.to_string(),
            });
        }

        // Parse an opening token.
        if bytes[i] == b'{' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            let token_start = i + 2;
            let close = find_close_token(bytes, token_start).ok_or_else(|| {
                K2FError::RunningBlockUnbalancedPlaceholders {
                    node_id: node_id.to_string(),
                }
            })?;

            // Safe: indices are at ASCII brace boundaries.
            let token = &text[token_start..close];
            match token {
                "page_current" | "page_total" => {}
                other => {
                    return Err(K2FError::RunningBlockInvalidPlaceholder {
                        node_id: node_id.to_string(),
                        token: other.to_string(),
                    });
                }
            }

            i = close + 2;
            continue;
        }

        i += 1;
    }

    Ok(())
}

fn find_close_token(bytes: &[u8], start: usize) -> Option<usize> {
    let mut j = start;
    while j + 1 < bytes.len() {
        if bytes[j] == b'}' && bytes[j + 1] == b'}' {
            return Some(j);
        }
        j += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        NodeContent, PageConfig, Pt, RunningBlockNode, RunningBlockPosition, SemanticNode,
    };

    fn vocab_with_role(role: &str) -> ThemeVocab {
        let mut vocab = ThemeVocab::default();
        vocab.roles.insert(role.to_string(), Default::default());
        vocab
    }

    fn text_node(id: &str, role: &str, text: &str) -> SemanticNode {
        SemanticNode {
            id: id.to_string(),
            role: role.to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text(text.to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        }
    }

    fn minimal_manifest(root: SemanticNode) -> Manifest {
        Manifest {
            title: "t".to_string(),
            canvas_mode: CanvasMode::Paged,
            page_config: PageConfig {
                width: Pt(1000),
                height: Pt(1000),
                margin: [Pt(0), Pt(0), Pt(0), Pt(0)],
            },
            root,
            running_blocks: vec![],
        }
    }

    #[test]
    fn accepts_allowed_placeholders() {
        let vocab = vocab_with_role("body");
        let mut m = minimal_manifest(text_node("root", "body", "hi"));
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Footer,
            node: text_node("rb", "body", "Page {{page_current}} / {{page_total}}"),
        });
        validate_manifest_running_blocks(&m, &vocab).unwrap();
    }

    #[test]
    fn rejects_unknown_placeholder() {
        let vocab = vocab_with_role("body");
        let mut m = minimal_manifest(text_node("root", "body", "hi"));
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Footer,
            node: text_node("rb", "body", "Page {{page}}"),
        });
        let err = validate_manifest_running_blocks(&m, &vocab).unwrap_err();
        assert!(matches!(
            err,
            K2FError::RunningBlockInvalidPlaceholder { .. }
        ));
    }

    #[test]
    fn rejects_unbalanced_open() {
        let vocab = vocab_with_role("body");
        let mut m = minimal_manifest(text_node("root", "body", "hi"));
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Footer,
            node: text_node("rb", "body", "Page {{page_current}"),
        });
        let err = validate_manifest_running_blocks(&m, &vocab).unwrap_err();
        assert!(matches!(
            err,
            K2FError::RunningBlockUnbalancedPlaceholders { .. }
        ));
    }

    #[test]
    fn rejects_unbalanced_close() {
        let vocab = vocab_with_role("body");
        let mut m = minimal_manifest(text_node("root", "body", "hi"));
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Footer,
            node: text_node("rb", "body", "Page page_total}}"),
        });
        let err = validate_manifest_running_blocks(&m, &vocab).unwrap_err();
        assert!(matches!(
            err,
            K2FError::RunningBlockUnbalancedPlaceholders { .. }
        ));
    }

    #[test]
    fn rejects_id_collision_with_root_tree() {
        let vocab = vocab_with_role("body");
        let mut m = minimal_manifest(text_node("same", "body", "hi"));
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Header,
            node: text_node("same", "body", "x"),
        });
        let err = validate_manifest_running_blocks(&m, &vocab).unwrap_err();
        assert!(matches!(err, K2FError::DuplicateNodeId { .. }));
    }

    #[test]
    fn rejects_duplicate_ids_across_running_blocks() {
        let vocab = vocab_with_role("body");
        let mut m = minimal_manifest(text_node("root", "body", "hi"));
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Header,
            node: text_node("dup", "body", "x"),
        });
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Footer,
            node: text_node("dup", "body", "y"),
        });
        let err = validate_manifest_running_blocks(&m, &vocab).unwrap_err();
        assert!(matches!(err, K2FError::DuplicateNodeId { .. }));
    }

    #[test]
    fn rejects_running_blocks_in_infinite_mode() {
        let vocab = vocab_with_role("body");
        let mut m = minimal_manifest(text_node("root", "body", "hi"));
        m.canvas_mode = CanvasMode::Infinite;
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Footer,
            node: text_node("rb", "body", "x"),
        });
        let err = validate_manifest_running_blocks(&m, &vocab).unwrap_err();
        assert!(matches!(err, K2FError::RunningBlocksRequirePagedMode));
    }
}
