//! Underline / strikethrough geometry from lock `TextPaintStyle` flags.
//! Shared by PNG and PDF so both paint the same millipt strokes.

use k2f_core::{GeometryNode, GlyphPosition, Rect, TextGlyphRun};
use std::collections::HashMap;
use ttf_parser::Face;

use crate::glyphs::{baseline_y_pt, face_for};

const FALLBACK_UNDERLINE_POS_EM: f64 = -0.1;
const FALLBACK_STRIKE_POS_EM: f64 = 0.26;
const FALLBACK_THICKNESS_EM: f64 = 0.05;

/// Horizontal decoration stroke in document (y-down) points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextDecorationLine {
    pub x0_pt: f64,
    pub x1_pt: f64,
    pub y_pt: f64,
    pub thickness_pt: f64,
    pub rgba: [u8; 4],
}

pub fn decoration_lines(
    faces: &HashMap<String, Face<'_>>,
    geo: &GeometryNode,
    rect: &Rect,
    runs: &[TextGlyphRun],
) -> Vec<TextDecorationLine> {
    if geo.glyphs.is_empty() || runs.is_empty() {
        return Vec::new();
    }
    let baseline = baseline_y_pt(faces, rect, runs);
    let mut out = Vec::new();
    for run in runs {
        if !run.style.underline && !run.style.strikethrough {
            continue;
        }
        let rgba = crate::geom::parse_hex_rgba(&run.style.color).unwrap_or([0, 0, 0, 255]);
        if rgba[3] == 0 {
            continue;
        }
        let font_size_pt = run.style.font_size.as_f64_pt();
        if font_size_pt <= 0.0 {
            continue;
        }
        let face = face_for(faces, &run.style.font_family);
        let units = face.map(|f| f.units_per_em() as f64).unwrap_or(1000.0);
        if units == 0.0 {
            continue;
        }
        let [start, end] = run.glyph_range;
        let start = start.min(geo.glyphs.len());
        let end = end.min(geo.glyphs.len());
        if start >= end {
            continue;
        }
        let (underline, strike) = metrics(face, units);
        for span in line_spans(&geo.glyphs[start..end]) {
            let Some((x0, x1, line_baseline)) = span_box(rect, baseline, span) else {
                continue;
            };
            if run.style.underline {
                if let Some(line) =
                    stroke_at(line_baseline, font_size_pt, units, underline, rgba, x0, x1)
                {
                    out.push(line);
                }
            }
            if run.style.strikethrough {
                if let Some(line) =
                    stroke_at(line_baseline, font_size_pt, units, strike, rgba, x0, x1)
                {
                    out.push(line);
                }
            }
        }
    }
    out
}

fn metrics(face: Option<&Face<'_>>, units: f64) -> ((f64, f64), (f64, f64)) {
    let underline = face
        .and_then(|f| f.underline_metrics())
        .map(|m| (m.position as f64, m.thickness as f64))
        .filter(|(_, t)| *t > 0.0)
        .unwrap_or((
            FALLBACK_UNDERLINE_POS_EM * units,
            FALLBACK_THICKNESS_EM * units,
        ));
    let strike = face
        .and_then(|f| f.strikeout_metrics())
        .map(|m| (m.position as f64, m.thickness as f64))
        .filter(|(_, t)| *t > 0.0)
        .unwrap_or((
            FALLBACK_STRIKE_POS_EM * units,
            FALLBACK_THICKNESS_EM * units,
        ));
    (underline, strike)
}

/// Font `position` is +up from baseline; paint y grows down.
fn stroke_at(
    baseline_y_pt: f64,
    font_size_pt: f64,
    units: f64,
    (position, thickness): (f64, f64),
    rgba: [u8; 4],
    x0: f64,
    x1: f64,
) -> Option<TextDecorationLine> {
    let scale = font_size_pt / units;
    let thickness_pt = (thickness * scale).abs();
    if thickness_pt <= 0.0 || x1 <= x0 {
        return None;
    }
    Some(TextDecorationLine {
        x0_pt: x0,
        x1_pt: x1,
        y_pt: baseline_y_pt - position * scale,
        thickness_pt,
        rgba,
    })
}

fn line_spans(glyphs: &[GlyphPosition]) -> Vec<&[GlyphPosition]> {
    let mut out = Vec::new();
    let mut start = 0;
    for i in 1..glyphs.len() {
        if glyphs[i].y_offset != glyphs[start].y_offset {
            out.push(&glyphs[start..i]);
            start = i;
        }
    }
    if start < glyphs.len() {
        out.push(&glyphs[start..]);
    }
    out
}

fn span_box(rect: &Rect, baseline: f64, glyphs: &[GlyphPosition]) -> Option<(f64, f64, f64)> {
    let first = glyphs.first()?;
    let last = glyphs.last()?;
    let x0 = rect.x.as_f64_pt() + first.x_offset.as_f64_pt();
    let x1 = rect.x.as_f64_pt() + last.x_offset.as_f64_pt() + last.x_advance.as_f64_pt();
    if x1 <= x0 {
        return None;
    }
    Some((x0, x1, baseline + first.y_offset.as_f64_pt()))
}
