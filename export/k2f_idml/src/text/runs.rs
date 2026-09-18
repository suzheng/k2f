use super::font::FontCtx;
use super::tracking::tracking_em;
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
        return split_piece(
            text,
            0,
            text.len(),
            &fallback_style(),
            modifiers,
            fonts,
            0,
            false,
        );
    }
    let default_style = &paint_runs[0].style;
    let mut spans: Vec<(usize, usize, &TextPaintStyle, Vec<&GlyphPosition>)> = Vec::new();
    for pr in paint_runs {
        let Some((bs, be)) = paint_byte_range(text, pr, geo) else {
            continue;
        };
        if bs < be {
            let glyphs = paint_glyphs(pr, geo);
            spans.push((bs, be, &pr.style, glyphs));
        }
    }
    spans.sort_by_key(|(bs, _, _, _)| *bs);
    let mut out = Vec::new();
    // Page tokens must keep the full source string so `{{page_*}}` can be
    // rewritten after paint-run splits.
    let keep_full = text.contains("{{page_current}}") || text.contains("{{page_total}}");
    if let Some(&(first, _, _, _)) = spans.first() {
        let mut pos = if keep_full { 0 } else { first };
        for (bs, be, style, glyphs) in spans {
            if be <= pos {
                continue;
            }
            let start = bs.max(pos);
            if start > pos {
                // Gap between paint spans still uses a paint style; don't
                // re-map emphasis on top (a missed space would become bold).
                out.extend(split_piece(
                    text,
                    pos,
                    start,
                    default_style,
                    modifiers,
                    fonts,
                    0,
                    true,
                ));
            }
            // Letter-spaced display titles often fill the lock box *and* contain
            // spaces. Skipping tracking then left-aligns a condensed line.
            // Justified wrap still lands near 0: tracking_em is the median extra.
            let tracking = tracking_em(&glyphs, style, fonts);
            out.extend(split_piece(
                text, start, be, style, modifiers, fonts, tracking, true,
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
                0,
                false,
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
            0,
            true,
        ));
    }
    out
}

fn paint_glyphs<'a>(pr: &TextGlyphRun, geo: Option<&'a GeometryNode>) -> Vec<&'a GlyphPosition> {
    let Some(geo) = geo else {
        return Vec::new();
    };
    let start = pr.glyph_range[0];
    let end = pr.glyph_range[1].min(geo.glyphs.len());
    if start >= end {
        return Vec::new();
    }
    geo.glyphs[start..end]
        .iter()
        .filter(|g| g.cluster != GlyphPosition::CLUSTER_NOT_SOURCE)
        .collect()
}

fn paint_byte_range(
    text: &str,
    pr: &TextGlyphRun,
    geo: Option<&GeometryNode>,
) -> Option<(usize, usize)> {
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
    Some((char_to_byte(text, cs), char_to_byte(text, ce)))
}

fn split_piece(
    text: &str,
    bs: usize,
    be: usize,
    style: &TextPaintStyle,
    modifiers: &[Modifier],
    fonts: &FontCtx,
    tracking: i32,
    from_paint: bool,
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
                apply_modifier(&mut run, m, style, from_paint);
            }
        }
        out.push(run);
    }
    out
}

fn run_from_style(text: &str, style: &TextPaintStyle, fonts: &FontCtx, tracking: i32) -> TextRun {
    TextRun {
        text: text.to_string(),
        font_name: fonts.typeface(&style.font_family),
        size_pt: crate::millipt_to_pt(style.font_size.0).max(1.0),
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        strike: style.strikethrough,
        color_hex: color_hex(&style.color),
        hyperlink: None,
        script: ScriptPos::Baseline,
        leading_pt: None,
        auto_page_number: false,
        tracking,
        face_style: fonts.face_style(&style.font_family),
    }
}

fn apply_modifier(run: &mut TextRun, m: &Modifier, style: &TextPaintStyle, from_paint: bool) {
    match m.mod_type.as_str() {
        "emphasis" => {
            // Lock paint already applied theme.modifiers.styles.emphasis
            // (`emphasis` → italic, `strong` → bold). Re-mapping any non-italic
            // intent to bold on top of that turns italic table cells into
            // faux-bold. Links/script still overlay; face comes from paint.
            if from_paint {
                return;
            }
            match m.intent.as_str() {
                "italic" => run.italic = true,
                _ => {
                    if !run.bold {
                        run.bold = true;
                    }
                }
            }
        }
        "underline" => {
            if !from_paint {
                run.underline = true;
            }
        }
        "strikethrough" => {
            if !from_paint {
                run.strike = true;
            }
        }
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

pub(crate) fn color_hex(color: &str) -> String {
    parse_hex_rgba(color)
        .map(|[r, g, b, _]| format!("{r:02X}{g:02X}{b:02X}"))
        .unwrap_or_else(|| "000000".into())
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
    use k2f_core::{GeometryNode, TextGlyphRun};
    use std::collections::BTreeMap;

    fn fonts() -> FontCtx {
        FontCtx::new(&BTreeMap::new())
    }

    fn paint(italic: bool) -> TextPaintStyle {
        TextPaintStyle {
            font_family: "default".into(),
            font_size: Pt(8500),
            color: "#22262E".into(),
            bold: false,
            italic,
            strikethrough: false,
            underline: false,
        }
    }

    fn glyph(cluster: u32, x: i128) -> GlyphPosition {
        GlyphPosition {
            glyph_id: 1,
            cluster,
            x_offset: Pt(x),
            y_offset: Pt(0),
            x_advance: Pt(8000),
            y_advance: Pt(0),
        }
    }

    #[test]
    fn theme_emphasis_on_italic_paint_is_not_bold() {
        let text = "92% Complete — Final validation underway";
        let n = text.chars().count();
        let prefix = 15; // "92% Complete — "
        let glyphs: Vec<GlyphPosition> = (0..n).map(|i| glyph(i as u32, i as i128 * 8000)).collect();
        let geo = GeometryNode {
            id: "c".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(156_900),
            height: Pt(36_950),
            glyphs,
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        let paint_runs = vec![
            TextGlyphRun {
                glyph_range: [0, prefix],
                style: paint(false),
            },
            TextGlyphRun {
                glyph_range: [prefix, n],
                style: paint(true),
            },
        ];
        let modifiers = vec![Modifier {
            range: [17, text.len()],
            mod_type: "emphasis".into(),
            intent: "emphasis".into(),
        }];
        let runs = runs_from_paint(text, &paint_runs, &modifiers, Some(&geo), &fonts());
        let italic: Vec<_> = runs.iter().filter(|r| r.italic).collect();
        assert!(!italic.is_empty(), "expected italic suffix, got {runs:?}");
        for r in &italic {
            assert!(
                !r.bold,
                "theme emphasis→italic must not become faux-bold, got {r:?}"
            );
        }
        for r in &runs {
            if !r.italic {
                assert!(!r.bold, "unemphasized prefix must stay regular, got {r:?}");
            }
        }
    }

    #[test]
    fn emphasis_without_paint_still_bolds() {
        let text = "note";
        let modifiers = vec![Modifier {
            range: [0, 4],
            mod_type: "emphasis".into(),
            intent: "emphasis".into(),
        }];
        let runs = runs_from_paint(text, &[], &modifiers, None, &fonts());
        assert_eq!(runs.len(), 1);
        assert!(runs[0].bold);
        assert!(!runs[0].italic);
    }
}
