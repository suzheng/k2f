//! Empty window: toolbar Open + centered CTA. No document, no lock pixels.

use super::blit::LETTERBOX;
use super::draw::{fill_rect, fill_round_rect, hline, Rect};
use super::font::{draw_text_centered, draw_text_px, em_height, text_width_px, BODY_PX, TITLE_PX};
use super::hud::{
    dip, toolbar_layout_at, ChromeHit, EXPORT_ACTION_LABEL, STATUS_HEIGHT, TOOLBAR_HEIGHT,
};

const TOOLBAR_BG: u32 = 0x3A3A3C;
const CANVAS: u32 = 0xFBFBFD;
const TITLE: u32 = 0xF5F5F7;
const HINT: u32 = 0x6E6E73;
const ERROR: u32 = 0x9B1C1C;
const PRIMARY: u32 = 0x007AFF;
const PRIMARY_FG: u32 = 0xFFFFFF;
const DIVIDER: u32 = 0xE8E8ED;
const STATUS_FG: u32 = 0x8E8E93;
const PAD_X: u32 = 20;
const RADIUS: u32 = 8;
const OPEN_LABEL: &str = "Open";

fn layout(win_w: u32, scale: f32) -> super::hud::ToolbarLayout {
    toolbar_layout_at(win_w, EXPORT_ACTION_LABEL, "100%", scale)
}

pub fn cta_rect(win_w: u32, win_h: u32, scale: f32) -> Rect {
    let w = dip(240, scale).min(win_w.saturating_sub(dip(48, scale)));
    let h = dip(40, scale);
    let tb = dip(TOOLBAR_HEIGHT, scale);
    let status = dip(STATUS_HEIGHT, scale);
    let stage_h = win_h.saturating_sub(tb + status);
    let x = (win_w.saturating_sub(w) / 2) as i32;
    let y = tb as i32 + (stage_h.saturating_sub(h) / 2) as i32;
    Rect::new(x, y, w, h)
}

pub fn hit(win_w: u32, win_h: u32, x: f64, y: f64, scale: f32) -> Option<ChromeHit> {
    if layout(win_w, scale).open.contains(x, y) {
        return Some(ChromeHit::Open);
    }
    if cta_rect(win_w, win_h, scale).contains(x, y) {
        return Some(ChromeHit::Open);
    }
    None
}

pub fn compose_frame(
    win_w: u32,
    win_h: u32,
    error: Option<&str>,
    hover: Option<ChromeHit>,
    pressed: Option<ChromeHit>,
    scale: f32,
) -> Vec<u32> {
    let n = win_w as usize * win_h as usize;
    let mut buf = vec![LETTERBOX; n];
    fill_rect(
        &mut buf,
        win_w,
        win_h,
        Rect::new(0, 0, win_w, win_h),
        CANVAS,
    );
    let tb = dip(TOOLBAR_HEIGHT, scale).min(win_h);
    fill_rect(
        &mut buf,
        win_w,
        win_h,
        Rect::new(0, 0, win_w, tb),
        TOOLBAR_BG,
    );
    let pad = dip(PAD_X, scale) as i32;
    let body = BODY_PX * scale;
    let title_px = TITLE_PX * scale;
    let open = layout(win_w, scale).open;
    let open_bg = if pressed == Some(ChromeHit::Open) {
        0x0066D6
    } else if hover == Some(ChromeHit::Open) {
        0x1A86FF
    } else {
        PRIMARY
    };
    fill_round_rect(&mut buf, win_w, win_h, open, dip(RADIUS, scale), open_bg);
    draw_text_centered(&mut buf, win_w, win_h, open, OPEN_LABEL, PRIMARY_FG, body);

    let title_x = open.x + open.w as i32 + dip(12, scale) as i32;
    let title_y = (tb as i32 - em_height(title_px) as i32) / 2;
    draw_text_px(
        &mut buf,
        win_w,
        win_h,
        title_x,
        title_y.max(0),
        "K2F Reader",
        TITLE,
        title_px,
    );

    let cta = cta_rect(win_w, win_h, scale);
    fill_round_rect(&mut buf, win_w, win_h, cta, dip(RADIUS, scale), open_bg);
    draw_text_centered(
        &mut buf,
        win_w,
        win_h,
        cta,
        "Open a .K2F file",
        PRIMARY_FG,
        body,
    );

    let hint = shortcut_hint();
    let hint_y = cta.y + cta.h as i32 + dip(16, scale) as i32;
    let hint_x = (win_w as i32 - text_width_px(hint, body) as i32) / 2;
    draw_text_px(
        &mut buf,
        win_w,
        win_h,
        hint_x.max(pad),
        hint_y,
        hint,
        HINT,
        body,
    );

    if let Some(err) = error {
        let err_y = hint_y + em_height(body) as i32 + dip(12, scale) as i32;
        let max_w = win_w.saturating_sub(dip(PAD_X, scale) * 2);
        let shown = if text_width_px(err, body) > max_w {
            "Could not open that file."
        } else {
            err
        };
        let err_x = (win_w as i32 - text_width_px(shown, body) as i32) / 2;
        draw_text_px(
            &mut buf,
            win_w,
            win_h,
            err_x.max(pad),
            err_y,
            shown,
            ERROR,
            body,
        );
    }

    let status_h = dip(STATUS_HEIGHT, scale);
    if win_h > status_h {
        let sy = win_h.saturating_sub(status_h) as i32;
        fill_rect(
            &mut buf,
            win_w,
            win_h,
            Rect::new(0, sy, win_w, status_h),
            CANVAS,
        );
        hline(&mut buf, win_w, win_h, sy, 0, win_w, DIVIDER);
        let cap = body * 13.0 / 15.0;
        draw_text_px(
            &mut buf,
            win_w,
            win_h,
            pad,
            sy + (status_h as i32 - em_height(cap) as i32).max(0) / 2,
            "No document",
            STATUS_FG,
            cap,
        );
    }
    buf
}

fn shortcut_hint() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "File → Open  or  ⌘O"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Open  or  Ctrl+O"
    }
}
