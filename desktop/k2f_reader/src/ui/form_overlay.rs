//! Paint fill-mode field overlays on the lock raster (viewer chrome, not lock ops).

use super::coords::PageView;
use super::draw::{fill_rect, fill_round_rect, stroke_line, Rect};
use super::font::{draw_text_px, em_height};
use super::form_fill::{field_contains_pt, millipt_to_pt, FillState};
use k2f_core::{FormFieldKind, CHECKBOX_CHECKED};
use k2f_paint::FormFieldLoc;

const PAPER: u32 = 0xFBFBFD;
const INK: u32 = 0x1A1A1A;
const PLACEHOLDER: u32 = 0x8E8E93;
const ACTIVE: u32 = 0x007AFF;
const CHECK: u32 = 0x1A1A1A;

pub fn field_window_rect(view: &PageView, field: &FormFieldLoc) -> Rect {
    let (x0, y0) = view.pt_to_window(millipt_to_pt(field.x), millipt_to_pt(field.y));
    let (x1, y1) = view.pt_to_window(
        millipt_to_pt(field.x.saturating_add(field.width)),
        millipt_to_pt(field.y.saturating_add(field.height)),
    );
    let w = (x1 - x0).round().max(1.0) as u32;
    let h = (y1 - y0).round().max(1.0) as u32;
    Rect::new(x0.round() as i32, y0.round() as i32, w, h)
}

pub fn hit_field<'a>(
    fields: &'a [FormFieldLoc],
    views: &[PageView],
    x: f64,
    y: f64,
) -> Option<&'a FormFieldLoc> {
    for f in fields {
        let Some(view) = views.get(f.page) else {
            continue;
        };
        let (px, py) = view.window_to_pt(x, y);
        if field_contains_pt(f, px, py) {
            return Some(f);
        }
    }
    None
}

pub fn draw_fields(
    buf: &mut [u32],
    win_w: u32,
    win_h: u32,
    views: &[PageView],
    fields: &[FormFieldLoc],
    fill: &FillState,
    scale: f32,
) {
    if !fill.is_filling() {
        return;
    }
    for field in fields {
        let Some(view) = views.get(field.page) else {
            continue;
        };
        let rect = field_window_rect(view, field);
        if rect.w == 0 || rect.h == 0 {
            continue;
        }
        fill_rect(buf, win_w, win_h, rect, PAPER);
        let active = fill.active().is_some_and(|a| a.id == field.id);
        if active {
            stroke_rect(buf, win_w, win_h, rect, ACTIVE);
        }
        let value = fill.display_value(field);
        if field.kind.is_checkbox() {
            draw_checkbox(buf, win_w, win_h, rect, value == CHECKBOX_CHECKED);
            continue;
        }
        let pad = (4.0 * scale).round().max(2.0) as i32;
        let px = (rect.h as f32 * 0.55).clamp(11.0, 18.0) * scale.max(1.0).min(1.4);
        if value.is_empty() {
            if let Some(ph) = field.placeholder.as_deref() {
                draw_text_px(
                    buf,
                    win_w,
                    win_h,
                    rect.x + pad,
                    rect.y + (rect.h as i32 - em_height(px) as i32).max(0) / 2,
                    ph,
                    PLACEHOLDER,
                    px,
                );
            }
            continue;
        }
        draw_value(
            buf, win_w, win_h, rect, pad, &value, field.kind, INK, px, scale,
        );
    }
}

fn draw_value(
    buf: &mut [u32],
    win_w: u32,
    win_h: u32,
    rect: Rect,
    pad: i32,
    value: &str,
    kind: FormFieldKind,
    color: u32,
    px: f32,
    scale: f32,
) {
    let line_h = em_height(px).max(1) as i32 + (2.0 * scale).round() as i32;
    if kind != FormFieldKind::Multiline {
        let y = rect.y + (rect.h as i32 - em_height(px) as i32).max(0) / 2;
        draw_text_px(buf, win_w, win_h, rect.x + pad, y, value, color, px);
        return;
    }
    let mut y = rect.y + pad;
    let bottom = rect.y + rect.h as i32 - pad;
    for line in value.split('\n') {
        if y + line_h > bottom + line_h / 2 {
            break;
        }
        draw_text_px(buf, win_w, win_h, rect.x + pad, y, line, color, px);
        y += line_h;
    }
}

fn draw_checkbox(buf: &mut [u32], win_w: u32, win_h: u32, rect: Rect, on: bool) {
    let side = rect.w.min(rect.h).saturating_sub(4).max(8);
    let x = rect.x + (rect.w as i32 - side as i32) / 2;
    let y = rect.y + (rect.h as i32 - side as i32) / 2;
    let box_r = Rect::new(x, y, side, side);
    fill_round_rect(buf, win_w, win_h, box_r, 3, PAPER);
    stroke_rect(buf, win_w, win_h, box_r, CHECK);
    if on {
        let inset = (side / 4).max(2) as i32;
        stroke_line(
            buf,
            win_w,
            win_h,
            (x + inset) as f32,
            (y + side as i32 / 2) as f32,
            (x + side as i32 / 2) as f32,
            (y + side as i32 - inset) as f32,
            2.0,
            CHECK,
        );
        stroke_line(
            buf,
            win_w,
            win_h,
            (x + side as i32 / 2) as f32,
            (y + side as i32 - inset) as f32,
            (x + side as i32 - inset) as f32,
            (y + inset) as f32,
            2.0,
            CHECK,
        );
    }
}

fn stroke_rect(buf: &mut [u32], win_w: u32, win_h: u32, r: Rect, color: u32) {
    let x1 = r.x.saturating_add(r.w as i32).saturating_sub(1);
    let y1 = r.y.saturating_add(r.h as i32).saturating_sub(1);
    let t = 1.5;
    stroke_line(
        buf,
        win_w,
        win_h,
        r.x as f32,
        r.y as f32,
        x1 as f32,
        r.y as f32,
        t,
        color,
    );
    stroke_line(
        buf,
        win_w,
        win_h,
        r.x as f32,
        y1 as f32,
        x1 as f32,
        y1 as f32,
        t,
        color,
    );
    stroke_line(
        buf,
        win_w,
        win_h,
        r.x as f32,
        r.y as f32,
        r.x as f32,
        y1 as f32,
        t,
        color,
    );
    stroke_line(
        buf,
        win_w,
        win_h,
        x1 as f32,
        r.y as f32,
        x1 as f32,
        y1 as f32,
        t,
        color,
    );
}
