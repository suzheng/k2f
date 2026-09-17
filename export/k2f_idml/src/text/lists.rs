use crate::align::source_glyphs;
use crate::coord::millipt_to_pt;
use crate::ir::{ScriptPos, TextRun};
use k2f_core::{GeometryNode, ListMarkerType, NodeContent, SemanticNode, TableDataSource};
use std::collections::BTreeMap;

pub(crate) fn list_start_at(root: &SemanticNode) -> BTreeMap<String, u32> {
    let mut out = BTreeMap::new();
    walk_list_starts(root, &mut out);
    out
}

fn walk_list_starts(node: &SemanticNode, out: &mut BTreeMap<String, u32>) {
    match &node.content {
        NodeContent::Container { children } => {
            scan_list_starts(children, out);
            for child in children {
                walk_list_starts(child, out);
            }
        }
        NodeContent::Table(spec) => {
            if let TableDataSource::Inline { rows } = &spec.data {
                for row in rows {
                    scan_list_starts(row, out);
                    for cell in row {
                        walk_list_starts(cell, out);
                    }
                }
            }
        }
        _ => {}
    }
}

fn scan_list_starts(nodes: &[SemanticNode], out: &mut BTreeMap<String, u32>) {
    let mut by_list: BTreeMap<String, u32> = BTreeMap::new();
    for node in nodes {
        if node.role != "list_item" || node.marker_type != Some(ListMarkerType::Number) {
            continue;
        }
        let Some(list_id) = node.list_id.as_deref() else {
            continue;
        };
        let n = by_list.entry(list_id.to_string()).or_insert(0);
        *n = n.saturating_add(1);
        out.insert(node.id.clone(), *n);
    }
}

/// Left pad before the decorative marker (min x of all lock glyphs).
pub(crate) fn list_outer_pad_pt(geo: Option<&GeometryNode>) -> f64 {
    millipt_to_pt(ink_left(geo))
}

/// Marker-column width for hanging indent when the marker is a literal run.
/// Decorative markers are `CLUSTER_NOT_SOURCE`; body starts after them.
/// Using body-left as the frame inset would double-count that column once
/// `{n}.` / `•` is prepended.
pub(crate) fn list_hanging_pt(geo: Option<&GeometryNode>) -> f64 {
    let ink = ink_left(geo);
    let body = body_left(geo).unwrap_or(ink);
    millipt_to_pt((body - ink).max(0))
}

fn ink_left(geo: Option<&GeometryNode>) -> i128 {
    let Some(geo) = geo else {
        return 0;
    };
    geo.glyphs
        .iter()
        .map(|g| g.x_offset.0)
        .min()
        .unwrap_or(0)
        .max(0)
}

fn body_left(geo: Option<&GeometryNode>) -> Option<i128> {
    let geo = geo?;
    let glyphs = source_glyphs(geo);
    if glyphs.is_empty() {
        return None;
    }
    Some(
        glyphs
            .iter()
            .map(|g| g.x_offset.0)
            .min()
            .unwrap_or(0)
            .max(0),
    )
}

pub(crate) fn prepend_literal_bullet(runs: &mut Vec<TextRun>) {
    let Some(first) = runs.first() else {
        return;
    };
    let marker = marker_run(first, "•\u{00A0}");
    runs.insert(0, marker);
}

pub(crate) fn prepend_literal_number(runs: &mut Vec<TextRun>, n: u32) {
    let Some(first) = runs.first() else {
        return;
    };
    let marker = marker_run(first, &format!("{n}.\u{00A0}"));
    runs.insert(0, marker);
}

fn marker_run(first: &TextRun, text: &str) -> TextRun {
    TextRun {
        text: text.into(),
        font_name: first.font_name.clone(),
        size_pt: first.size_pt,
        bold: false,
        italic: false,
        underline: false,
        strike: false,
        color_hex: first.color_hex.clone(),
        hyperlink: None,
        script: ScriptPos::Baseline,
        leading_pt: first.leading_pt,
        auto_page_number: false,
        tracking: first.tracking,
        face_style: first.face_style.clone(),
    }
}

const PAGE_CURRENT: &str = "{{page_current}}";
const PAGE_TOTAL: &str = "{{page_total}}";

pub(crate) fn expand_page_tokens(
    runs: Vec<TextRun>,
    total_pages: usize,
    current: usize,
) -> Vec<TextRun> {
    if runs.is_empty() {
        return runs;
    }
    let full: String = runs.iter().map(|r| r.text.as_str()).collect();
    if !full.contains(PAGE_CURRENT) && !full.contains(PAGE_TOTAL) {
        return runs;
    }
    let total = total_pages.to_string();
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < full.len() {
        let next = next_token(&full, pos);
        let Some((i, tok)) = next else {
            out.extend(slice_runs(&runs, pos, full.len()));
            break;
        };
        if i > pos {
            out.extend(slice_runs(&runs, pos, i));
        }
        let mut run = style_at(&runs, i).clone();
        if tok == PAGE_CURRENT {
            run.text = current.to_string();
            run.auto_page_number = false;
        } else {
            run.text = total.clone();
            run.auto_page_number = false;
        }
        out.push(run);
        pos = i + tok.len();
    }
    out
}

fn next_token(full: &str, pos: usize) -> Option<(usize, &'static str)> {
    let cur = full[pos..]
        .find(PAGE_CURRENT)
        .map(|i| (pos + i, PAGE_CURRENT));
    let tot = full[pos..].find(PAGE_TOTAL).map(|i| (pos + i, PAGE_TOTAL));
    match (cur, tot) {
        (Some(a), Some(b)) if a.0 <= b.0 => Some(a),
        (Some(a), None) => Some(a),
        (_, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn style_at(runs: &[TextRun], byte_off: usize) -> &TextRun {
    let mut pos = 0;
    for run in runs {
        let end = pos + run.text.len();
        if byte_off < end || (byte_off == pos && run.text.is_empty()) {
            return run;
        }
        pos = end;
    }
    runs.last().expect("non-empty runs")
}

fn slice_runs(runs: &[TextRun], start: usize, end: usize) -> Vec<TextRun> {
    let mut pos = 0;
    let mut out = Vec::new();
    for run in runs {
        let a = pos;
        let b = pos + run.text.len();
        pos = b;
        let lo = start.max(a);
        let hi = end.min(b);
        if lo >= hi {
            continue;
        }
        let mut piece = run.clone();
        piece.text = run.text[lo - a..hi - a].to_string();
        piece.auto_page_number = false;
        if !piece.text.is_empty() {
            out.push(piece);
        }
    }
    out
}

#[cfg(test)]
mod hanging_tests {
    use super::*;
    use k2f_core::{GlyphPosition, Pt};

    fn gp(cluster: u32, x: i128) -> GlyphPosition {
        GlyphPosition {
            glyph_id: 1,
            cluster,
            x_offset: Pt(x),
            y_offset: Pt(0),
            x_advance: Pt(6_000),
            y_advance: Pt(0),
        }
    }

    #[test]
    fn hanging_is_marker_column_not_body_left() {
        let geo = GeometryNode {
            id: "li".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(400_000),
            height: Pt(20_000),
            glyphs: vec![
                gp(GlyphPosition::CLUSTER_NOT_SOURCE, 8_000),
                gp(GlyphPosition::CLUSTER_NOT_SOURCE, 14_000),
                gp(0, 26_000),
                gp(1, 26_000),
            ],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        assert!((list_outer_pad_pt(Some(&geo)) - 8.0).abs() < 0.01);
        assert!((list_hanging_pt(Some(&geo)) - 18.0).abs() < 0.01);
        assert!((list_hanging_pt(None) - 0.0).abs() < 0.01);
    }
}
