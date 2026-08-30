use k2f_core::{
    clusters_usable, find_in_trees, node_text, GeometryNode, GlyphPosition, Page, RunningBlockNode,
    SemanticNode,
};

use crate::document::OpenedDocument;

/// One selectable fragment on a painted page. Coordinates are points, not millipts.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct TextSpan {
    pub node_id: String,
    pub char_start: usize,
    pub char_end: usize,
    pub text: String,
    pub x_pt: f64,
    pub y_pt: f64,
    pub width_pt: f64,
    pub height_pt: f64,
}

impl OpenedDocument {
    pub fn text_layer(&self, page: usize) -> Vec<TextSpan> {
        let Some(lock) = &self.lock else {
            return vec![];
        };
        let Some(page) = lock.geometry.pages.get(page) else {
            return vec![];
        };
        spans_for_page(page, &self.query_root, &self.query_running)
    }
}

pub fn spans_for_page(
    page: &Page,
    root: &SemanticNode,
    running: &[RunningBlockNode],
) -> Vec<TextSpan> {
    let mut out = Vec::new();
    walk(&page.root, root, running, &mut out);
    out
}

fn walk(
    geo: &GeometryNode,
    root: &SemanticNode,
    running: &[RunningBlockNode],
    out: &mut Vec<TextSpan>,
) {
    if !geo.glyphs.is_empty() {
        if let Some(text) = find_in_trees(root, running, &geo.id).and_then(node_text) {
            out.extend(spans_for_node(geo, text));
        }
    }
    for child in &geo.children {
        walk(child, root, running, out);
    }
}

fn spans_for_node(geo: &GeometryNode, text: &str) -> Vec<TextSpan> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return vec![];
    }
    if !clusters_usable(&geo.glyphs) {
        return vec![span(
            &geo.id,
            0,
            chars.len(),
            text.to_string(),
            geo.x.as_f64_pt(),
            geo.y.as_f64_pt(),
            geo.width.as_f64_pt(),
            geo.height.as_f64_pt(),
        )];
    }

    let mut by_y: std::collections::BTreeMap<i128, Vec<(usize, &GlyphPosition)>> =
        std::collections::BTreeMap::new();
    for (i, g) in geo.glyphs.iter().enumerate() {
        if g.cluster == GlyphPosition::CLUSTER_NOT_SOURCE {
            continue;
        }
        if (g.cluster as usize) >= chars.len() {
            continue;
        }
        by_y.entry(g.y_offset.0).or_default().push((i, g));
    }

    let ys: Vec<i128> = by_y.keys().copied().collect();
    let mut out = Vec::new();
    for (line_i, y) in ys.iter().enumerate() {
        let glyphs = &by_y[y];
        let char_start = glyphs.iter().map(|(_, g)| g.cluster).min().unwrap() as usize;
        let char_end = (glyphs.iter().map(|(_, g)| g.cluster).max().unwrap() as usize + 1)
            .min(chars.len());
        if char_start >= char_end {
            continue;
        }
        let fragment: String = chars[char_start..char_end].iter().collect();
        let min_x = glyphs.iter().map(|(_, g)| g.x_offset.0).min().unwrap();
        let max_r = glyphs
            .iter()
            .map(|(_, g)| g.x_offset.0 + g.x_advance.0.max(0))
            .max()
            .unwrap();
        let height = match ys.get(line_i + 1) {
            Some(next_y) => (*next_y - *y).max(1),
            None => glyphs
                .first()
                .and_then(|(idx, _)| font_size_milli(geo, *idx))
                .unwrap_or(12_000),
        };
        out.push(span(
            &geo.id,
            char_start,
            char_end,
            fragment,
            (geo.x.0 + min_x) as f64 / 1000.0,
            (geo.y.0 + y) as f64 / 1000.0,
            (max_r - min_x).max(0) as f64 / 1000.0,
            height as f64 / 1000.0,
        ));
    }
    out
}

fn font_size_milli(geo: &GeometryNode, glyph_index: usize) -> Option<i128> {
    geo.text_runs
        .iter()
        .find(|run| glyph_index >= run.glyph_range[0] && glyph_index < run.glyph_range[1])
        .map(|run| run.style.font_size.0)
}

fn span(
    node_id: &str,
    char_start: usize,
    char_end: usize,
    text: String,
    x_pt: f64,
    y_pt: f64,
    width_pt: f64,
    height_pt: f64,
) -> TextSpan {
    TextSpan {
        node_id: node_id.to_string(),
        char_start,
        char_end,
        text,
        x_pt,
        y_pt,
        width_pt,
        height_pt,
    }
}
