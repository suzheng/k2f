use crate::{for_each_node, K2FError, Manifest, SemanticNode};
use std::collections::BTreeSet;

/// Allowed node IDs: dotted segments of ASCII letters, digits, and underscore.
/// Examples: `root`, `contract.clause_4.amount`.
pub const NODE_ID_PATTERN: &str = r"^[A-Za-z0-9][A-Za-z0-9_]*(\.[A-Za-z0-9][A-Za-z0-9_]*)*$";

pub fn is_valid_node_id(id: &str) -> bool {
    // Must match NODE_ID_PATTERN / schema: each dotted segment starts with [A-Za-z0-9].
    if id.is_empty() {
        return false;
    }
    let mut seg_start = true;
    for c in id.chars() {
        if seg_start {
            if !c.is_ascii_alphanumeric() {
                return false;
            }
            seg_start = false;
            continue;
        }
        if c == '.' {
            seg_start = true;
            continue;
        }
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }
    !seg_start
}

pub fn collect_sorted_node_ids(manifest: &Manifest) -> Vec<String> {
    let mut ids = BTreeSet::new();
    for_each_node(&manifest.root, &mut |n| {
        ids.insert(n.id.clone());
    });
    for rb in &manifest.running_blocks {
        for_each_node(&rb.node, &mut |n| {
            ids.insert(n.id.clone());
        });
    }
    ids.into_iter().collect()
}

pub fn validate_manifest_node_ids(manifest: &Manifest) -> Result<(), K2FError> {
    let mut seen = BTreeSet::new();
    validate_tree_ids(&manifest.root, &mut seen)?;
    for rb in &manifest.running_blocks {
        validate_tree_ids(&rb.node, &mut seen)?;
    }
    Ok(())
}

fn validate_tree_ids(node: &SemanticNode, seen: &mut BTreeSet<String>) -> Result<(), K2FError> {
    let mut err = Ok(());
    for_each_node(node, &mut |n| {
        if err.is_err() {
            return;
        }
        if n.id.is_empty() {
            err = Err(K2FError::EmptyNodeId);
            return;
        }
        if !is_valid_node_id(&n.id) {
            err = Err(K2FError::InvalidNodeId {
                node_id: n.id.clone(),
            });
            return;
        }
        if !seen.insert(n.id.clone()) {
            err = Err(K2FError::DuplicateNodeId {
                node_id: n.id.clone(),
            });
        }
    });
    err
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CanvasMode, Manifest, NodeContent, PageConfig, Pt, RunningBlockNode, RunningBlockPosition,
        SemanticNode,
    };

    fn text_node(id: &str, text: &str) -> SemanticNode {
        SemanticNode {
            id: id.to_string(),
            role: "body".to_string(),
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

    fn manifest(root: SemanticNode) -> Manifest {
        Manifest {
            title: "t".to_string(),
            canvas_mode: CanvasMode::Paged,
            page_config: PageConfig {
                width: Pt(595000),
                height: Pt(842000),
                margin: [Pt(72000); 4],
            },
            root,
            running_blocks: vec![],
        }
    }

    #[test]
    fn accepts_dotted_ids() {
        let m = manifest(text_node("contract.clause_4.amount", "100"));
        validate_manifest_node_ids(&m).unwrap();
    }

    #[test]
    fn rejects_empty_id() {
        let m = manifest(text_node("", "x"));
        assert!(matches!(
            validate_manifest_node_ids(&m),
            Err(K2FError::EmptyNodeId)
        ));
    }

    #[test]
    fn rejects_spaces_and_hyphens() {
        let m = manifest(text_node("Times New Roman_10", "x"));
        assert!(matches!(
            validate_manifest_node_ids(&m),
            Err(K2FError::InvalidNodeId { .. })
        ));
        let m = manifest(text_node("clause-4", "x"));
        assert!(matches!(
            validate_manifest_node_ids(&m),
            Err(K2FError::InvalidNodeId { .. })
        ));
    }

    #[test]
    fn rejects_underscore_right_after_dot() {
        assert!(!is_valid_node_id("clause._amount"));
        assert!(is_valid_node_id("clause.amount"));
        assert!(is_valid_node_id("clause.amount_1"));
        assert!(!is_valid_node_id("clause."));
        assert!(!is_valid_node_id(".clause"));
        assert!(!is_valid_node_id("clause..amount"));
    }

    #[test]
    fn rejects_duplicate_ids_in_root_tree() {
        let root = SemanticNode {
            id: "root".to_string(),
            role: "document".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Container {
                children: vec![text_node("dup", "a"), text_node("dup", "b")],
            },
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };
        let err = validate_manifest_node_ids(&manifest(root)).unwrap_err();
        match err {
            K2FError::DuplicateNodeId { node_id } => assert_eq!(node_id, "dup"),
            other => panic!("expected DuplicateNodeId, got {other:?}"),
        }
    }

    #[test]
    fn rejects_id_collision_with_running_blocks() {
        let mut m = manifest(text_node("same", "hi"));
        m.running_blocks.push(RunningBlockNode {
            position: RunningBlockPosition::Header,
            node: text_node("same", "x"),
        });
        let err = validate_manifest_node_ids(&m).unwrap_err();
        match err {
            K2FError::DuplicateNodeId { node_id } => assert_eq!(node_id, "same"),
            other => panic!("expected DuplicateNodeId, got {other:?}"),
        }
    }
}
