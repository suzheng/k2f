use crate::node_edit::node_text;
use crate::{for_each_node, nfc, RunningBlockNode, SemanticNode};

/// Tree search. Hits are stable node IDs, not a flat string index.
/// Empty query matches nothing.
pub fn search_tree(root: &SemanticNode, query: &str) -> Vec<String> {
    let q = nfc(query);
    if q.is_empty() {
        return vec![];
    }
    let mut ids = Vec::new();
    for_each_node(root, &mut |n| {
        if n.id.contains(&q) || node_text(n).is_some_and(|t| t.contains(&q)) {
            ids.push(n.id.clone());
        }
    });
    ids
}

pub fn search_trees(root: &SemanticNode, running: &[RunningBlockNode], query: &str) -> Vec<String> {
    let mut ids = search_tree(root, query);
    for rb in running {
        ids.extend(search_tree(&rb.node, query));
    }
    ids
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
    fn finds_by_text_and_id_and_skips_empty_query() {
        let root = SemanticNode {
            id: "root".into(),
            role: "document".into(),
            content: NodeContent::Container {
                children: vec![
                    text("invoice.total", "Grand Total: 100"),
                    text("invoice.note", "Net 30"),
                ],
            },
            ..Default::default()
        };
        assert_eq!(search_tree(&root, "100"), vec!["invoice.total"]);
        assert_eq!(search_tree(&root, "invoice.note"), vec!["invoice.note"]);
        assert!(search_tree(&root, "").is_empty());
        assert!(search_tree(&root, "missing").is_empty());
    }
}
