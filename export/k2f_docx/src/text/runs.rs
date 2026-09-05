use super::fields::expand_fields;
use super::font::FontCtx;
use super::tracking::tracking_twips;
use crate::ir::{ScriptPos, TextRun};
use k2f_core::{GeometryNode, GlyphPosition, Modifier, Pt, TextGlyphRun, TextPaintStyle};
use k2f_paint::parse_hex_rgba;

pub(crate) fn runs_from_paint(
    text: &str,
    paint_runs: &[TextGlyphRun],
    modifiers: &[Modifier],
    geo: Option<&GeometryNode>,
    fonts: &FontCtx,
) -> Vec<TextRun> {
    if paint_runs.is_empty() {
        let style = fallback_style();
        let out = split_piece(text, 0, text.len(), &style, modifiers, fonts, &[]);
        return expand_fields(out);
    }
    let default_style = &paint_runs[0].style;
    let mut spans: Vec<(usize, usize, &TextPaintStyle, Vec<&GlyphPosition>)> = Vec::new();
    for pr in paint_runs {
        let Some((bs, be, glyphs)) = paint_byte_range(text, pr, geo) else {
            continue;
        };
        if bs >= be {
            continue;
        }
        push_span(&mut spans, bs, be, &pr.style, glyphs);
    }
    spans.sort_by_key(|(bs, _, _, _)| *bs);
    let mut out = Vec::new();
    let mut pos = 0usize;
    for (bs, be, style, glyphs) in spans {
        if be <= pos {
            continue;
        }
        let start = bs.max(pos);
        if start > pos {
            out.extend(split_piece(
                text,
                pos,
                start,
                default_style,
                modifiers,
                fonts,
                &[],
            ));
        }
        out.extend(split_piece(
            text, start, be, style, modifiers, fonts, &glyphs,
        ));
        pos = pos.max(be);
    }
    if pos < text.len() {
        out.extend(split_piece(
            text,
            pos,
            text.len(),
            default_style,
            modifiers,
            fonts,
            &[],
        ));
    }
    if out.is_empty() {
        out.extend(split_piece(
            text,
            0,
            text.len(),
            default_style,
            modifiers,
            fonts,
            &[],
        ));
    }
    expand_fields(out)
}

fn push_span<'a>(
    spans: &mut Vec<(usize, usize, &'a TextPaintStyle, Vec<&'a GlyphPosition>)>,
    bs: usize,
    be: usize,
    style: &'a TextPaintStyle,
    glyphs: Vec<&'a GlyphPosition>,
) {
    spans.push((bs, be, style, glyphs));
}

fn paint_byte_range<'a>(
    text: &str,
    pr: &TextGlyphRun,
    geo: Option<&'a GeometryNode>,
) -> Option<(usize, usize, Vec<&'a GlyphPosition>)> {
    let geo = geo?;
    let start = pr.glyph_range[0];
    let end = pr.glyph_range[1].min(geo.glyphs.len());
    if start >= end {
        return None;
    }
    let glyphs: Vec<&GlyphPosition> = geo.glyphs[start..end]
        .iter()
        .filter(|g| g.cluster != GlyphPosition::CLUSTER_NOT_SOURCE)
        .collect();
    if glyphs.is_empty() {
        return None;
    }
    let cs = glyphs.iter().map(|g| g.cluster).min()? as usize;
    let ce = glyphs.iter().map(|g| g.cluster).max()? as usize + 1;
    let bs = char_to_byte(text, cs);
    let be = char_to_byte(text, ce);
    Some((bs, be, glyphs))
}

fn split_piece(
    text: &str,
    bs: usize,
    be: usize,
    style: &TextPaintStyle,
    modifiers: &[Modifier],
    fonts: &FontCtx,
    glyphs: &[&GlyphPosition],
) -> Vec<TextRun> {
    let mut cuts = vec![bs, be];
    for m in modifiers {
        let [s, e] = m.range;
        if s > bs && s < be {
            cuts.push(s);
        }
        if e > bs && e < be {
            cuts.push(e);
        }
    }
    cuts.sort_unstable();
    cuts.dedup();
    let tracking = tracking_twips(glyphs, style, fonts);
    let mut out = Vec::new();
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        if a >= b || b > text.len() {
            continue;
        }
        if !text.is_char_boundary(a) || !text.is_char_boundary(b) {
            continue;
        }
        let piece = text[a..b].replace('\u{FFFC}', "");
        if piece.is_empty() {
            continue;
        }
        let mut run = run_from_style(&piece, style, fonts, tracking);
        for m in modifiers {
            let [s, e] = m.range;
            if s <= a && b <= e {
                apply_modifier(&mut run, m, style);
            }
        }
        out.push(run);
    }
    out
}

fn run_from_style(
    text: &str,
    style: &TextPaintStyle,
    fonts: &FontCtx,
    tracking_twips: i32,
) -> TextRun {
    TextRun {
        text: text.to_string(),
        font_name: fonts.typeface(&style.font_family),
        sz_half_points: sz_half_points(style.font_size.0),
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        strike: style.strikethrough,
        color_hex: color_hex(&style.color),
        hyperlink: None,
        script: ScriptPos::Baseline,
        tracking_twips,
        field: None,
    }
}

fn apply_modifier(run: &mut TextRun, m: &Modifier, style: &TextPaintStyle) {
    match m.mod_type.as_str() {
        "emphasis" => {
            if !style.bold {
                run.bold = true;
            }
        }
        "underline" => run.underline = true,
        "strikethrough" => run.strike = true,
        "link" => run.hyperlink = Some(m.intent.clone()),
        "subscript" => {
            run.script = ScriptPos::Sub;
            run.italic = style.italic;
        }
        "superscript" => {
            run.script = ScriptPos::Super;
            run.italic = style.italic;
        }
        "syntax_highlight" | "math" => {}
        _ => {}
    }
}

fn sz_half_points(millipt: i128) -> i32 {
    i32::try_from(millipt / 500).unwrap_or(i32::MAX).max(1)
}

pub(crate) fn color_hex(color: &str) -> String {
    parse_hex_rgba(color)
        .map(|[r, g, b, _]| format!("{r:02X}{g:02X}{b:02X}"))
        .unwrap_or_else(|| {
            color
                .trim()
                .trim_start_matches('#')
                .chars()
                .take(6)
                .collect::<String>()
                .to_ascii_uppercase()
        })
}

fn char_to_byte(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
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
