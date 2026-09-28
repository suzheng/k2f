//! Confirm before Cancel drops unsaved popover text.

use super::draw::{blend_over, fill_round_rect, Rect};
use super::font::{draw_text_px, em_height, BODY_PX};
use super::hud::dip;

const OVERLAY: u32 = 0x000000;
const OVERLAY_ALPHA: u8 = 96;
const PANEL_BG: u32 = 0xFFFFFF;
const PANEL_FG: u32 = 0x1A1A1A;
const BORDER: u32 = 0xE8E8ED;
const SECONDARY_BG: u32 = 0xF5F5F7;
const PRIMARY: u32 = 0x007AFF;
const PRIMARY_FG: u32 = 0xFFFFFF;
const QUESTION: &str = "Discard unsaved changes?";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscardHit {
    Keep,
    Discard,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DiscardPrompt {
    pub open: bool,
    pub pressed: Option<DiscardHit>,
}

struct Layout {
    panel: Rect,
    keep: Rect,
    discard: Rect,
}

fn layout(win_w: u32, win_h: u32, scale: f32) -> Layout {
    let pad = dip(20, scale);
    let gap = dip(12, scale);
    let title_h = dip(28, scale);
    let btn_h = dip(36, scale);
    let panel_w = dip(340, scale).min(win_w.saturating_sub(dip(48, scale)).max(160));
    let panel_h = pad + title_h + gap + btn_h + pad;
    let px = ((win_w.saturating_sub(panel_w)) / 2) as i32;
    let py = ((win_h.saturating_sub(panel_h)) / 2) as i32;
    let panel = Rect::new(px, py, panel_w, panel_h);
    let inner_w = panel_w.saturating_sub(pad * 2);
    let btn_w = inner_w.saturating_sub(gap) / 2;
    let btn_y = panel.y + (pad + title_h + gap) as i32;
    let keep = Rect::new(panel.x + pad as i32, btn_y, btn_w, btn_h);
    let discard = Rect::new(keep.x + btn_w as i32 + gap as i32, btn_y, btn_w, btn_h);
    Layout {
        panel,
        keep,
        discard,
    }
}

/// Backdrop and Keep editing both dismiss. Only the Discard button confirms.
pub fn hit_at(
    open: bool,
    win_w: u32,
    win_h: u32,
    x: f64,
    y: f64,
    scale: f32,
) -> Option<DiscardHit> {
    if !open {
        return None;
    }
    let l = layout(win_w, win_h, scale);
    if l.discard.contains(x, y) {
        Some(DiscardHit::Discard)
    } else {
        Some(DiscardHit::Keep)
    }
}

pub fn keep_point(win_w: u32, win_h: u32, scale: f32) -> (f64, f64) {
    let r = layout(win_w, win_h, scale).keep;
    center(r)
}

pub fn discard_point(win_w: u32, win_h: u32, scale: f32) -> (f64, f64) {
    let r = layout(win_w, win_h, scale).discard;
    center(r)
}

pub fn draw(buf: &mut [u32], win_w: u32, win_h: u32, prompt: &DiscardPrompt, scale: f32) {
    if !prompt.open {
        return;
    }
    for px in buf.iter_mut() {
        *px = blend_over(*px, OVERLAY, OVERLAY_ALPHA);
    }
    let l = layout(win_w, win_h, scale);
    let radius = dip(12, scale);
    fill_round_rect(buf, win_w, win_h, l.panel, radius, PANEL_BG);
    let pad = dip(20, scale) as i32;
    let body = BODY_PX * scale;
    let title_y = l.panel.y + pad + (dip(28, scale) as i32 - em_height(body) as i32).max(0) / 2;
    draw_text_px(
        buf,
        win_w,
        win_h,
        l.panel.x + pad,
        title_y,
        QUESTION,
        PANEL_FG,
        body,
    );
    paint_button(
        buf,
        win_w,
        win_h,
        l.keep,
        scale,
        "Keep editing",
        if prompt.pressed == Some(DiscardHit::Keep) {
            0xE4E4E8
        } else {
            SECONDARY_BG
        },
        PANEL_FG,
        true,
    );
    paint_button(
        buf,
        win_w,
        win_h,
        l.discard,
        scale,
        "Discard",
        if prompt.pressed == Some(DiscardHit::Discard) {
            0x0066D6
        } else {
            PRIMARY
        },
        PRIMARY_FG,
        false,
    );
}

fn paint_button(
    buf: &mut [u32],
    win_w: u32,
    win_h: u32,
    rect: Rect,
    scale: f32,
    label: &str,
    bg: u32,
    fg: u32,
    border: bool,
) {
    fill_round_rect(buf, win_w, win_h, rect, dip(8, scale), bg);
    if border {
        let x0 = rect.x as f32;
        let y0 = rect.y as f32;
        let x1 = (rect.x + rect.w as i32) as f32;
        let y1 = (rect.y + rect.h as i32) as f32;
        super::draw::stroke_line(buf, win_w, win_h, x0, y0, x1, y0, 1.0, BORDER);
        super::draw::stroke_line(buf, win_w, win_h, x1, y0, x1, y1, 1.0, BORDER);
        super::draw::stroke_line(buf, win_w, win_h, x0, y1, x1, y1, 1.0, BORDER);
        super::draw::stroke_line(buf, win_w, win_h, x0, y0, x0, y1, 1.0, BORDER);
    }
    let body = BODY_PX * scale;
    let tw = super::font::text_width_px(label, body) as i32;
    let tx = rect.x + (rect.w as i32 - tw).max(0) / 2;
    let ty = rect.y + (rect.h as i32 - em_height(body) as i32).max(0) / 2;
    draw_text_px(buf, win_w, win_h, tx, ty, label, fg, body);
}

fn center(rect: Rect) -> (f64, f64) {
    (
        rect.x as f64 + rect.w as f64 * 0.5,
        rect.y as f64 + rect.h as f64 * 0.5,
    )
}
