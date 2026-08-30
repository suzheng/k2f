use super::types::OutlineNode;
use k2f_core::{node_text, NodeContent, SemanticNode};

const PREVIEW_CHARS: usize = 80;

pub fn build_outline(root: &SemanticNode) -> Vec<OutlineNode> {
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn walk(node: &SemanticNode, out: &mut Vec<OutlineNode>) {
    out.push(OutlineNode {
        id: node.id.clone(),
        role: node.role.clone(),
        preview: preview_text(node),
        children: match &node.content {
            NodeContent::Container { children } => children.iter().map(|c| c.id.clone()).collect(),
            _ => Vec::new(),
        },
    });
    if let NodeContent::Container { children } = &node.content {
        for child in children {
            walk(child, out);
        }
    }
}

fn preview_text(node: &SemanticNode) -> Option<String> {
    let text = node_text(node)?;
    let char_count = text.chars().count();
    let mut preview: String = text.chars().take(PREVIEW_CHARS).collect();
    if char_count > PREVIEW_CHARS {
        preview.push('…');
    }
    Some(preview)
}
