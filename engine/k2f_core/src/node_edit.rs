use crate::{
    find_node, find_node_mut, for_each_node, nfc, BreakInside, NodeContent, RunningBlockNode,
    SemanticNode, TableDataSource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeEditError {
    UnknownId(String),
    NotText(String),
    NotContainer(String),
    InvalidIndex {
        parent: String,
        index: usize,
        len: usize,
    },
    CannotDeleteRoot,
    DuplicateId(String),
}

impl std::fmt::Display for NodeEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownId(id) => write!(f, "no node with id '{id}'"),
            Self::NotText(id) => write!(f, "node '{id}' is not text"),
            Self::NotContainer(id) => write!(f, "node '{id}' is not a container"),
            Self::InvalidIndex { parent, index, len } => {
                write!(
                    f,
                    "insert index {index} out of range for '{parent}' (len {len})"
                )
            }
            Self::CannotDeleteRoot => write!(f, "cannot delete root"),
            Self::DuplicateId(id) => write!(f, "duplicate node id '{id}'"),
        }
    }
}

pub fn node_text(node: &SemanticNode) -> Option<&str> {
    match &node.content {
        NodeContent::Text(s) => Some(s.as_str()),
        NodeContent::Math(s) => Some(s.as_str()),
        NodeContent::CodeBlock(v) => match v {
            crate::CodeBlockValue::Text(s) => Some(s.as_str()),
            crate::CodeBlockValue::Lines(_) => None,
        },
        _ => None,
    }
}

pub fn find_in_trees<'a>(
    root: &'a SemanticNode,
    running: &'a [RunningBlockNode],
    id: &str,
) -> Option<&'a SemanticNode> {
    find_node(root, id).or_else(|| running.iter().find_map(|rb| find_node(&rb.node, id)))
}

pub fn find_in_trees_mut<'a>(
    root: &'a mut SemanticNode,
    running: &'a mut [RunningBlockNode],
    id: &str,
) -> Option<&'a mut SemanticNode> {
    if find_node(root, id).is_some() {
        return find_node_mut(root, id);
    }
    for rb in running {
        if find_node(&rb.node, id).is_some() {
            return find_node_mut(&mut rb.node, id);
        }
    }
    None
}

/// Replace a text node's string. NFC first. Drop modifiers that no longer fit.
pub fn replace_node_text(
    root: &mut SemanticNode,
    running: &mut [RunningBlockNode],
    id: &str,
    text: &str,
) -> Result<(), NodeEditError> {
    let node =
        find_in_trees_mut(root, running, id).ok_or_else(|| NodeEditError::UnknownId(id.into()))?;
    match &mut node.content {
        NodeContent::Text(existing) => {
            *existing = nfc(text);
            let len = existing.len();
            node.modifiers.retain(|m| {
                let [s, e] = m.range;
                s < e && e <= len && existing.is_char_boundary(s) && existing.is_char_boundary(e)
            });
            Ok(())
        }
        NodeContent::Math(existing) => {
            *existing = nfc(text);
            Ok(())
        }
        _ => Err(NodeEditError::NotText(id.into())),
    }
}

/// Surgical role change. Theme vocab is checked by the SDK; this only mutates the node.
pub fn apply_role(node: &mut SemanticNode, role: &str, variant: Option<&str>) {
    node.role = role.to_string();
    node.variant = variant.map(str::to_string);
    match role {
        "warning" | "critical_warning" | "signature_block" | "math" => {
            node.break_inside = BreakInside::Avoid
        }
        "h1" | "h2" | "h3" | "h4" => node.keep_with_next = true,
        _ => {}
    }
}

fn container_children(node: &mut SemanticNode) -> Option<&mut Vec<SemanticNode>> {
    match &mut node.content {
        NodeContent::Container { children } => Some(children),
        _ => None,
    }
}

fn insert_in_node(
    node: &mut SemanticNode,
    parent_id: &str,
    index: usize,
    child: SemanticNode,
) -> Result<bool, NodeEditError> {
    if node.id == parent_id {
        let children = container_children(node)
            .ok_or_else(|| NodeEditError::NotContainer(parent_id.into()))?;
        if index > children.len() {
            return Err(NodeEditError::InvalidIndex {
                parent: parent_id.into(),
                index,
                len: children.len(),
            });
        }
        children.insert(index, child);
        return Ok(true);
    }
    if let NodeContent::Container { children } = &mut node.content {
        for c in children.iter_mut() {
            if insert_in_node(c, parent_id, index, child.clone())? {
                return Ok(true);
            }
        }
    }
    if let NodeContent::Table(spec) = &mut node.content {
        if let TableDataSource::Inline { rows } = &mut spec.data {
            for row in rows.iter_mut() {
                for cell in row.iter_mut() {
                    if insert_in_node(cell, parent_id, index, child.clone())? {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

/// Insert `child` into `parent_id` at `index` (`index == len` appends).
pub fn insert_child(
    root: &mut SemanticNode,
    running: &mut [RunningBlockNode],
    parent_id: &str,
    index: usize,
    child: SemanticNode,
) -> Result<(), NodeEditError> {
    if insert_in_node(root, parent_id, index, child.clone())? {
        return Ok(());
    }
    for rb in running.iter_mut() {
        if insert_in_node(&mut rb.node, parent_id, index, child.clone())? {
            return Ok(());
        }
    }
    Err(NodeEditError::UnknownId(parent_id.into()))
}

fn remove_from_node(node: &mut SemanticNode, id: &str) -> Result<bool, NodeEditError> {
    if id == "root" {
        return Err(NodeEditError::CannotDeleteRoot);
    }
    if let NodeContent::Container { children } = &mut node.content {
        if let Some(pos) = children.iter().position(|c| c.id == id) {
            children.remove(pos);
            return Ok(true);
        }
        for c in children.iter_mut() {
            if remove_from_node(c, id)? {
                return Ok(true);
            }
        }
    }
    if let NodeContent::Table(spec) = &mut node.content {
        if let TableDataSource::Inline { rows } = &mut spec.data {
            for row in rows.iter_mut() {
                for cell in row.iter_mut() {
                    if remove_from_node(cell, id)? {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

/// Remove node `id`. Cannot delete `root`.
pub fn remove_node(
    root: &mut SemanticNode,
    running: &mut [RunningBlockNode],
    id: &str,
) -> Result<(), NodeEditError> {
    if id == "root" {
        return Err(NodeEditError::CannotDeleteRoot);
    }
    if remove_from_node(root, id)? {
        return Ok(());
    }
    for rb in running.iter_mut() {
        if remove_from_node(&mut rb.node, id)? {
            return Ok(());
        }
    }
    Err(NodeEditError::UnknownId(id.into()))
}

/// True if `id` exists anywhere in the semantic trees.
pub fn tree_contains_id(root: &SemanticNode, running: &[RunningBlockNode], id: &str) -> bool {
    find_in_trees(root, running, id).is_some()
}

/// Collect every node id under `node` (including `node` itself).
pub fn collect_ids_under(node: &SemanticNode, out: &mut Vec<String>) {
    for_each_node(node, &mut |n| out.push(n.id.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Modifier, SemanticNode};

    fn text(id: &str, s: &str) -> SemanticNode {
        SemanticNode {
            id: id.into(),
            role: "body".into(),
            content: NodeContent::Text(s.into()),
            modifiers: vec![Modifier {
                range: [0, 3],
                mod_type: "emphasis".into(),
                intent: "x".into(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn replace_normalizes_nfc_and_drops_invalid_modifiers() {
        let mut root = text("n", "abc");
        replace_node_text(&mut root, &mut [], "n", "e\u{0301}").unwrap();
        match &root.content {
            NodeContent::Text(t) => assert_eq!(t, "\u{00e9}"),
            _ => panic!(),
        }
        assert!(root.modifiers.is_empty());
    }

    #[test]
    fn replace_unknown_id_errors() {
        let mut root = text("n", "a");
        assert!(matches!(
            replace_node_text(&mut root, &mut [], "missing", "b"),
            Err(NodeEditError::UnknownId(_))
        ));
    }

    #[test]
    fn apply_role_sets_keep_together_for_headings_and_signature() {
        let mut n = text("n", "x");
        apply_role(&mut n, "h2", None);
        assert_eq!(n.role, "h2");
        assert!(n.keep_with_next);
        apply_role(&mut n, "signature_block", None);
        assert_eq!(n.break_inside, BreakInside::Avoid);
        apply_role(&mut n, "math", None);
        assert_eq!(n.break_inside, BreakInside::Avoid);
        apply_role(&mut n, "warning", Some("glass"));
        assert_eq!(n.variant.as_deref(), Some("glass"));
        assert_eq!(n.break_inside, BreakInside::Avoid);
    }
}
