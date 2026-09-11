mod align;
mod fields;
mod font;
mod metrics;
mod runs;
mod tracking;

use crate::coord::pt_to_emu;
use crate::ir::{TextBox, TextRun};
use crate::ooxml;
use k2f_core::{
    GeometryNode, ListMarkerType, NodeContent, Rect, SemanticNode, TableDataSource, TextGlyphRun,
};
use std::collections::{BTreeMap, HashSet};

pub use align::infer_text_align;
pub(crate) use fields::{expand_page_vars, has_page_tokens};
pub(crate) use font::FontCtx;
pub(crate) use metrics::{line_spacing_twips, vert_center};
pub(crate) use runs::runs_from_paint;

pub fn textbox_from_draw(
    node: &SemanticNode,
    rect: &Rect,
    paint_runs: &[TextGlyphRun],
    geo: Option<&GeometryNode>,
    fonts: &BTreeMap<String, Vec<u8>>,
) -> Option<TextBox> {
    textbox_from_draw_ctx(node, rect, paint_runs, geo, &FontCtx::new(fonts), 0, 0, 1, 1)
}

pub(crate) fn textbox_from_draw_ctx(
    node: &SemanticNode,
    rect: &Rect,
    paint_runs: &[TextGlyphRun],
    geo: Option<&GeometryNode>,
    fonts: &FontCtx,
    relative_height: u32,
    page_idx: usize,
    total_pages: usize,
    list_start: u32,
) -> Option<TextBox> {
    if node.role == "math" || matches!(node.content, NodeContent::Math(_)) {
        return None;
    }
    let raw = k2f_core::node_text(node)?;
    if raw.is_empty() {
        return None;
    }
    let running = node.role.starts_with("running_");
    let mut runs = if running {
        let text = expand_page_vars(raw, page_idx, total_pages);
        if text.is_empty() {
            return None;
        }
        runs::runs_from_plain(&text, paint_runs, &node.modifiers, fonts, geo)
    } else {
        runs::runs_from_paint(raw, paint_runs, &node.modifiers, geo, fonts)
    };
    if runs.is_empty() {
        return None;
    }
    let align = geo
        .map(|g| infer_text_align(g, raw))
        .unwrap_or(crate::ir::TextAlign::Left);
    let (mut l_ins_emu, mut t_ins_emu, r_ins_emu, b_ins_emu) = metrics::insets(geo, align);
    // Prefer the largest face (body), not paint_runs[0] — that is often a
    // leading superscript whose half-size would crush wrap/spacing heuristics.
    let font_size = paint_runs
        .iter()
        .map(|r| r.style.font_size)
        .max_by_key(|p| p.0.abs())
        .or_else(|| geo.map(|g| k2f_core::Pt(align::body_font_size(g))))
        .unwrap_or(k2f_core::Pt(12_000));
    let numbered = node.marker_type == Some(ListMarkerType::Number);
    let bullet =
        !numbered && (node.role == "list_item" || node.marker_type == Some(ListMarkerType::Bullet));
    // Lists need host wrap. Pinning lock breaks splits one item into many
    // <w:p>s; hosts often ignore w:ind inside drawing text boxes.
    let pin = !running
        && !(bullet || numbered)
        && align::should_pin_lock_breaks(geo, font_size, Some(raw), align);
    if pin {
        if let Some(g) = geo {
            runs = insert_lock_line_breaks(runs, raw, g);
        }
    }
    // Pin inserts hard `\n` at lock line starts. wrap=none prevents host
    // reflow clipping for left text, but Word/PPT ignore jc/algn under
    // wrap=none — so non-left still takes host_wrap (square when aligned).
    // Lists always wrap so hanging / literal markers can reflow.
    let wrap = if bullet || numbered {
        true
    } else if pin && matches!(align, crate::ir::TextAlign::Left) {
        false
    } else {
        !running
            && align::host_wrap(
                geo,
                align,
                !pin && align::should_wrap_lock(geo, font_size, Some(raw)),
            )
    };
    let mut line_twips = metrics::line_spacing_twips(geo);
    if !wrap && line_twips.is_none() {
        // Host default line pitch (~12pt) clips 7–9pt text in one-line-tall
        // frames (pills, title rows). Pin exact spacing to the lock font.
        line_twips = Some(crate::coord::pt_to_twips(font_size).max(20));
    }
    let last_line_twips = if pin {
        Some(metrics::last_line_twips(font_size))
    } else {
        None
    };
    let mut height = rect.height;
    if let Some(ink) = align::lock_ink_height(geo, font_size) {
        if ink.0 > height.0 {
            height = ink;
        }
    }
    // List markers: paint as ordinary text (num_id stays 0 → no w:numPr).
    // Word/LibreOffice auto-numbering in floating wps boxes is unreliable
    // (unique-numId pools cap at 64; z-order emission reverses counters).
    if bullet {
        prepend_literal_bullet(&mut runs);
    } else if numbered {
        prepend_literal_number(&mut runs, list_start.max(1));
    }
    let x_emu = pt_to_emu(rect.x);
    let cx_emu = pt_to_emu(rect.width);
    if bullet || numbered {
        // Outer pad → bodyPr lIns; marker column → w:ind hanging. Literal
        // marker already inks in the hanging gutter on line one.
        l_ins_emu = metrics::list_outer_pad_emu(geo);
    }
    let hang_emu = if bullet || numbered {
        metrics::list_hanging_lock_emu(geo)
    } else {
        0
    };
    let vert_center = metrics::vert_center(geo, rect, font_size);
    if vert_center {
        t_ins_emu = 0;
    }
    Some(TextBox {
        node_id: node.id.clone(),
        x_emu,
        y_emu: pt_to_emu(rect.y),
        cx_emu,
        cy_emu: pt_to_emu(height),
        runs,
        align,
        bullet,
        numbered,
        num_id: 0,
        ilvl: node.depth.unwrap_or(0).min(8),
        l_ins_emu,
        hang_emu,
        t_ins_emu,
        r_ins_emu,
        b_ins_emu,
        line_twips,
        last_line_twips,
        para_line_twips: Vec::new(),
        para_after_twips: Vec::new(),
        vert_center,
        preserve_whitespace: node.preserve_whitespace == Some(true)
            || node.role == "code_block"
            || matches!(node.content, NodeContent::CodeBlock(_)),
        relative_height,
        fill_hex: None,
        fill_alpha: 255,
        wrap,
        corner_emu: 0,
        line_hex: None,
        line_alpha: 255,
        line_w_emu: 0,
        line_dash: crate::ir::LineDash::Solid,
    })
}

pub fn textbox_wml(tb: &TextBox) -> String {
    format!(
        r#"<root xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
{}</root>"#,
        ooxml::textbox_wsp_xml(tb, &BTreeMap::new())
    )
}

pub(crate) fn font_ctx(fonts: &BTreeMap<String, Vec<u8>>) -> FontCtx {
    FontCtx::new(fonts)
}

/// Lock paints bullets as decorative glyphs; export drops them. Native Word
/// numbering in floating text boxes often inks U+2022 as a 1–2px speck.
/// Emit the marker as a normal run (Arial so the glyph exists) + NBSP gap.
fn prepend_literal_bullet(runs: &mut Vec<TextRun>) {
    let Some(first) = runs.first() else {
        return;
    };
    let marker = TextRun {
        text: "•\u{00A0}".into(),
        font_name: "Arial".into(),
        sz_half_points: first.sz_half_points,
        bold: false,
        italic: false,
        underline: false,
        strike: false,
        color_hex: first.color_hex.clone(),
        hyperlink: None,
        script: crate::ir::ScriptPos::Baseline,
        tracking_twips: 0,
        field: None,
    };
    runs.insert(0, marker);
}

/// Same strategy as bullets: lock-derived index as plain text. Host `w:numPr`
/// across floating anchors renumbers by document order and breaks past the
/// fixed numId pool.
fn prepend_literal_number(runs: &mut Vec<TextRun>, n: u32) {
    let Some(first) = runs.first() else {
        return;
    };
    let marker = TextRun {
        text: format!("{n}.\u{00A0}"),
        font_name: first.font_name.clone(),
        sz_half_points: first.sz_half_points,
        bold: false,
        italic: false,
        underline: false,
        strike: false,
        color_hex: first.color_hex.clone(),
        hyperlink: None,
        script: crate::ir::ScriptPos::Baseline,
        tracking_twips: 0,
        field: None,
    };
    runs.insert(0, marker);
}

/// 1-based index per `list_id` stream (matches layout marker derivation for
/// flat depth-0 lists; same algorithm as `k2f_pptx::text::list_start_at`).
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

fn insert_lock_line_breaks(runs: Vec<TextRun>, text: &str, geo: &GeometryNode) -> Vec<TextRun> {
    let breaks: HashSet<usize> = align::lock_break_char_indices(geo, text)
        .into_iter()
        .collect();
    if breaks.is_empty() {
        return runs;
    }
    let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
    let start_byte = text.find(&joined).unwrap_or(0);
    let mut char_idx = text[..start_byte].chars().count();
    let mut out = Vec::new();
    for run in runs {
        let mut buf = String::new();
        for ch in run.text.chars() {
            if breaks.contains(&char_idx) && !buf.is_empty() {
                let mut head = run.clone();
                head.text = std::mem::take(&mut buf);
                out.push(head);
            }
            if breaks.contains(&char_idx) {
                let mut nl = run.clone();
                nl.text = "\n".into();
                out.push(nl);
            }
            buf.push(ch);
            char_idx += 1;
        }
        if !buf.is_empty() {
            let mut tail = run;
            tail.text = buf;
            out.push(tail);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ScriptPos;
    use k2f_core::{GlyphPosition, Pt};

    fn dummy_text_run(text: &str) -> TextRun {
        TextRun {
            text: text.into(),
            font_name: "Roboto".into(),
            sz_half_points: 24,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "F8FAFC".into(),
            hyperlink: None,
            script: ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }
    }

    #[test]
    fn pin_breaks_do_not_double_source_newlines() {
        let text = "Ab\nCd wrap";
        let geo = GeometryNode {
            id: "g".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(40_000),
            height: Pt(36_000),
            glyphs: vec![
                GlyphPosition {
                    glyph_id: 1,
                    cluster: 0,
                    x_offset: Pt(0),
                    y_offset: Pt(0),
                    x_advance: Pt(12_000),
                    y_advance: Pt(0),
                },
                GlyphPosition {
                    glyph_id: 1,
                    cluster: 1,
                    x_offset: Pt(12_000),
                    y_offset: Pt(0),
                    x_advance: Pt(12_000),
                    y_advance: Pt(0),
                },
                GlyphPosition {
                    glyph_id: 1,
                    cluster: 3,
                    x_offset: Pt(0),
                    y_offset: Pt(12_000),
                    x_advance: Pt(12_000),
                    y_advance: Pt(0),
                },
                GlyphPosition {
                    glyph_id: 1,
                    cluster: 4,
                    x_offset: Pt(12_000),
                    y_offset: Pt(12_000),
                    x_advance: Pt(12_000),
                    y_advance: Pt(0),
                },
                GlyphPosition {
                    glyph_id: 1,
                    cluster: 6,
                    x_offset: Pt(0),
                    y_offset: Pt(24_000),
                    x_advance: Pt(12_000),
                    y_advance: Pt(0),
                },
                GlyphPosition {
                    glyph_id: 1,
                    cluster: 7,
                    x_offset: Pt(12_000),
                    y_offset: Pt(24_000),
                    x_advance: Pt(12_000),
                    y_advance: Pt(0),
                },
            ],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        let out = insert_lock_line_breaks(vec![dummy_text_run(text)], text, &geo);
        let blob: String = out.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(blob, "Ab\nCd \nwrap", "got {blob:?}");
        assert_eq!(blob.matches('\n').count(), 2);
        assert!(!blob.contains("\n\n"));
    }

    #[test]
    fn list_start_at_counts_per_list_id() {
        let item = |id: &str, list_id: &str| SemanticNode {
            id: id.into(),
            role: "list_item".into(),
            list_id: Some(list_id.into()),
            marker_type: Some(ListMarkerType::Number),
            content: NodeContent::Text("x".into()),
            ..Default::default()
        };
        let root = SemanticNode {
            id: "root".into(),
            role: "document".into(),
            content: NodeContent::Container {
                children: vec![item("a", "ol"), item("b", "ol"), item("c", "other")],
            },
            ..Default::default()
        };
        let map = list_start_at(&root);
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
        assert_eq!(map.get("c"), Some(&1));
    }

    #[test]
    fn prepend_literal_number_uses_index() {
        let mut runs = vec![dummy_text_run("Dong")];
        prepend_literal_number(&mut runs, 64);
        assert_eq!(runs[0].text, "64.\u{00A0}");
        assert_eq!(runs[1].text, "Dong");
    }
}
