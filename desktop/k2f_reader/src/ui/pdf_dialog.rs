use super::draw::{blend_over, fill_rect, fill_round_rect, Rect};
use super::font::{draw_text_px, em_height, text_width_px, BODY_PX, TITLE_PX};
use super::hud::dip;
use k2f_pdf::{DEFAULT_EXPORT_SCALE, PdfScale};

pub const DEFAULT_PDF_DIALOG_SCALE: f32 = DEFAULT_EXPORT_SCALE;

const OVERLAY: u32 = 0x000000;
const OVERLAY_ALPHA: u8 = 96;
const PANEL_BG: u32 = 0xFFFFFF;
const PANEL_FG: u32 = 0x1A1A1A;
const PANEL_HINT: u32 = 0x6E6E73;
const BORDER: u32 = 0xE8E8ED;
const HOVER: u32 = 0xF2F2F7;
const PRIMARY: u32 = 0x007AFF;
const PRIMARY_FG: u32 = 0xFFFFFF;
const SECONDARY_BG: u32 = 0xF2F2F7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfDialogHit {
    Scale2,
    Scale3,
    Scale4,
    Cancel,
    Confirm,
}

#[derive(Debug, Clone, Copy)]
pub struct PdfDialogState {
    pub open: bool,
    pub selected_scale: f32,
    pub hover: Option<PdfDialogHit>,
    pub pressed: Option<PdfDialogHit>,
}

impl PdfDialogState {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_scale: DEFAULT_PDF_DIALOG_SCALE,
            hover: None,
            pressed: None,
        }
    }

    pub fn open_dialog(&mut self) {
        self.open = true;
        self.hover = None;
        self.pressed = None;
    }

    pub fn close_dialog(&mut self) -> bool {
        let was = self.open;
        self.open = false;
        self.hover = None;
        self.pressed = None;
        was
    }

    pub fn selected_pdf_scale(&self) -> PdfScale {
        PdfScale::from_f32(self.selected_scale).unwrap_or(PdfScale::DEFAULT)
    }
}

struct Layout {
    panel: Rect,
    scale2: Rect,
    scale3: Rect,
    scale4: Rect,
    cancel: Rect,
    confirm: Rect,
}

fn layout(win_w: u32, win_h: u32, scale: f32) -> Layout {
    let panel_w = dip(360, scale).min(win_w.saturating_sub(dip(48, scale)));
    let row_h = dip(40, scale);
    let gap = dip(8, scale);
    let pad = dip(20, scale);
    let title_h = dip(28, scale);
    let hint_h = dip(40, scale);
    let actions_h = dip(36, scale);
    let panel_h = pad * 2 + title_h + gap + hint_h + gap + row_h * 3 + gap * 2 + actions_h;
    let px = ((win_w - panel_w) / 2) as i32;
    let py = ((win_h - panel_h) / 2) as i32;
    let panel = Rect::new(px, py, panel_w, panel_h);
    let inner_x = panel.x + pad as i32;
    let inner_w = panel.w.saturating_sub(pad * 2);
    let y0 = panel.y + (pad + title_h + gap + hint_h + gap) as i32;
    let row_step = (row_h + gap) as i32;
    let scale2 = Rect::new(inner_x, y0, inner_w, row_h);
    let scale3 = Rect::new(inner_x, y0 + row_step, inner_w, row_h);
    let scale4 = Rect::new(inner_x, y0 + row_step * 2, inner_w, row_h);
    let btn_w = dip(84, scale);
    let btn_gap = dip(8, scale);
    let actions_y = panel.y + panel.h as i32 - pad as i32 - actions_h as i32;
    let confirm = Rect::new(
        panel.x + panel.w as i32 - pad as i32 - btn_w as i32,
        actions_y,
        btn_w,
        actions_h,
    );
    let cancel = Rect::new(
        confirm.x - btn_gap as i32 - btn_w as i32,
        actions_y,
        btn_w,
        actions_h,
    );
    Layout {
        panel,
        scale2,
        scale3,
        scale4,
        cancel,
        confirm,
    }
}

pub fn hit_at(
    state: &PdfDialogState,
    win_w: u32,
    win_h: u32,
    x: f64,
    y: f64,
    scale: f32,
) -> Option<PdfDialogHit> {
    if !state.open {
        return None;
    }
    let l = layout(win_w, win_h, scale);
    if l.confirm.contains(x, y) {
        return Some(PdfDialogHit::Confirm);
    }
    if l.cancel.contains(x, y) {
        return Some(PdfDialogHit::Cancel);
    }
    if l.scale2.contains(x, y) {
        return Some(PdfDialogHit::Scale2);
    }
    if l.scale3.contains(x, y) {
        return Some(PdfDialogHit::Scale3);
    }
    if l.scale4.contains(x, y) {
        return Some(PdfDialogHit::Scale4);
    }
    Some(PdfDialogHit::Cancel)
}

pub fn draw(buf: &mut [u32], win_w: u32, win_h: u32, state: &PdfDialogState, scale: f32) {
    if !state.open {
        return;
    }
    for px in buf.iter_mut() {
        *px = blend_over(*px, OVERLAY, OVERLAY_ALPHA);
    }
    let l = layout(win_w, win_h, scale);
    let radius = dip(12, scale);
    fill_round_rect(buf, win_w, win_h, l.panel, radius, PANEL_BG);
    let pad = dip(20, scale) as i32;
    let title = TITLE_PX * scale;
    let body = BODY_PX * scale;
    let title_y = l.panel.y + pad;
    draw_text_px(
        buf,
        win_w,
        win_h,
        l.panel.x + pad,
        title_y,
        "Export PDF",
        PANEL_FG,
        title,
    );
    let hint_y = title_y + em_height(title) as i32 + dip(8, scale) as i32;
    for (i, line) in [
        "Choose quality for pages with blur,",
        "shadow, or gradient.",
    ]
    .iter()
    .enumerate()
    {
        draw_text_px(
            buf,
            win_w,
            win_h,
            l.panel.x + pad,
            hint_y + (i as i32 * em_height(body) as i32),
            line,
            PANEL_HINT,
            body,
        );
    }
    draw_scale_row(
        buf,
        win_w,
        win_h,
        l.scale2,
        scale,
        PdfDialogHit::Scale2,
        2.0,
        "2x - Standard",
        state,
    );
    draw_scale_row(
        buf,
        win_w,
        win_h,
        l.scale3,
        scale,
        PdfDialogHit::Scale3,
        3.0,
        "3x - High (4K)",
        state,
    );
    draw_scale_row(
        buf,
        win_w,
        win_h,
        l.scale4,
        scale,
        PdfDialogHit::Scale4,
        4.0,
        "4x - Maximum",
        state,
    );
    draw_button(
        buf,
        win_w,
        win_h,
        l.cancel,
        scale,
        "Cancel",
        SECONDARY_BG,
        PANEL_FG,
        state.hover == Some(PdfDialogHit::Cancel),
    );
    draw_button(
        buf,
        win_w,
        win_h,
        l.confirm,
        scale,
        "Export",
        PRIMARY,
        PRIMARY_FG,
        state.hover == Some(PdfDialogHit::Confirm),
    );
}

fn draw_scale_row(
    buf: &mut [u32],
    win_w: u32,
    win_h: u32,
    row: Rect,
    ui_scale: f32,
    hit: PdfDialogHit,
    value: f32,
    label: &str,
    state: &PdfDialogState,
) {
    let selected = (state.selected_scale - value).abs() < f32::EPSILON;
    let hover = state.hover == Some(hit);
    let bg = if hover { HOVER } else { PANEL_BG };
    fill_round_rect(buf, win_w, win_h, row, dip(8, ui_scale), bg);
    let radio = dip(14, ui_scale);
    let rx = row.x + dip(12, ui_scale) as i32;
    let ry = row.y + (row.h as i32 - radio as i32) / 2;
    let r = Rect::new(rx, ry, radio, radio);
    fill_round_rect(buf, win_w, win_h, r, radio / 2, if selected { PRIMARY } else { BORDER });
    let inner = if selected { dip(8, ui_scale) } else { dip(10, ui_scale) };
    let ix = rx + (radio as i32 - inner as i32) / 2;
    let iy = ry + (radio as i32 - inner as i32) / 2;
    fill_round_rect(
        buf,
        win_w,
        win_h,
        Rect::new(ix, iy, inner, inner),
        inner / 2,
        if selected { PRIMARY } else { PANEL_BG },
    );
    let body = BODY_PX * ui_scale;
    let ty = row.y + (row.h as i32 - em_height(body) as i32) / 2;
    draw_text_px(
        buf,
        win_w,
        win_h,
        rx + radio as i32 + dip(10, ui_scale) as i32,
        ty,
        label,
        PANEL_FG,
        body,
    );
}

fn draw_button(
    buf: &mut [u32],
    win_w: u32,
    win_h: u32,
    rect: Rect,
    ui_scale: f32,
    label: &str,
    bg: u32,
    fg: u32,
    hover: bool,
) {
    let color = if hover { blend(bg, 0x111111, 0.08) } else { bg };
    fill_round_rect(buf, win_w, win_h, rect, dip(8, ui_scale), color);
    let body = BODY_PX * ui_scale;
    let tw = text_width_px(label, body);
    let tx = rect.x + (rect.w as i32 - tw as i32) / 2;
    let ty = rect.y + (rect.h as i32 - em_height(body) as i32) / 2;
    draw_text_px(buf, win_w, win_h, tx, ty, label, fg, body);
}

fn blend(base: u32, overlay: u32, alpha: f32) -> u32 {
    let br = ((base >> 16) & 0xFF) as f32;
    let bg = ((base >> 8) & 0xFF) as f32;
    let bb = (base & 0xFF) as f32;
    let or = ((overlay >> 16) & 0xFF) as f32;
    let og = ((overlay >> 8) & 0xFF) as f32;
    let ob = (overlay & 0xFF) as f32;
    let r = (br * (1.0 - alpha) + or * alpha).round() as u32;
    let g = (bg * (1.0 - alpha) + og * alpha).round() as u32;
    let b = (bb * (1.0 - alpha) + ob * alpha).round() as u32;
    (r << 16) | (g << 8) | b
}
