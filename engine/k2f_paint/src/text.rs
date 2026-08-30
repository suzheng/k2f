use k2f_core::{GeometryNode, Rect, TextGlyphRun};
use std::collections::HashMap;
use tiny_skia::{Color, FillRule, LineJoin, Paint, Path, Pixmap, Stroke, Transform};
use ttf_parser::{Face, GlyphId, OutlineBuilder};

use crate::error::PaintError;
use crate::geo_index::geo_for_op;
use crate::glyphs::{face_for, placed_glyphs, PlacedGlyph};

pub(crate) fn draw_text(
    pixmap: &mut Pixmap,
    faces: &HashMap<String, Face<'_>>,
    geo_by_id: &HashMap<String, Vec<&GeometryNode>>,
    node_id: &str,
    rect: &Rect,
    runs: &[TextGlyphRun],
    scale: f32,
) -> Result<(), PaintError> {
    let geo = geo_for_op(geo_by_id, node_id, rect)
        .ok_or_else(|| PaintError::MissingGeometry(node_id.to_string()))?;
    for g in placed_glyphs(faces, geo, rect, runs)? {
        fill_glyph(pixmap, faces, &g, scale);
    }
    Ok(())
}

fn fill_glyph(pixmap: &mut Pixmap, faces: &HashMap<String, Face<'_>>, g: &PlacedGlyph, scale: f32) {
    let Some(face) = face_for(faces, &g.font_family) else {
        return;
    };
    if g.font_size_pt <= 0.0 || g.units_per_em == 0.0 {
        return;
    }
    let [r, green, b, a] = g.rgba;
    if a == 0 {
        return;
    }
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(r, green, b, a));
    let px_per_font_unit = ((g.font_size_pt / g.units_per_em) * scale as f64) as f32;
    let mut pb = TinySkiaOutlineBuilder::new(
        px_per_font_unit,
        (g.origin_x_pt * scale as f64) as f32,
        (g.origin_y_pt * scale as f64) as f32,
        g.italic,
    );
    if face.outline_glyph(GlyphId(g.glyph_id), &mut pb).is_some() {
        if let Some(path) = pb.finish() {
            pixmap.as_mut().fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
            let stroke_px = (g.synthetic_bold_stroke_pt() * scale as f64) as f32;
            if stroke_px > 0.0 {
                let mut stroke = Stroke::default();
                stroke.width = stroke_px;
                stroke.line_join = LineJoin::Round;
                pixmap
                    .as_mut()
                    .stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
    }
}

struct TinySkiaOutlineBuilder {
    pb: tiny_skia::PathBuilder,
    scale: f32,
    origin_x: f32,
    origin_y: f32,
    italic: bool,
}

impl TinySkiaOutlineBuilder {
    fn new(scale: f32, origin_x: f32, origin_y: f32, italic: bool) -> Self {
        Self {
            pb: tiny_skia::PathBuilder::new(),
            scale,
            origin_x,
            origin_y,
            italic,
        }
    }

    fn tx(&self, x_units: f32, y_units: f32) -> f32 {
        let slant_x = if self.italic { y_units * 0.2126 } else { 0.0 };
        self.origin_x + (x_units + slant_x) * self.scale
    }

    fn ty(&self, y_units: f32) -> f32 {
        self.origin_y - y_units * self.scale
    }

    fn finish(self) -> Option<Path> {
        self.pb.finish()
    }
}

impl OutlineBuilder for TinySkiaOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pb.move_to(self.tx(x, y), self.ty(y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.pb.line_to(self.tx(x, y), self.ty(y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.pb
            .quad_to(self.tx(x1, y1), self.ty(y1), self.tx(x, y), self.ty(y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.pb.cubic_to(
            self.tx(x1, y1),
            self.ty(y1),
            self.tx(x2, y2),
            self.ty(y2),
            self.tx(x, y),
            self.ty(y),
        );
    }

    fn close(&mut self) {
        self.pb.close();
    }
}
