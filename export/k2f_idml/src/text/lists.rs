use crate::ir::{ScriptPos, TextRun};
use k2f_core::{ListMarkerType, NodeContent, SemanticNode, TableDataSource};
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
    }
}

const PAGE_CURRENT: &str = "{{page_current}}";
const PAGE_TOTAL: &str = "{{page_total}}";

pub(crate) fn expand_page_tokens(runs: Vec<TextRun>, total_pages: usize) -> Vec<TextRun> {
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
            run.text.clear();
            run.auto_page_number = true;
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
