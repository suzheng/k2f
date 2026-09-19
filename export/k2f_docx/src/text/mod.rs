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
    CHECKBOX_CHECKED,
};
use std::collections::{BTreeMap, HashSet};

pub use align::infer_text_align;
pub(crate) use fields::{expand_page_vars, has_page_tokens};
pub(crate) use font::FontCtx;
pub(crate) use metrics::{cell_h_insets_emu, line_spacing_twips, vert_center};
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
    let raw = office_source_text(node)?;
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
    let checkbox = matches!(
        &node.content,
        NodeContent::FormField(spec) if spec.kind.is_checkbox()
    );
    let numbered = node.marker_type == Some(ListMarkerType::Number);
    let bullet =
        !numbered && (node.role == "list_item" || node.marker_type == Some(ListMarkerType::Bullet));
    let align = if checkbox {
        crate::ir::TextAlign::Center
    } else if bullet || numbered {
        // Marker glyphs are CLUSTER_NOT_SOURCE, so gap inference sees a
        // left-padded body and calls short items Right/Center.
        crate::ir::TextAlign::Left
    } else {
        geo.map(|g| infer_text_align(g, raw))
            .unwrap_or(crate::ir::TextAlign::Left)
    };
    let para_align = geo
        .map(|g| align::infer_para_align(g, raw))
        .unwrap_or_default();
    let (mut l_ins_emu, t_ins_emu, r_ins_emu, b_ins_emu) = metrics::insets(geo, align);
    // Prefer the largest face (body), not paint_runs[0] — that is often a
    // leading superscript whose half-size would crush wrap/spacing heuristics.
    let font_size = paint_runs
        .iter()
        .map(|r| r.style.font_size)
        .max_by_key(|p| p.0.abs())
        .or_else(|| geo.map(|g| k2f_core::Pt(align::body_font_size(g))))
        .unwrap_or(k2f_core::Pt(12_000));
    let hang_emu = if bullet || numbered {
        metrics::list_hanging_lock_emu(geo)
    } else {
        0
    };
    // Hosts often ignore w:ind in drawing text boxes, so lock-wrapped lists
    // pin those wraps and pad continuation lines with NBSPs. U+2028 keeps
    // the wrap inside one paragraph when the item is later folded into a
    // card (a new `<w:p>` would sit under the marker).
    let list_wraps = (bullet || numbered)
        && geo
            .map(|g| !align::lock_break_char_indices(g, raw).is_empty())
            .unwrap_or(false);
    let pin = !running
        && (list_wraps
            || (!(bullet || numbered)
                && align::should_pin_lock_breaks(geo, font_size, Some(raw), align)));
    if pin {
        if let Some(g) = geo {
            runs = insert_lock_line_breaks(runs, raw, g);
        }
    }
    if bullet || numbered {
        // Pad wrap by the literal `•` / `{n}.` we prepend, not the lock
        // marker box (often 12–16pt). Hang-by-gutter made line 2 sit past
        // line-1 body — same iceberg IDML already fixed with marker width.
        pad_list_wrap_lines(&mut runs, &list_marker_text(bullet, numbered, list_start));
    }
    // Pin inserts hard breaks at lock line starts. wrap=none prevents host
    // reflow clipping for left text, but Word/PPT ignore jc/algn under
    // wrap=none — so non-left still takes host_wrap (square when aligned).
    // Lists stay wrap=none so the NBSP pad is not reflowed under the marker.
    let wrap = if checkbox {
        true
    } else if bullet || numbered {
        false
    } else if pin && matches!(align, crate::ir::TextAlign::Left) {
        false
    } else {
        !running
            && align::host_wrap_box(
                geo,
                align,
                &para_align,
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
    // Lock-pinned lines use wrap=square for center/right jc, but each pinned
    // paragraph must stay one host row (hero titles). Do not NBSP all
    // wrap=none one-liners — running headers/footers stay ordinary spaces.
    if pin && !list_wraps {
        runs = nobreak_spaces_runs(runs);
    }
    let x_emu = pt_to_emu(rect.x);
    let cx_emu = pt_to_emu(rect.width);
    if bullet || numbered {
        // Outer pad → bodyPr lIns. Wrap indent is NBSP pad, not w:ind —
        // hosts that honor hanging would double-count the marker column.
        l_ins_emu = metrics::list_outer_pad_emu(geo);
    }
    let vert_center = checkbox || metrics::vert_center(geo, rect, font_size);
    // Keep first-line pad on the IR even when centered. Tall boxes emit it as
    // tIns (Writer ignores wps anchor=ctr). Compact pills still zero tIns at
    // XML emit and use anchor=ctr.
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
        para_before_twips: Vec::new(),
        para_align,
        vert_center,
        preserve_whitespace: node.preserve_whitespace == Some(true)
            || node.role == "code_block"
            || matches!(node.content, NodeContent::CodeBlock(_)),
        relative_height,
        fill_hex: None,
        fill_alpha: 255,
        fill_blip: None,
        gradient: None,
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
        ooxml::textbox_wsp_xml(tb, &BTreeMap::new(), &BTreeMap::new(), &Default::default())
    )
}

pub(crate) fn font_ctx(fonts: &BTreeMap<String, Vec<u8>>) -> FontCtx {
    FontCtx::new(fonts)
}

/// Lock paints a checked box as `X`. `node_text` is the semantic value `"true"`,
/// which overflows a 12pt square (`tr` / garbled host glyphs).
pub(crate) fn office_source_text(node: &SemanticNode) -> Option<&str> {
    match &node.content {
        NodeContent::FormField(spec) if spec.kind.is_checkbox() => {
            (spec.value == CHECKBOX_CHECKED).then_some("X")
        }
        _ => k2f_core::node_text(node),
    }
}

/// Lock paints bullets as decorative glyphs; export drops them. Native Word
/// numbering in floating text boxes often inks U+2022 as a 1–2px speck.
/// Emit the marker as a normal run (Arial so the glyph exists) + NBSP gap.
fn nobreak_spaces_runs(mut runs: Vec<TextRun>) -> Vec<TextRun> {
    for r in &mut runs {
        if r.text.contains(' ') {
            r.text = r.text.replace(' ', "\u{00A0}");
        }
    }
    runs
}

fn prepend_literal_bullet(runs: &mut Vec<TextRun>) {
    let Some(first) = runs.first() else {
        return;
    };
    let marker = TextRun {
        text: "•\u{00A0}".into(),
        font_family_key: String::new(),
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
        font_family_key: first.font_family_key.clone(),
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

fn list_marker_text(bullet: bool, numbered: bool, list_start: u32) -> String {
    if numbered {
        format!("{}.\u{00A0}", list_start.max(1))
    } else if bullet {
        "•\u{00A0}".into()
    } else {
        String::new()
    }
}

/// Hosts ignore `w:ind` hanging inside many drawing text boxes. Pin lock wrap
/// points, then put NBSPs on continuation lines so wrap aligns with body, not
/// the literal `•` / `{n}.` marker. ~0.25em per NBSP.
///
/// Wrap breaks become U+2028 (line separator), not `\n`. Folded cards split
/// `\n` into a new paragraph, which resets the first-line gutter.
fn pad_list_wrap_lines(runs: &mut Vec<TextRun>, marker: &str) {
    isolate_line_breaks(runs);
    let pad = list_wrap_nbsp_for_marker(marker);
    if !pad.is_empty() {
        let mut out = Vec::with_capacity(runs.len() + 4);
        let mut after_break = false;
        for run in runs.drain(..) {
            if after_break {
                if !run.text.chars().all(|c| c == '\u{00A0}') {
                    let mut spacer = run.clone();
                    spacer.text = pad.clone();
                    out.push(spacer);
                }
                after_break = false;
            }
            let is_break = is_list_wrap_break(&run.text);
            out.push(run);
            if is_break {
                after_break = true;
            }
        }
        *runs = out;
    }
    for run in runs.iter_mut() {
        if run.text == "\n" {
            run.text = "\u{2028}".into();
        }
    }
}

fn is_list_wrap_break(text: &str) -> bool {
    text == "\n" || text == "\u{2028}"
}

fn isolate_line_breaks(runs: &mut Vec<TextRun>) {
    let mut out = Vec::with_capacity(runs.len());
    for run in runs.drain(..) {
        if !run.text.contains('\n') && !run.text.contains('\u{2028}') {
            out.push(run);
            continue;
        }
        let mut buf = String::new();
        for ch in run.text.chars() {
            if ch == '\n' || ch == '\u{2028}' {
                if !buf.is_empty() {
                    let mut head = run.clone();
                    head.text = std::mem::take(&mut buf);
                    out.push(head);
                }
                let mut br = run.clone();
                br.text = ch.to_string();
                out.push(br);
            } else {
                buf.push(ch);
            }
        }
        if !buf.is_empty() {
            let mut tail = run;
            tail.text = buf;
            out.push(tail);
        }
    }
    *runs = out;
}

/// Approximate the prepended marker in NBSP units so wrap lines start even
/// with line-1 body. Bullet/digit ≈ 0.5em (2), other glyphs ≈ 0.25em (1).
fn list_wrap_nbsp_for_marker(marker: &str) -> String {
    let n: usize = marker
        .chars()
        .map(|c| match c {
            '•' | '0'..='9' => 2,
            _ => 1,
        })
        .sum();
    "\u{00A0}".repeat(n.max(1))
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
            font_family_key: String::new(),
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

    #[test]
    fn list_wrap_lines_get_nbsp_pad_after_lock_break() {
        let mut runs = vec![
            dummy_text_run("Senior managers"),
            dummy_text_run("\n"),
            dummy_text_run("leads."),
        ];
        // Literal `•` + NBSP ≈ 3 NBSPs (~0.75em), not the lock gutter.
        pad_list_wrap_lines(&mut runs, "•\u{00A0}");
        let blob: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert!(
            blob.starts_with("Senior managers\u{2028}") && blob.ends_with("leads."),
            "{blob:?}"
        );
        assert!(
            blob.contains('\u{2028}'),
            "list wrap must use U+2028 so fold does not start a new paragraph, got {blob:?}"
        );
        let pad = blob
            .split('\u{2028}')
            .nth(1)
            .unwrap()
            .chars()
            .take_while(|c| *c == '\u{00A0}')
            .count();
        assert_eq!(pad, 3, "{blob:?}");
    }

    #[test]
    fn list_wrap_pad_matches_numbered_marker_not_lock_gutter() {
        assert_eq!(list_wrap_nbsp_for_marker("•\u{00A0}").chars().count(), 3);
        assert_eq!(list_wrap_nbsp_for_marker("1.\u{00A0}").chars().count(), 4);
        assert_eq!(list_wrap_nbsp_for_marker("10.\u{00A0}").chars().count(), 6);
    }
}
