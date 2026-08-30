use crate::range::char_to_byte;
use k2f_core::{Modifier, NodeContent, SemanticNode};

/// Slice a text node's value by UTF-8 character indexes; remap modifier byte ranges.
pub fn slice_text_node(node: &SemanticNode, char_start: usize, char_end: usize) -> SemanticNode {
    let mut out = node.clone();
    let Some(text) = text_of(node) else {
        return out;
    };
    let n = text.chars().count();
    let start = char_start.min(n);
    let end = char_end.min(n).max(start);
    let b0 = char_to_byte(text, start);
    let b1 = char_to_byte(text, end);
    let sliced = text[b0..b1].to_string();
    out.modifiers = remap_modifiers(&node.modifiers, b0, b1);
    set_text(&mut out, sliced);
    out
}

fn text_of(node: &SemanticNode) -> Option<&str> {
    match &node.content {
        NodeContent::Text(s) => Some(s.as_str()),
        NodeContent::Math(s) => Some(s.as_str()),
        _ => None,
    }
}

fn set_text(node: &mut SemanticNode, s: String) {
    match &mut node.content {
        NodeContent::Text(t) => *t = s,
        NodeContent::Math(t) => *t = s,
        _ => {}
    }
}

fn remap_modifiers(mods: &[Modifier], b0: usize, b1: usize) -> Vec<Modifier> {
    let mut out = Vec::new();
    for m in mods {
        let s = m.range[0].max(b0);
        let e = m.range[1].min(b1);
        if s >= e {
            continue;
        }
        out.push(Modifier {
            range: [s - b0, e - b0],
            mod_type: m.mod_type.clone(),
            intent: m.intent.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(id: &str, text: &str, mods: Vec<Modifier>) -> SemanticNode {
        SemanticNode {
            id: id.into(),
            role: "body".into(),
            content: NodeContent::Text(text.into()),
            modifiers: mods,
            ..Default::default()
        }
    }

    #[test]
    fn slices_cjk_and_remaps_strong() {
        // "你好世界" — strong on 好 (chars 1..2 = bytes 3..6)
        let n = body(
            "t",
            "你好世界",
            vec![Modifier {
                range: [3, 6],
                mod_type: "emphasis".into(),
                intent: "strong".into(),
            }],
        );
        let sliced = slice_text_node(&n, 1, 3); // 好世
        assert_eq!(
            match &sliced.content {
                NodeContent::Text(t) => t.as_str(),
                _ => panic!(),
            },
            "好世"
        );
        assert_eq!(sliced.modifiers.len(), 1);
        assert_eq!(sliced.modifiers[0].range, [0, 3]); // 好 is 3 bytes
    }
}
