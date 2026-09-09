use k2f_core::{Border, BorderEdge, BorderStyle, BoxDecoration, Fill, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

use crate::coord::pdf_y;
use crate::draw::PageDraw;
use crate::error::PdfError;

pub fn draw_box(
    page: &mut PageDraw,
    rect: &Rect,
    decoration: &BoxDecoration,
) -> Result<(), PdfError> {
    if decoration.shadow.is_some() {
        return Err(PdfError::RasterOp("shadow"));
    }
    page.note_box(rect);
    let fill = resolve_fill(decoration)?;
    let radius = decoration
        .corner_radius_pt
        .map(|r| r as f64 / 1000.0)
        .unwrap_or(0.0);

    if let Some(Fill::Solid { color }) = &fill {
        if let Some([r, g, b, a]) = parse_hex_rgba(color) {
            if a > 0 {
                set_fill(&mut page.content, r, g, b, a);
                rounded_rect(&mut page.content, page.page_h, rect, radius);
                page.content.fill_nonzero();
            }
        }
    } else if let Some(Fill::LinearGradient { .. }) = &fill {
        return Err(PdfError::RasterOp("linear_gradient"));
    }

    if let Some(border) = &decoration.border {
        stroke_border(page, rect, border, radius)?;
    }
    Ok(())
}

fn stroke_border(
    page: &mut PageDraw,
    rect: &Rect,
    border: &Border,
    radius: f64,
) -> Result<(), PdfError> {
    let Some([r, g, b, a]) = parse_hex_rgba(&border.color) else {
        return Ok(());
    };
    let width = (border.width_pt as f64 / 1000.0) as f32;
    if width <= 0.0 || a == 0 || border.edges.is_empty() {
        return Ok(());
    }
    set_stroke(&mut page.content, r, g, b, a);
    page.content.set_line_width(width);
    apply_dash(&mut page.content, border.style);

    if border.draws_all_four_edges() {
        rounded_rect(&mut page.content, page.page_h, rect, radius);
        page.content.stroke();
        return Ok(());
    }

    let x = rect.x.as_f64_pt() as f32;
    let w = rect.width.as_f64_pt() as f32;
    let y_bottom = pdf_y(page.page_h, rect.y.as_f64_pt() + rect.height.as_f64_pt());
    let y_top = pdf_y(page.page_h, rect.y.as_f64_pt());

    for edge in &border.edges {
        match edge {
            BorderEdge::Top => {
                // PDF y grows up; "top" of the box is higher on the page = y_top in PDF space?
                // pdf_y converts top-left document coords: document y=0 is top of page.
                // rect.y is top of box; bottom of box is rect.y + height.
                // pdf_y(page_h, doc_y) = page_h - doc_y
                // So top edge of box in PDF: pdf_y(page_h, rect.y) = higher PDF y value... wait
                // document top of box = rect.y (smaller doc y)
                // pdf_y(page_h, rect.y) = page_h - rect.y  (larger PDF y = higher on page) ✓
                page.content.move_to(x, y_top);
                page.content.line_to(x + w, y_top);
            }
            BorderEdge::Bottom => {
                page.content.move_to(x, y_bottom);
                page.content.line_to(x + w, y_bottom);
            }
            BorderEdge::Left => {
                page.content.move_to(x, y_bottom);
                page.content.line_to(x, y_top);
            }
            BorderEdge::Right => {
                page.content.move_to(x + w, y_bottom);
                page.content.line_to(x + w, y_top);
            }
        }
        page.content.stroke();
    }
    Ok(())
}

fn apply_dash(content: &mut pdf_writer::Content, style: BorderStyle) {
    match style {
        BorderStyle::Solid => {
            content.set_dash_pattern([], 0.0);
        }
        BorderStyle::Dashed => {
            content.set_dash_pattern([3.0, 2.0], 0.0);
        }
        BorderStyle::Dotted => {
            content.set_dash_pattern([1.0, 1.0], 0.0);
        }
    }
}

pub fn draw_placeholder(page: &mut PageDraw, rect: &Rect) {
    page.note_box(rect);
    set_fill(&mut page.content, 230, 233, 238, 255);
    rounded_rect(&mut page.content, page.page_h, rect, 0.0);
    page.content.fill_nonzero();
}

fn set_fill(content: &mut pdf_writer::Content, r: u8, g: u8, b: u8, _a: u8) {
    content.set_fill_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
}

fn set_stroke(content: &mut pdf_writer::Content, r: u8, g: u8, b: u8, _a: u8) {
    content.set_stroke_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
}

fn rounded_rect(content: &mut pdf_writer::Content, page_h: f64, rect: &Rect, radius_pt: f64) {
    let x = rect.x.as_f64_pt() as f32;
    let w = rect.width.as_f64_pt() as f32;
    let h = rect.height.as_f64_pt() as f32;
    let y = pdf_y(page_h, rect.y.as_f64_pt() + rect.height.as_f64_pt());
    let r = (radius_pt as f32).max(0.0).min(w * 0.5).min(h * 0.5);
    if r == 0.0 {
        content.rect(x, y, w, h);
        return;
    }
    let mut p = Path {
        content,
        x: x + r,
        y,
    };
    p.move_to(x + r, y);
    p.line_to(x + w - r, y);
    p.quad_to(x + w, y, x + w, y + r);
    p.line_to(x + w, y + h - r);
    p.quad_to(x + w, y + h, x + w - r, y + h);
    p.line_to(x + r, y + h);
    p.quad_to(x, y + h, x, y + h - r);
    p.line_to(x, y + r);
    p.quad_to(x, y, x + r, y);
    p.content.close_path();
}

struct Path<'a> {
    content: &'a mut pdf_writer::Content,
    x: f32,
    y: f32,
}

impl Path<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.content.move_to(x, y);
        self.x = x;
        self.y = y;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.content.line_to(x, y);
        self.x = x;
        self.y = y;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let cx1 = self.x + (x1 - self.x) * (2.0 / 3.0);
        let cy1 = self.y + (y1 - self.y) * (2.0 / 3.0);
        let cx2 = x + (x1 - x) * (2.0 / 3.0);
        let cy2 = y + (y1 - y) * (2.0 / 3.0);
        self.content.cubic_to(cx1, cy1, cx2, cy2, x, y);
        self.x = x;
        self.y = y;
    }
}
