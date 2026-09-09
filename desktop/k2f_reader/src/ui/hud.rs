use super::chrome::{
    banner_compact, banner_copy, banner_height_at, chrome_top_at, hud_color, page_status_label,
    zoom_label,
};
use super::draw::{blend_over, fill_rect, fill_round_rect, hline, stroke_line, Rect};
use super::font::{
    draw_text_centered, draw_text_px, ellipsize_to_width, em_height, text_width_px, wrap_text,
    BODY_PX, CAPTION_PX, TITLE_PX,
};
use crate::export::ExportFormat;
use crate::AppState;

pub const TOOLBAR_HEIGHT: u32 = 56;
pub const STATUS_HEIGHT: u32 = 28;
pub const BANNER_COMPACT_HEIGHT: u32 = 28;
pub const BANNER_ALERT_HEIGHT: u32 = 72;
pub const STAGE_PAD: u32 = 24;
pub const SCROLLBAR_WIDTH: u32 = 8;
pub const DEFAULT_INNER_W: u32 = 1280;
pub const DEFAULT_INNER_H: u32 = 820;
pub const MIN_INNER_W: u32 = 960;
pub const MIN_INNER_H: u32 = 640;

/// Top inset to page 0 at scroll 0 for a quiet document (no warning strip).
pub const HUD_HEIGHT: u32 = TOOLBAR_HEIGHT + STAGE_PAD;

/// Default split-button label (web `exportFormatLabel("k2f")`).
pub const EXPORT_ACTION_LABEL: &str = "Export as K2F";

const TOOLBAR_BG: u32 = 0x3A3A3C;
const CANVAS: u32 = 0xFBFBFD;
const TITLE: u32 = 0xF5F5F7;
const SUBTITLE: u32 = 0xC7C7CC;
const ICON: u32 = 0xF5F5F7;
const DIVIDER: u32 = 0xE8E8ED;
const STATUS_FG: u32 = 0x8E8E93;
const PRIMARY: u32 = 0x007AFF;
const PRIMARY_FG: u32 = 0xFFFFFF;
const PAD_X: u32 = 20;
const GAP: u32 = 8;
const ICON_BTN: u32 = 36;
const BTN_H: u32 = 36;
const RADIUS: u32 = 8;
const CARET_W: u32 = 28;
const MENU_ITEM_H: u32 = 34;
const MENU_PAD: u32 = 6;
const MENU_GAP: u32 = 4;
const MENU_BG: u32 = 0xFFFFFF;
const MENU_FG: u32 = 0x1A1A1A;
const MENU_HOVER: u32 = 0xF2F2F7;
const MENU_SHADOW: u32 = 0xD8D8DC;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromeHit {
    Open,
    ZoomOut,
    ZoomIn,
    Copy,
    Export,
    ExportMenu,
    ExportItem(usize),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChromePaint {
    pub hover: Option<ChromeHit>,
    pub pressed: Option<ChromeHit>,
    pub export_menu_open: bool,
}

pub struct ToolbarLayout {
    pub open: Rect,
    pub zoom_out: Rect,
    pub zoom_label: Rect,
    pub zoom_in: Rect,
    pub copy: Rect,
    pub export: Rect,
    pub export_caret: Rect,
}

pub fn dip(v: u32, scale: f32) -> u32 {
    (v as f32 * scale).round().max(1.0) as u32
}

pub fn toolbar_layout(
    win_w: u32,
    copy_label: &str,
    export_label: &str,
    zoom: &str,
) -> ToolbarLayout {
    toolbar_layout_at(win_w, copy_label, export_label, zoom, 1.0)
}

pub fn toolbar_layout_at(
    win_w: u32,
    copy_label: &str,
    export_label: &str,
    zoom: &str,
    scale: f32,
) -> ToolbarLayout {
    let pad = dip(PAD_X, scale) as i32;
    let gap = dip(GAP, scale) as i32;
    let btn_h = dip(BTN_H, scale);
    let icon = dip(ICON_BTN, scale);
    let tb = dip(TOOLBAR_HEIGHT, scale);
    let btn_y = (tb.saturating_sub(btn_h) / 2) as i32;
    let body = BODY_PX * scale;

    let caret_w = dip(CARET_W, scale);
    let export_w = text_width_px(export_label, body)
        .saturating_add(dip(28, scale))
        .max(dip(84, scale));
    let copy_w = icon
        .saturating_add(dip(10, scale))
        .saturating_add(text_width_px(copy_short(copy_label), body));
    let zoom_w = text_width_px(zoom, body).max(dip(44, scale));
    let open_w = text_width_px("Open", body)
        .saturating_add(dip(24, scale))
        .max(dip(60, scale));
    let open = Rect::new(pad, btn_y, open_w, btn_h);

    let mut x = win_w as i32 - pad;
    x -= caret_w as i32;
    let export_caret = Rect::new(x, btn_y, caret_w, btn_h);
    x -= export_w as i32;
    let export = Rect::new(x, btn_y, export_w, btn_h);
    x -= gap + copy_w as i32;
    let copy = Rect::new(x, btn_y, copy_w, btn_h);
    x -= dip(16, scale) as i32 + icon as i32;
    let zoom_in = Rect::new(x, btn_y, icon, icon);
    x -= zoom_w as i32;
    let zoom_label = Rect::new(x, btn_y, zoom_w, icon);
    x -= icon as i32;
    let zoom_out = Rect::new(x, btn_y, icon, icon);

    ToolbarLayout {
        open,
        zoom_out,
        zoom_label,
        zoom_in,
        copy,
        export,
        export_caret,
    }
}

fn copy_short(copy_label: &str) -> &'static str {
    if copy_label.contains("text") {
        "Text"
    } else {
        "MD"
    }
}

fn layout_for(app: &AppState, win_w: u32, scale: f32) -> ToolbarLayout {
    toolbar_layout_at(
        win_w,
        app.copy_format().hud_label(),
        app.export_format().action_label(),
        &zoom_label(app),
        scale,
    )
}

pub fn export_label_x(win_w: u32) -> u32 {
    toolbar_layout(win_w, "Copy MD", EXPORT_ACTION_LABEL, "100%")
        .export
        .x
        .max(0) as u32
}

fn toolbar_hit_y(win_h: u32, y: f64, scale: f32) -> bool {
    y >= 0.0 && y < f64::from(dip(TOOLBAR_HEIGHT, scale).min(win_h))
}

fn export_menu_size(scale: f32) -> (u32, u32) {
    let body = BODY_PX * scale;
    let mut w = dip(200, scale);
    for format in ExportFormat::ALL {
        w = w.max(text_width_px(format.action_label(), body).saturating_add(dip(48, scale)));
    }
    let h = dip(MENU_PAD, scale) * 2 + dip(MENU_ITEM_H, scale) * ExportFormat::ALL.len() as u32;
    (w, h)
}

fn export_menu_rect(layout: &ToolbarLayout, scale: f32) -> Rect {
    let (mw, mh) = export_menu_size(scale);
    let caret = layout.export_caret;
    let x = (caret.x + caret.w as i32 - mw as i32).max(dip(PAD_X, scale) as i32);
    let y = dip(TOOLBAR_HEIGHT, scale) as i32 + dip(MENU_GAP, scale) as i32;
    Rect::new(x, y, mw, mh)
}

fn export_menu_item_rect(menu: Rect, index: usize, scale: f32) -> Rect {
    let pad = dip(MENU_PAD, scale);
    let ih = dip(MENU_ITEM_H, scale);
    Rect::new(
        menu.x + pad as i32,
        menu.y + pad as i32 + (index as u32 * ih) as i32,
        menu.w.saturating_sub(pad * 2),
        ih,
    )
}

pub fn chrome_hit_at(
    app: &AppState,
    win_w: u32,
    win_h: u32,
    x: f64,
    y: f64,
    scale: f32,
    export_menu_open: bool,
) -> Option<ChromeHit> {
    let layout = layout_for(app, win_w, scale);
    if toolbar_hit_y(win_h, y, scale) {
        if layout.open.contains(x, y) {
            return Some(ChromeHit::Open);
        }
        if layout.export.contains(x, y) {
            return Some(ChromeHit::Export);
        }
        if layout.export_caret.contains(x, y) {
            return Some(ChromeHit::ExportMenu);
        }
        if layout.copy.contains(x, y) {
            return Some(ChromeHit::Copy);
        }
        if layout.zoom_in.contains(x, y) {
            return Some(ChromeHit::ZoomIn);
        }
        if layout.zoom_out.contains(x, y) {
            return Some(ChromeHit::ZoomOut);
        }
        return None;
    }
    if export_menu_open {
        let menu = export_menu_rect(&layout, scale);
        if menu.contains(x, y) {
            for i in 0..ExportFormat::ALL.len() {
                if export_menu_item_rect(menu, i, scale).contains(x, y) {
                    return Some(ChromeHit::ExportItem(i));
                }
            }
            return Some(ChromeHit::ExportMenu);
        }
    }
    None
}

pub fn open_hit(win_w: u32, win_h: u32, x: f64, y: f64) -> bool {
    if !toolbar_hit_y(win_h, y, 1.0) {
        return false;
    }
    toolbar_layout(win_w, "Copy MD", EXPORT_ACTION_LABEL, "100%")
        .open
        .contains(x, y)
}

pub fn export_hit(win_w: u32, win_h: u32, x: f64, y: f64) -> bool {
    if !toolbar_hit_y(win_h, y, 1.0) {
        return false;
    }
    toolbar_layout(win_w, "Copy MD", EXPORT_ACTION_LABEL, "100%")
        .export
        .contains(x, y)
}

pub fn export_menu_hit(win_w: u32, win_h: u32, x: f64, y: f64) -> bool {
    if !toolbar_hit_y(win_h, y, 1.0) {
        return false;
    }
    toolbar_layout(win_w, "Copy MD", EXPORT_ACTION_LABEL, "100%")
        .export_caret
        .contains(x, y)
}

pub fn export_menu_item_hit(win_w: u32, index: usize, x: f64, y: f64) -> bool {
    let layout = toolbar_layout(win_w, "Copy MD", EXPORT_ACTION_LABEL, "100%");
    let menu = export_menu_rect(&layout, 1.0);
    export_menu_item_rect(menu, index, 1.0).contains(x, y)
}

pub fn copy_format_hit(
    win_w: u32,
    win_h: u32,
    copy_label: &str,
    export_label: &str,
    x: f64,
    y: f64,
) -> bool {
    if !toolbar_hit_y(win_h, y, 1.0) {
        return false;
    }
    toolbar_layout(win_w, copy_label, export_label, "100%")
        .copy
        .contains(x, y)
}

#[allow(dead_code)]
pub fn zoom_out_hit(
    win_w: u32,
    win_h: u32,
    copy_label: &str,
    export_label: &str,
    zoom: &str,
    x: f64,
    y: f64,
) -> bool {
    if !toolbar_hit_y(win_h, y, 1.0) {
        return false;
    }
    toolbar_layout(win_w, copy_label, export_label, zoom)
        .zoom_out
        .contains(x, y)
}

#[allow(dead_code)]
pub fn zoom_in_hit(
    win_w: u32,
    win_h: u32,
    copy_label: &str,
    export_label: &str,
    zoom: &str,
    x: f64,
    y: f64,
) -> bool {
    if !toolbar_hit_y(win_h, y, 1.0) {
        return false;
    }
    toolbar_layout(win_w, copy_label, export_label, zoom)
        .zoom_in
        .contains(x, y)
}

pub fn in_chrome(
    app: &AppState,
    win_w: u32,
    win_h: u32,
    x: f64,
    y: f64,
    scale: f32,
    export_menu_open: bool,
) -> bool {
    if y < f64::from(chrome_top_at(app, scale)) {
        return true;
    }
    if y >= f64::from(win_h.saturating_sub(dip(STATUS_HEIGHT, scale))) {
        return true;
    }
    if x >= f64::from(win_w.saturating_sub(dip(SCROLLBAR_WIDTH, scale))) {
        return true;
    }
    if export_menu_open {
        let layout = layout_for(app, win_w, scale);
        if export_menu_rect(&layout, scale).contains(x, y) {
            return true;
        }
    }
    false
}

fn icon_bg(hover: bool, pressed: bool) -> Option<u32> {
    if pressed {
        Some(blend_over(TOOLBAR_BG, 0xFFFFFF, 70))
    } else if hover {
        Some(blend_over(TOOLBAR_BG, 0xFFFFFF, 46))
    } else {
        None
    }
}

fn paint_icon_btn(
    buf: &mut [u32],
    width: u32,
    height: u32,
    rect: Rect,
    hover: bool,
    pressed: bool,
    radius: u32,
) {
    if let Some(bg) = icon_bg(hover, pressed) {
        fill_round_rect(buf, width, height, rect, radius, bg);
    }
}

fn icon_minus(buf: &mut [u32], width: u32, height: u32, rect: Rect, color: u32) {
    let s = rect.w.min(rect.h) as f32;
    let cx = rect.x as f32 + rect.w as f32 * 0.5;
    let cy = rect.y as f32 + rect.h as f32 * 0.5;
    let arm = s * 0.28;
    let thick = (s * 0.055).max(1.6);
    stroke_line(buf, width, height, cx - arm, cy, cx + arm, cy, thick, color);
}

fn icon_plus(buf: &mut [u32], width: u32, height: u32, rect: Rect, color: u32) {
    let s = rect.w.min(rect.h) as f32;
    let cx = rect.x as f32 + rect.w as f32 * 0.5;
    let cy = rect.y as f32 + rect.h as f32 * 0.5;
    let arm = s * 0.28;
    let thick = (s * 0.055).max(1.6);
    stroke_line(buf, width, height, cx - arm, cy, cx + arm, cy, thick, color);
    stroke_line(buf, width, height, cx, cy - arm, cx, cy + arm, thick, color);
}

fn icon_copy(buf: &mut [u32], width: u32, height: u32, rect: Rect, color: u32) {
    let s = rect.w.min(rect.h) as f32;
    let thick = (s * 0.045).max(1.4);
    let x = rect.x as f32 + s * 0.22;
    let y = rect.y as f32 + s * 0.22;
    let a = s * 0.36;
    let b = s * 0.36;
    stroke_line(buf, width, height, x + s * 0.12, y, x + a, y, thick, color);
    stroke_line(buf, width, height, x + a, y, x + a, y + b, thick, color);
    stroke_line(
        buf,
        width,
        height,
        x + a,
        y + b,
        x + s * 0.12,
        y + b,
        thick,
        color,
    );
    stroke_line(
        buf,
        width,
        height,
        x + s * 0.12,
        y + b,
        x + s * 0.12,
        y,
        thick,
        color,
    );
    stroke_line(
        buf,
        width,
        height,
        x,
        y + s * 0.14,
        x,
        y + b + s * 0.14,
        thick,
        color,
    );
    stroke_line(
        buf,
        width,
        height,
        x,
        y + b + s * 0.14,
        x + a - s * 0.12,
        y + b + s * 0.14,
        thick,
        color,
    );
}

fn icon_check(buf: &mut [u32], width: u32, height: u32, rect: Rect, color: u32) {
    let s = rect.w.min(rect.h) as f32;
    let thick = (s * 0.12).max(1.6);
    let x0 = rect.x as f32 + s * 0.18;
    let y0 = rect.y as f32 + s * 0.52;
    let xm = rect.x as f32 + s * 0.38;
    let ym = rect.y as f32 + s * 0.72;
    let x1 = rect.x as f32 + s * 0.82;
    let y1 = rect.y as f32 + s * 0.28;
    stroke_line(buf, width, height, x0, y0, xm, ym, thick, color);
    stroke_line(buf, width, height, xm, ym, x1, y1, thick, color);
}

fn icon_caret(buf: &mut [u32], width: u32, height: u32, rect: Rect, color: u32) {
    let s = rect.w.min(rect.h) as f32;
    let cx = rect.x as f32 + rect.w as f32 * 0.5;
    let cy = rect.y as f32 + rect.h as f32 * 0.5;
    let arm = s * 0.18;
    let thick = (s * 0.07).max(1.6);
    stroke_line(
        buf,
        width,
        height,
        cx - arm,
        cy - arm * 0.35,
        cx,
        cy + arm * 0.45,
        thick,
        color,
    );
    stroke_line(
        buf,
        width,
        height,
        cx + arm,
        cy - arm * 0.35,
        cx,
        cy + arm * 0.45,
        thick,
        color,
    );
}

fn is_hit(hover: Option<ChromeHit>, pressed: Option<ChromeHit>, id: ChromeHit) -> (bool, bool) {
    (hover == Some(id), pressed == Some(id))
}

fn export_hot(hit: Option<ChromeHit>) -> bool {
    matches!(hit, Some(ChromeHit::Export) | Some(ChromeHit::ExportMenu))
}

pub fn draw_hud(
    buf: &mut [u32],
    width: u32,
    height: u32,
    app: &AppState,
    chrome: ChromePaint,
    scale: f32,
) {
    let hover = chrome.hover;
    let pressed = chrome.pressed;
    let export_menu_open = chrome.export_menu_open;
    let copy_label = app.copy_format().hud_label();
    let export_label = app.export_format().action_label();
    let format_label = app.export_format().hud_label();
    let zoom = zoom_label(app);
    let layout = toolbar_layout_at(width, copy_label, export_label, &zoom, scale);
    let pad = dip(PAD_X, scale) as i32;
    let tb = dip(TOOLBAR_HEIGHT, scale).min(height);
    let body = BODY_PX * scale;
    let title_px = TITLE_PX * scale;
    let cap = CAPTION_PX * scale;
    let radius = dip(RADIUS, scale);
    let icon = dip(ICON_BTN, scale);

    fill_rect(buf, width, height, Rect::new(0, 0, width, tb), TOOLBAR_BG);

    let (open_h, open_p) = is_hit(hover, pressed, ChromeHit::Open);
    let open_bg = if open_p {
        0x0066D6
    } else if open_h {
        0x1A86FF
    } else {
        PRIMARY
    };
    fill_round_rect(buf, width, height, layout.open, radius, open_bg);
    draw_text_centered(buf, width, height, layout.open, "Open", PRIMARY_FG, body);

    let title_h = em_height(title_px);
    let sub_h = em_height(cap);
    let stack_gap = dip(5, scale);
    let block_h = title_h + stack_gap + sub_h;
    let title_y = ((tb.saturating_sub(block_h)) / 2) as i32;
    let sub_y = title_y + title_h as i32 + stack_gap as i32;
    let title_x = layout.open.x + layout.open.w as i32 + dip(12, scale) as i32;
    let title_max = (layout.zoom_out.x - title_x - dip(12, scale) as i32).max(48) as u32;
    let title = ellipsize_to_width(app.title(), title_max, title_px);
    draw_text_px(
        buf, width, height, title_x, title_y, &title, TITLE, title_px,
    );
    let sub = page_status_label(app);
    draw_text_px(buf, width, height, title_x, sub_y, &sub, SUBTITLE, cap);

    let (h, p) = is_hit(hover, pressed, ChromeHit::ZoomOut);
    paint_icon_btn(buf, width, height, layout.zoom_out, h, p, radius);
    icon_minus(buf, width, height, layout.zoom_out, ICON);
    draw_text_centered(buf, width, height, layout.zoom_label, &zoom, SUBTITLE, body);
    let (h, p) = is_hit(hover, pressed, ChromeHit::ZoomIn);
    paint_icon_btn(buf, width, height, layout.zoom_in, h, p, radius);
    icon_plus(buf, width, height, layout.zoom_in, ICON);

    let (h, p) = is_hit(hover, pressed, ChromeHit::Copy);
    paint_icon_btn(buf, width, height, layout.copy, h, p, radius);
    let copy_icon = Rect::new(layout.copy.x, layout.copy.y, icon, layout.copy.h);
    icon_copy(buf, width, height, copy_icon, ICON);
    let copy_text = Rect::new(
        layout.copy.x + icon as i32,
        layout.copy.y,
        layout.copy.w.saturating_sub(icon),
        layout.copy.h,
    );
    draw_text_centered(
        buf,
        width,
        height,
        copy_text,
        copy_short(copy_label),
        SUBTITLE,
        body,
    );

    let export_bg = if export_hot(pressed) {
        0x0066D6
    } else if export_hot(hover) {
        0x1A86FF
    } else {
        PRIMARY
    };
    let combined = Rect::new(
        layout.export.x,
        layout.export.y,
        layout.export.w + layout.export_caret.w,
        layout.export.h,
    );
    fill_round_rect(buf, width, height, combined, radius, export_bg);
    let split_x = layout.export.x + layout.export.w as i32;
    fill_rect(
        buf,
        width,
        height,
        Rect::new(
            split_x,
            layout.export.y + dip(8, scale) as i32,
            dip(1, scale).max(1),
            layout.export.h.saturating_sub(dip(16, scale)),
        ),
        blend_over(export_bg, 0xFFFFFF, 64),
    );
    draw_text_centered(
        buf,
        width,
        height,
        layout.export,
        export_label,
        PRIMARY_FG,
        body,
    );
    icon_caret(buf, width, height, layout.export_caret, PRIMARY_FG);

    if let Some(msg) = banner_copy(app) {
        let y = tb;
        let h = banner_height_at(app, scale).min(height.saturating_sub(y));
        fill_rect(
            buf,
            width,
            height,
            Rect::new(0, y as i32, width, h),
            hud_color(app),
        );
        let max_w = width.saturating_sub(pad as u32 * 2).max(48);
        let line_gap = dip(4, scale);
        let line_h = em_height(body) + line_gap;
        let lines = if banner_compact(app) {
            vec![ellipsize_to_width(&msg, max_w, body)]
        } else {
            let mut lines = wrap_text(&msg, max_w, body);
            if lines.len() > 3 {
                let rest = lines[2..].join(" ");
                lines.truncate(2);
                lines.push(ellipsize_to_width(&rest, max_w, body));
            }
            lines
        };
        let block_h = (lines.len() as u32 * line_h).saturating_sub(line_gap);
        let mut ty = y as i32 + (h as i32 - block_h as i32).max(0) / 2;
        for line in &lines {
            draw_text_px(buf, width, height, pad, ty, line, PRIMARY_FG, body);
            ty += line_h as i32;
        }
    }

    let status_h = dip(STATUS_HEIGHT, scale);
    if height <= status_h {
        return;
    }
    let sy = height.saturating_sub(status_h) as i32;
    fill_rect(
        buf,
        width,
        height,
        Rect::new(0, sy, width, status_h),
        CANVAS,
    );
    hline(buf, width, height, sy, 0, width, DIVIDER);
    let status_rect = Rect::new(pad, sy, width.saturating_sub(pad as u32 * 2), status_h);
    draw_text_px(
        buf,
        width,
        height,
        pad,
        sy + (status_h as i32 - em_height(cap) as i32).max(0) / 2,
        &page_status_label(app),
        STATUS_FG,
        cap,
    );
    let right = format!("{}  ·  {}", zoom_label(app), format_label);
    let rx = width as i32 - pad - text_width_px(&right, cap) as i32;
    draw_text_px(
        buf,
        width,
        height,
        rx.max(pad),
        status_rect.y + (status_h as i32 - em_height(cap) as i32).max(0) / 2,
        &right,
        STATUS_FG,
        cap,
    );

    if export_menu_open {
        draw_export_menu(buf, width, height, app, hover, scale, &layout);
    }
}

fn draw_export_menu(
    buf: &mut [u32],
    width: u32,
    height: u32,
    app: &AppState,
    hover: Option<ChromeHit>,
    scale: f32,
    layout: &ToolbarLayout,
) {
    let menu = export_menu_rect(layout, scale);
    let shadow = Rect::new(
        menu.x + dip(2, scale) as i32,
        menu.y + dip(2, scale) as i32,
        menu.w,
        menu.h,
    );
    let radius = dip(RADIUS, scale);
    fill_round_rect(buf, width, height, shadow, radius, MENU_SHADOW);
    fill_round_rect(buf, width, height, menu, radius, MENU_BG);
    let body = BODY_PX * scale;
    let item_pad = dip(10, scale) as i32;
    let current = app.export_format();
    for (i, format) in ExportFormat::ALL.iter().enumerate() {
        let item = export_menu_item_rect(menu, i, scale);
        if hover == Some(ChromeHit::ExportItem(i)) {
            fill_round_rect(buf, width, height, item, dip(6, scale), MENU_HOVER);
        }
        let th = em_height(body) as i32;
        let ty = item.y + (item.h as i32 - th).max(0) / 2;
        draw_text_px(
            buf,
            width,
            height,
            item.x + item_pad,
            ty,
            format.action_label(),
            MENU_FG,
            body,
        );
        if *format == current {
            let icon = dip(14, scale);
            let ix = item.x + item.w as i32 - item_pad - icon as i32;
            let iy = item.y + (item.h as i32 - icon as i32) / 2;
            icon_check(buf, width, height, Rect::new(ix, iy, icon, icon), PRIMARY);
        }
    }
}

pub fn draw_scrollbar(
    buf: &mut [u32],
    width: u32,
    height: u32,
    app: &AppState,
    scroll_y: f64,
    content_h: f64,
    viewport_h: f64,
    scale: f32,
) {
    let top = chrome_top_at(app, scale);
    let status = dip(STATUS_HEIGHT, scale);
    let track_h = height.saturating_sub(top + status);
    let sb = dip(SCROLLBAR_WIDTH, scale);
    if track_h == 0 || width < sb {
        return;
    }
    let track = Rect::new(width.saturating_sub(sb) as i32, top as i32, sb, track_h);
    let thumb_w = dip(4, scale).max(3);
    let thumb_x = track.x + ((sb.saturating_sub(thumb_w)) / 2) as i32;
    if content_h <= viewport_h || viewport_h <= 0.0 {
        fill_round_rect(
            buf,
            width,
            height,
            Rect::new(
                thumb_x,
                track.y + dip(8, scale) as i32,
                thumb_w,
                track_h.saturating_sub(dip(16, scale)),
            ),
            dip(2, scale),
            0xD8D8DC,
        );
        return;
    }
    let ratio = (viewport_h / content_h).clamp(0.05, 1.0);
    let thumb_h = ((track_h as f64 * ratio).round() as u32)
        .max(dip(32, scale))
        .min(track_h);
    let max_scroll = (content_h - viewport_h).max(1.0);
    let t = (scroll_y / max_scroll).clamp(0.0, 1.0);
    let travel = track_h.saturating_sub(thumb_h);
    let thumb_y = track.y + (t * f64::from(travel)).round() as i32;
    fill_round_rect(
        buf,
        width,
        height,
        Rect::new(thumb_x, thumb_y, thumb_w, thumb_h),
        dip(2, scale),
        0xC0C0C4,
    );
}
