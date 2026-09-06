mod align;
mod fields;
mod font;
mod metrics;
mod runs;
mod tracking;

use crate::coord::pt_to_emu;
use crate::ir::TextBox;
use crate::ooxml;
use k2f_core::{GeometryNode, ListMarkerType, NodeContent, Rect, SemanticNode, TextGlyphRun};
use std::collections::BTreeMap;

pub use align::infer_text_align;
pub(crate) use fields::has_page_tokens;
pub(crate) use font::FontCtx;
pub(crate) use runs::runs_from_paint;

pub fn textbox_from_draw(
    node: &SemanticNode,
    rect: &Rect,
    paint_runs: &[TextGlyphRun],
    geo: Option<&GeometryNode>,
    fonts: &BTreeMap<String, Vec<u8>>,
) -> Option<TextBox> {
    textbox_from_draw_ctx(node, rect, paint_runs, geo, &FontCtx::new(fonts), 0)
}

pub(crate) fn textbox_from_draw_ctx(
    node: &SemanticNode,
    rect: &Rect,
    paint_runs: &[TextGlyphRun],
    geo: Option<&GeometryNode>,
    fonts: &FontCtx,
    relative_height: u32,
) -> Option<TextBox> {
    if node.role == "math" || matches!(node.content, NodeContent::Math(_)) {
        return None;
    }
    let text = k2f_core::node_text(node)?;
    if text.is_empty() {
        return None;
    }
    let runs = runs::runs_from_paint(text, paint_runs, &node.modifiers, geo, fonts);
    if runs.is_empty() {
        return None;
    }
    let align = geo
        .map(|g| infer_text_align(g, text))
        .unwrap_or(crate::ir::TextAlign::Left);
    let (l_ins_emu, mut t_ins_emu, r_ins_emu, b_ins_emu) = metrics::insets(geo, align);
    let font_size = paint_runs
        .first()
        .map(|r| r.style.font_size)
        .unwrap_or(k2f_core::Pt(12_000));
    let wrap = align::should_wrap_lock(geo, font_size, Some(text));
    let mut line_twips = metrics::line_spacing_twips(geo);
    if !wrap && line_twips.is_none() {
        // Host default line pitch (~12pt) clips 7–9pt text in one-line-tall
        // frames (pills, title rows). Pin exact spacing to the lock font.
        line_twips = Some(crate::coord::pt_to_twips(font_size).max(20));
    }
    let numbered = node.marker_type == Some(ListMarkerType::Number);
    let bullet =
        !numbered && (node.role == "list_item" || node.marker_type == Some(ListMarkerType::Bullet));
    let vert_center = metrics::vert_center(geo, rect, font_size);
    if vert_center {
        t_ins_emu = 0;
    }
    Some(TextBox {
        node_id: node.id.clone(),
        x_emu: pt_to_emu(rect.x),
        y_emu: pt_to_emu(rect.y),
        cx_emu: pt_to_emu(rect.width),
        cy_emu: pt_to_emu(rect.height),
        runs,
        align,
        bullet,
        numbered,
        ilvl: node.depth.unwrap_or(0).min(8),
        l_ins_emu,
        t_ins_emu,
        r_ins_emu,
        b_ins_emu,
        line_twips: line_twips,
        vert_center,
        preserve_whitespace: node.preserve_whitespace == Some(true)
            || node.role == "code_block"
            || matches!(node.content, NodeContent::CodeBlock(_)),
        relative_height,
        fill_hex: None,
        wrap,
        corner_emu: 0,
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
