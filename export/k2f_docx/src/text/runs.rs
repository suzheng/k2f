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
    // Page-token fields must keep the full source string so `{{page_*}}` can be
    // rewritten after paint-run splits. Ordinary nodes must not fill 0..first /
    // last..len — that re-inserts glyphs that belong to other pages.
    let keep_full = text.contains("{{page_current}}") || text.contains("{{page_total}}");
    if let Some(&(first, _, _, _)) = spans.first() {
        let mut pos = if keep_full { 0 } else { first };
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
        if keep_full && pos < text.len() {
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
        "emphasis" => match m.intent.as_str() {
            "italic" => run.italic = true,
            _ => run.bold = true,
        },
        "underline" => run.underline = true,
        "strikethrough" => run.strike = true,
        "link" => run.hyperlink = k2f_core::hyperlink_href(&m.intent).map(str::to_string),
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
    let hex = parse_hex_rgba(color)
        .map(|[r, g, b, _]| format!("{r:02X}{g:02X}{b:02X}"))
        .or_else(|| six_digit_hex(color))
        .unwrap_or_else(|| "000000".into());
    crate::xml::word_hex_color(&hex)
}

/// Word `ST_HexColor` is exactly 6 hex digits (or `auto`). Palette tokens
/// such as `ACCENT` must not be written into `w:color` / `a:srgbClr`.
pub(crate) fn six_digit_hex(color: &str) -> Option<String> {
    let t = color.trim().trim_start_matches('#');
    if t.len() == 6 && t.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(t.to_ascii_uppercase())
    } else {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_are_not_office_automatic() {
        assert_eq!(color_hex("#000000"), "000001");
        assert_eq!(color_hex("#FFFFFF"), "FFFFFE");
        assert_eq!(color_hex("#C1002A"), "C1002A");
        assert_eq!(
            color_hex("ACCENT"),
            "000001",
            "unresolved palette tokens must become a valid hex color, not ST_HexColor garbage"
        );
        assert_eq!(color_hex("black"), "000001");
    }

    fn dummy_run() -> TextRun {
        TextRun {
            text: "link".into(),
            font_name: "Roboto".into(),
            sz_half_points: 22,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "000001".into(),
            hyperlink: None,
            script: ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }
    }

    #[test]
    fn link_default_intent_is_not_a_hyperlink() {
        let mut run = dummy_run();
        apply_modifier(
            &mut run,
            &Modifier {
                range: [0, 4],
                mod_type: "link".into(),
                intent: "default".into(),
            },
            &fallback_style(),
        );
        assert!(run.hyperlink.is_none());
    }

    #[test]
    fn link_url_intent_is_a_hyperlink() {
        let mut run = dummy_run();
        apply_modifier(
            &mut run,
            &Modifier {
                range: [0, 4],
                mod_type: "link".into(),
                intent: "https://example.com".into(),
            },
            &fallback_style(),
        );
        assert_eq!(run.hyperlink.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn italic_emphasis_is_not_forced_bold() {
        let mut run = dummy_run();
        apply_modifier(
            &mut run,
            &Modifier {
                range: [0, 4],
                mod_type: "emphasis".into(),
                intent: "italic".into(),
            },
            &fallback_style(),
        );
        assert!(run.italic);
        assert!(!run.bold);
    }

    #[test]
    fn critical_emphasis_is_bold() {
        let mut run = dummy_run();
        apply_modifier(
            &mut run,
            &Modifier {
                range: [0, 4],
                mod_type: "emphasis".into(),
                intent: "critical".into(),
            },
            &fallback_style(),
        );
        assert!(run.bold);
        assert!(!run.italic);
    }
}
