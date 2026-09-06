use super::{join_span_text, selected_spans, slices_at, RectPt};
use k2f_markdown::NodeCharRange;
use k2f_paint::{OpenedDocument, TextSpan};
use serde::Serialize;

/// Same MIME as the web viewer (`sdk/js/viewer/copy.js`).
pub const K2F_NODES_MIME: &str = "application/x-k2f-nodes+json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CopyFormat {
    #[default]
    Markdown,
    Plain,
}

impl CopyFormat {
    pub fn toggle(self) -> Self {
        match self {
            Self::Markdown => Self::Plain,
            Self::Plain => Self::Markdown,
        }
    }

    pub fn hud_label(self) -> &'static str {
        match self {
            Self::Markdown => "Copy MD",
            Self::Plain => "Copy text",
        }
    }
}

/// Geometry-free node slice for `application/x-k2f-nodes+json`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelectedNode {
    pub node_id: String,
    pub char_start: usize,
    pub char_end: usize,
    pub text: String,
}

/// Clipboard payload: human `text/plain` plus optional K2F node JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct CopyPayload {
    pub plain: String,
    pub nodes: Vec<SelectedNode>,
}

impl CopyPayload {
    pub fn from_spans(spans: &[TextSpan], sel: RectPt) -> Option<Self> {
        Self::from_hits(&selected_spans(spans, sel))
    }

    /// Character-level range between two document points (web text-layer copy).
    pub fn from_points(spans: &[TextSpan], ax: f64, ay: f64, bx: f64, by: f64) -> Option<Self> {
        let sliced = slices_at(spans, ax, ay, bx, by);
        let hits: Vec<&TextSpan> = sliced.iter().collect();
        Self::from_hits(&hits)
    }

    fn from_hits(hits: &[&TextSpan]) -> Option<Self> {
        let plain = join_span_text(hits);
        if plain.is_empty() {
            return None;
        }
        let nodes = selected_nodes_from(hits);
        if nodes.is_empty() {
            return None;
        }
        Some(Self { plain, nodes })
    }

    /// Apply clipboard format. Markdown uses the semantic tree (same as web viewer).
    pub fn with_format(mut self, doc: &OpenedDocument, format: CopyFormat) -> Option<Self> {
        if format == CopyFormat::Plain {
            return Some(self);
        }
        let ranges: Vec<NodeCharRange> = self
            .nodes
            .iter()
            .map(|n| NodeCharRange {
                node_id: n.node_id.clone(),
                char_start: n.char_start,
                char_end: n.char_end,
            })
            .collect();
        let md = doc.selection_to_markdown(&ranges);
        if md.is_empty() {
            return None;
        }
        self.plain = md;
        Some(self)
    }

    pub fn nodes_json(&self) -> String {
        serde_json::to_string(&self.nodes).expect("SelectedNode is always serializable")
    }
}

fn selected_nodes_from(hits: &[&TextSpan]) -> Vec<SelectedNode> {
    let mut out: Vec<SelectedNode> = Vec::new();
    for s in hits {
        if let Some(prev) = out.iter_mut().find(|n| n.node_id == s.node_id) {
            prev.char_start = prev.char_start.min(s.char_start);
            prev.char_end = prev.char_end.max(s.char_end);
            prev.text.push_str(&s.text);
        } else {
            out.push(SelectedNode {
                node_id: s.node_id.clone(),
                char_start: s.char_start,
                char_end: s.char_end,
                text: s.text.clone(),
            });
        }
    }
    out
}
