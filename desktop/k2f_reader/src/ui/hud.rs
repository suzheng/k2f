use super::chrome::{hud_color, overlay_label};
use super::font::draw_text;
use crate::AppState;

pub const HUD_HEIGHT: u32 = 24;
pub const EXPORT_ACTION_LABEL: &str = "Export";
const FONT_PX: u32 = 8;
const HUD_PAD: u32 = 8;
const HUD_FG: u32 = 0xF8FAFC;
const HUD_GAP: u32 = 12;

fn text_px(s: &str) -> u32 {
    (s.chars().count() as u32).saturating_mul(FONT_PX)
}

pub fn export_label_x(win_w: u32) -> u32 {
    win_w.saturating_sub(HUD_PAD.saturating_add(text_px(EXPORT_ACTION_LABEL)))
}

pub fn export_format_label_x(win_w: u32, format_label: &str) -> u32 {
    export_label_x(win_w).saturating_sub(HUD_GAP.saturating_add(text_px(format_label)))
}

pub fn copy_format_label_x(win_w: u32, copy_label: &str, format_label: &str) -> u32 {
    export_format_label_x(win_w, format_label)
        .saturating_sub(HUD_GAP.saturating_add(text_px(copy_label)))
}

/// Right-edge HUD control: run export in the current format.
pub fn export_hit(win_w: u32, win_h: u32, x: f64, y: f64) -> bool {
    let h = f64::from(HUD_HEIGHT.min(win_h));
    if y < 0.0 || y >= h {
        return false;
    }
    x >= f64::from(export_label_x(win_w)) && x < f64::from(win_w)
}

/// Cycle export format (K2F / PDF / MD / PNG / JPG), left of Export.
pub fn export_format_hit(
    win_w: u32,
    win_h: u32,
    format_label: &str,
    x: f64,
    y: f64,
) -> bool {
    let h = f64::from(HUD_HEIGHT.min(win_h));
    if y < 0.0 || y >= h {
        return false;
    }
    let x0 = export_format_label_x(win_w, format_label);
    let x1 = export_label_x(win_w).saturating_sub(HUD_GAP / 2);
    x >= f64::from(x0) && x < f64::from(x1)
}

/// Toggle clipboard format (Markdown vs plain), left of export format.
pub fn copy_format_hit(
    win_w: u32,
    win_h: u32,
    copy_label: &str,
    format_label: &str,
    x: f64,
    y: f64,
) -> bool {
    let h = f64::from(HUD_HEIGHT.min(win_h));
    if y < 0.0 || y >= h {
        return false;
    }
    let x0 = copy_format_label_x(win_w, copy_label, format_label);
    let x1 = export_format_label_x(win_w, format_label).saturating_sub(HUD_GAP / 2);
    x >= f64::from(x0) && x < f64::from(x1)
}

fn clip_text(s: &str, max_px: u32) -> String {
    s.chars().take((max_px / FONT_PX) as usize).collect()
}

pub fn draw_hud(buf: &mut [u32], width: u32, height: u32, app: &AppState) {
    let h = HUD_HEIGHT.min(height);
    let bg = hud_color(app);
    for y in 0..h {
        let row = y * width;
        for x in 0..width {
            buf[(row + x) as usize] = bg;
        }
    }
    let copy_label = app.copy_format().hud_label();
    let format_label = app.export_format().hud_label();
    let export_x = export_label_x(width);
    let format_x = export_format_label_x(width, format_label);
    let copy_x = copy_format_label_x(width, copy_label, format_label);
    let overlay_max = copy_x.saturating_sub(HUD_PAD.saturating_add(HUD_PAD));
    let label = clip_text(&overlay_label(app), overlay_max);
    draw_text(buf, width, height, HUD_PAD as i32, 8, &label, HUD_FG);
    draw_text(buf, width, height, copy_x as i32, 8, copy_label, HUD_FG);
    draw_text(buf, width, height, format_x as i32, 8, format_label, HUD_FG);
    draw_text(buf, width, height, export_x as i32, 8, EXPORT_ACTION_LABEL, HUD_FG);
}
