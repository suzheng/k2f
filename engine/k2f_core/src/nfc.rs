use crate::{for_each_node_mut, CodeBlockValue, Manifest, NodeContent};
use unicode_normalization::UnicodeNormalization;

pub fn nfc(s: &str) -> String {
    s.nfc().collect()
}

pub fn normalize_manifest_nfc(manifest: &mut Manifest) {
    for_each_node_mut(&mut manifest.root, &mut normalize_node_text);
    for rb in &mut manifest.running_blocks {
        for_each_node_mut(&mut rb.node, &mut normalize_node_text);
    }
}

fn normalize_node_text(node: &mut crate::SemanticNode) {
    match &mut node.content {
        NodeContent::Text(text) => {
            *text = nfc(text);
        }
        NodeContent::Math(text) => {
            *text = nfc(text);
        }
        NodeContent::CodeBlock(CodeBlockValue::Text(text)) => {
            *text = nfc(text);
        }
        NodeContent::CodeBlock(CodeBlockValue::Lines(lines)) => {
            for line in lines {
                *line = nfc(line);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CanvasMode, Manifest, NodeContent, PageConfig, Pt, SemanticNode};

    fn text_manifest(text: &str) -> Manifest {
        Manifest {
            title: "t".to_string(),
            canvas_mode: CanvasMode::Paged,
            page_config: PageConfig {
                width: Pt(595000),
                height: Pt(842000),
                margin: [Pt(72000); 4],
            },
            root: SemanticNode {
                id: "root".to_string(),
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
            },
            running_blocks: vec![],
        }
    }

    #[test]
    fn nfc_composes_decomposed_e_acute() {
        let nfd = "e\u{0301}";
        let composed = "\u{00e9}";
        assert_ne!(nfd, composed);
        assert_eq!(nfc(nfd), composed);
        assert_eq!(nfc(composed), composed);
    }

    #[test]
    fn manifest_text_is_normalized_before_hash() {
        let mut nfd = text_manifest("e\u{0301}");
        let mut composed = text_manifest("\u{00e9}");
        normalize_manifest_nfc(&mut nfd);
        normalize_manifest_nfc(&mut composed);
        match (&nfd.root.content, &composed.root.content) {
            (NodeContent::Text(a), NodeContent::Text(b)) => assert_eq!(a, b),
            _ => panic!("expected text"),
        }
        assert_eq!(
            crate::hash_manifest_semantic(&nfd),
            crate::hash_manifest_semantic(&composed)
        );
    }
}
