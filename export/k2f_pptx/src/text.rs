use crate::coord::pt_to_emu;
use crate::ir::{TextAlign, TextBox, TextRun};
use k2f_core::{
    ListMarkerType, Modifier, NodeContent, Pt, Rect, SemanticNode, TextGlyphRun, TextPaintStyle,
};
use k2f_paint::parse_hex_rgba;
use std::collections::BTreeMap;
use ttf_parser::{name_id, Face, Language};

pub(crate) struct FontCtx {
    default_family: String,
}

impl FontCtx {
    pub(crate) fn new(fonts: &BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            default_family: embedded_family(fonts).unwrap_or_else(|| "Roboto".into()),
        }
    }

    fn typeface(&self, family: &str) -> String {
        if family == "default" {
            self.default_family.clone()
        } else {
            family.to_string()
        }
    }
}

pub(crate) fn textbox_from_draw(
    node: &SemanticNode,
    rect: &Rect,
    paint_runs: &[TextGlyphRun],
    fonts: &FontCtx,
) -> Option<TextBox> {
    if node.role == "math" || matches!(node.content, NodeContent::Math(_)) {
        return None;
    }
    let text = k2f_core::node_text(node)?;
    if text.is_empty() {
        return None;
    }
    let style = paint_runs
        .first()
        .map(|r| r.style.clone())
        .unwrap_or_else(fallback_style);
    let font_name = fonts.typeface(&style.font_family);
    let runs = split_runs(text, &style, &node.modifiers, &font_name);
    if runs.is_empty() {
        return None;
    }
    let numbered = node.marker_type == Some(ListMarkerType::Number);
    let bullet = node.role == "list_item" || node.marker_type.is_some();
    Some(TextBox {
        node_id: node.id.clone(),
        x_emu: pt_to_emu(rect.x),
        y_emu: pt_to_emu(rect.y),
        cx_emu: pt_to_emu(rect.width),
        cy_emu: pt_to_emu(rect.height),
        runs,
        align: TextAlign::Left,
        bullet,
        numbered,
        preserve_whitespace: node.preserve_whitespace == Some(true) || node.role == "code_block",
    })
}

pub(crate) fn cell_runs(
    node: Option<&SemanticNode>,
    paint_runs: &[TextGlyphRun],
    fonts: &FontCtx,
    header_bold: bool,
) -> (Vec<TextRun>, bool) {
    let Some(node) = node else {
        return (Vec::new(), false);
    };
    let Some(text) = k2f_core::node_text(node) else {
        return (Vec::new(), false);
    };
    let mut style = paint_runs
        .first()
        .map(|r| r.style.clone())
        .unwrap_or_else(|| table_fallback_style(header_bold));
    if paint_runs.is_empty() && header_bold {
        style.bold = true;
    }
    let font_name = fonts.typeface(&style.font_family);
    let preserve = node.preserve_whitespace == Some(true) || node.role == "code_block";
    (
        split_runs(text, &style, &node.modifiers, &font_name),
        preserve,
    )
}

fn fallback_style() -> TextPaintStyle {
    TextPaintStyle {
        font_family: "default".into(),
        font_size: Pt(12_000),
        color: "#000000".into(),
        bold: false,
        italic: false,
        strikethrough: false,
        underline: false,
    }
}

fn table_fallback_style(bold: bool) -> TextPaintStyle {
    TextPaintStyle {
        font_family: "default".into(),
        font_size: Pt(11_000),
        color: "#000000".into(),
        bold,
        italic: false,
        strikethrough: false,
        underline: false,
    }
}

fn split_runs(
    text: &str,
    style: &TextPaintStyle,
    modifiers: &[Modifier],
    font_name: &str,
) -> Vec<TextRun> {
    let base = run_from_style("", style, font_name);
    let mut cuts = vec![0usize, text.len()];
    for m in modifiers {
        let [s, e] = m.range;
        if s <= e && e <= text.len() && text.is_char_boundary(s) && text.is_char_boundary(e) {
            cuts.push(s);
            cuts.push(e);
        }
    }
    cuts.sort_unstable();
    cuts.dedup();
    let mut out = Vec::new();
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        if a >= b {
            continue;
        }
        let Ok(piece) = std::str::from_utf8(&text.as_bytes()[a..b]) else {
            continue;
        };
        if piece.is_empty() {
            continue;
        }
        let mut run = base.clone();
        run.text = piece.to_string();
        for m in modifiers {
            let [s, e] = m.range;
            if s <= a && b <= e {
                apply_modifier(&mut run, m);
            }
        }
        out.push(run);
    }
    out
}

fn run_from_style(text: &str, style: &TextPaintStyle, font_name: &str) -> TextRun {
    let sz = (style.font_size.0 / 10).clamp(100, i32::MAX as i128) as i32;
    TextRun {
        text: text.to_string(),
        font_name: font_name.to_string(),
        sz_hundredths_pt: sz,
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        strike: style.strikethrough,
        color_hex: color_hex(&style.color),
        hyperlink: None,
    }
}

fn apply_modifier(run: &mut TextRun, m: &Modifier) {
    match m.mod_type.as_str() {
        "emphasis" => run.bold = true,
        "underline" => run.underline = true,
        "strikethrough" => run.strike = true,
        "link" => run.hyperlink = Some(m.intent.clone()),
        "subscript" | "superscript" => run.italic = true,
        "math" | "syntax_highlight" => {}
        _ => {}
    }
}

fn color_hex(color: &str) -> String {
    parse_hex_rgba(color)
        .map(|[r, g, b, _]| format!("{r:02X}{g:02X}{b:02X}"))
        .unwrap_or_else(|| "000000".into())
}

fn embedded_family(fonts: &BTreeMap<String, Vec<u8>>) -> Option<String> {
    fonts
        .get("default")
        .or_else(|| fonts.values().next())
        .and_then(|b| family_from_bytes(b))
}

fn family_from_bytes(data: &[u8]) -> Option<String> {
    let face = Face::parse(data, 0).ok()?;
    let mut fallback = None;
    for name in face.names() {
        if name.name_id != name_id::FAMILY || !name.is_unicode() {
            continue;
        }
        let Some(s) = name.to_string() else {
            continue;
        };
        if name.language() == Language::English_UnitedStates {
            return Some(s);
        }
        if fallback.is_none() {
            fallback = Some(s);
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emphasis_splits_and_sets_bold() {
        let style = fallback_style();
        let mods = [Modifier {
            range: [0, 5],
            mod_type: "emphasis".into(),
            intent: "critical".into(),
        }];
        let runs = split_runs("Hello world", &style, &mods, "Roboto");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].text, "Hello");
        assert!(runs[0].bold);
        assert_eq!(runs[1].text, " world");
        assert!(!runs[1].bold);
    }
}
