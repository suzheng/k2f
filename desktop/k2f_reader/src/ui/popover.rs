//! Painted node popover, anchored under the first lock box (web viewer).

use super::coords::PageView;
use super::draw::{fill_rect, fill_round_rect, stroke_line, Rect};
use super::edit::{Popover, PopoverField, SHOW_META_FIELDS};
use super::font::{
    draw_text_centered, draw_text_px, ellipsize_to_width, em_height, text_width_px, BODY_PX,
    CAPTION_PX,
};
use super::form_fill::millipt_to_pt;
use super::hud::{dip, STATUS_HEIGHT, TOOLBAR_HEIGHT};
use k2f_paint::LocatedBox;

const PANEL_W: u32 = 280;
const PAD: u32 = 10;
const GAP: u32 = 6;
const FIELD_H: u32 = 28;
/// Design-pixel height of the node text box. Dragging its bottom edge grows this.
pub const TEXT_H: u32 = 56;
const TEXT_H_MAX: u32 = 480;
const LABEL_W: u32 = 58;
const RADIUS: u32 = 8;
const PANEL: u32 = 0xFFFFFF;
const INK: u32 = 0x1A1A1A;
const MUTED: u32 = 0x8E8E93;
const LINE: u32 = 0xE8E8ED;
const FIELD_BG: u32 = 0xF5F5F7;
const FOCUS: u32 = 0x007AFF;
const PRIMARY: u32 = 0x007AFF;
const PRIMARY_FG: u32 = 0xFFFFFF;
const FIELD_PAD: i32 = 6;

pub struct PopoverLayout {
    pub panel: Rect,
    pub text: Option<Rect>,
    pub role: Option<Rect>,
    pub variant: Option<Rect>,
    pub cancel: Rect,
    pub save: Rect,
    pub copy: Option<Rect>,
    /// Bottom edge of the text box, including the gap under it. Drag to grow.
    pub resize: Option<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverHit {
    Text,
    Role,
    Variant,
    Cancel,
    Save,
    Copy,
    /// Drag the text box taller.
    Resize,
    Panel,
}

pub fn anchor_rect(view: &PageView, b: &LocatedBox) -> Rect {
    let (x0, y0) = view.pt_to_window(millipt_to_pt(b.x), millipt_to_pt(b.y));
    let (x1, y1) = view.pt_to_window(
        millipt_to_pt(b.x.saturating_add(b.width)),
        millipt_to_pt(b.y.saturating_add(b.height)),
    );
    Rect::new(
        x0.round() as i32,
        y0.round() as i32,
        (x1 - x0).round().max(1.0) as u32,
        (y1 - y0).round().max(1.0) as u32,
    )
}

pub fn layout_at(
    anchor: Rect,
    text_on: bool,
    scale: f32,
    win_w: u32,
    win_h: u32,
    text_px: u32,
) -> PopoverLayout {
    let pad = dip(PAD, scale);
    let gap = dip(GAP, scale);
    let field_h = dip(FIELD_H, scale);
    let min_text = if text_on { dip(TEXT_H, scale) } else { 0 };
    let panel_w = dip(PANEL_W, scale).min(win_w.saturating_sub(pad * 2).max(120));
    let inner_w = panel_w.saturating_sub(pad * 2);
    let meta_block = if SHOW_META_FIELDS {
        (field_h + gap) * 2
    } else {
        0
    };
    let copy_block = if SHOW_META_FIELDS { gap + field_h } else { 0 };
    // Everything except the text box itself. The gap under the box stays.
    let fixed = pad + if text_on { gap } else { 0 } + meta_block + field_h + copy_block + pad;
    let top_limit = dip(TOOLBAR_HEIGHT, scale) as i32;
    let bottom_limit = win_h.saturating_sub(dip(STATUS_HEIGHT, scale)) as i32;
    let min_h = fixed + min_text;
    let below = anchor.y + anchor.h as i32 + gap as i32;
    let above_y = anchor.y - gap as i32 - min_h as i32;
    let place_below = below + min_h as i32 <= bottom_limit || above_y < top_limit;
    let side_room = if place_below {
        (bottom_limit - below).max(0) as u32
    } else {
        (anchor.y - gap as i32 - top_limit).max(0) as u32
    };
    let text_h = if text_on {
        let wanted = dip(text_px.max(TEXT_H), scale).max(min_text);
        let room = side_room.saturating_sub(fixed).max(min_text);
        wanted.min(room)
    } else {
        0
    };
    let h = fixed + text_h;
    let mut x = anchor.x;
    let mut y = if place_below {
        below
    } else {
        anchor.y - gap as i32 - h as i32
    };
    if y < top_limit {
        y = top_limit;
    }
    if x + panel_w as i32 > win_w as i32 - pad as i32 {
        x = win_w as i32 - pad as i32 - panel_w as i32;
    }
    if x < pad as i32 {
        x = pad as i32;
    }
    let panel = Rect::new(x, y, panel_w, h);
    let mut row_y = y + pad as i32;
    let text = if text_on {
        let r = Rect::new(x + pad as i32, row_y, inner_w, text_h);
        row_y += text_h as i32 + gap as i32;
        Some(r)
    } else {
        None
    };
    let (role, variant) = if SHOW_META_FIELDS {
        let label_w = dip(LABEL_W, scale);
        let input_w = inner_w.saturating_sub(label_w + gap);
        let role = Rect::new(
            x + pad as i32 + label_w as i32 + gap as i32,
            row_y,
            input_w,
            field_h,
        );
        row_y += field_h as i32 + gap as i32;
        let variant = Rect::new(role.x, row_y, input_w, field_h);
        row_y += field_h as i32 + gap as i32;
        (Some(role), Some(variant))
    } else {
        (None, None)
    };
    let btn_w = inner_w.saturating_sub(gap) / 2;
    let cancel = Rect::new(x + pad as i32, row_y, btn_w, field_h);
    let save = Rect::new(cancel.x + btn_w as i32 + gap as i32, row_y, btn_w, field_h);
    let copy = if SHOW_META_FIELDS {
        row_y += field_h as i32 + gap as i32;
        Some(Rect::new(x + pad as i32, row_y, inner_w, field_h))
    } else {
        None
    };
    let resize = text.map(|r| text_resize_rect(r, scale));
    PopoverLayout {
        panel,
        text,
        role,
        variant,
        cancel,
        save,
        copy,
        resize,
    }
}

/// Bottom rim of the text box plus the gap beneath it. Stops at the buttons.
fn text_resize_rect(text: Rect, scale: f32) -> Rect {
    let into = dip(4, scale).max(3);
    let gap = dip(GAP, scale);
    Rect::new(
        text.x,
        text.y + text.h as i32 - into as i32,
        text.w,
        into + gap,
    )
}

pub fn hit(layout: &PopoverLayout, x: f64, y: f64) -> Option<PopoverHit> {
    if layout.resize.is_some_and(|r| r.contains(x, y)) {
        return Some(PopoverHit::Resize);
    }
    if layout.text.is_some_and(|r| r.contains(x, y)) {
        return Some(PopoverHit::Text);
    }
    if layout.role.is_some_and(|r| r.contains(x, y)) {
        return Some(PopoverHit::Role);
    }
    if layout.variant.is_some_and(|r| r.contains(x, y)) {
        return Some(PopoverHit::Variant);
    }
    if layout.cancel.contains(x, y) {
        return Some(PopoverHit::Cancel);
    }
    if layout.save.contains(x, y) {
        return Some(PopoverHit::Save);
    }
    if layout.copy.is_some_and(|r| r.contains(x, y)) {
        return Some(PopoverHit::Copy);
    }
    if layout.panel.contains(x, y) {
        return Some(PopoverHit::Panel);
    }
    None
}

pub fn draw(
    buf: &mut [u32],
    width: u32,
    height: u32,
    pop: &Popover,
    layout: &PopoverLayout,
    pressed: Option<PopoverHit>,
    scale: f32,
    scroll_px: f64,
) {
    let body = BODY_PX * scale;
    let cap = CAPTION_PX * scale;
    let radius = dip(RADIUS, scale);
    fill_round_rect(buf, width, height, layout.panel, radius, PANEL);
    stroke_rect(buf, width, height, layout.panel, LINE);
    if let Some(rect) = layout.text {
        let focused = pop.focus() == PopoverField::Text;
        paint_field(
            buf,
            width,
            height,
            rect,
            &pop.shown(PopoverField::Text),
            focused,
            focused.then_some(pop.caret()),
            true,
            body,
            scale,
            scroll_px,
        );
        draw_resize_grip(
            buf,
            width,
            height,
            rect,
            scale,
            if focused { FOCUS } else { MUTED },
        );
    }
    if let Some(role) = layout.role {
        paint_labeled(
            buf,
            width,
            height,
            layout,
            "Role",
            role,
            &pop.shown(PopoverField::Role),
            pop.focus() == PopoverField::Role,
            focused_caret(pop, PopoverField::Role),
            cap,
            body,
            scale,
        );
    }
    if let Some(variant) = layout.variant {
        paint_labeled(
            buf,
            width,
            height,
            layout,
            "Variant",
            variant,
            &pop.shown(PopoverField::Variant),
            pop.focus() == PopoverField::Variant,
            focused_caret(pop, PopoverField::Variant),
            cap,
            body,
            scale,
        );
    }
    let cancel_bg = if pressed == Some(PopoverHit::Cancel) {
        0xE4E4E8
    } else {
        FIELD_BG
    };
    fill_round_rect(buf, width, height, layout.cancel, radius, cancel_bg);
    stroke_rect(buf, width, height, layout.cancel, LINE);
    let cancel_label = ellipsize_to_width("Cancel", layout.cancel.w.saturating_sub(8), cap);
    draw_text_centered(buf, width, height, layout.cancel, &cancel_label, INK, cap);
    let save_bg = if pressed == Some(PopoverHit::Save) {
        0x0066D6
    } else {
        PRIMARY
    };
    fill_round_rect(buf, width, height, layout.save, radius, save_bg);
    let save_label = ellipsize_to_width("Save and relock", layout.save.w.saturating_sub(8), cap);
    draw_text_centered(
        buf,
        width,
        height,
        layout.save,
        &save_label,
        PRIMARY_FG,
        cap,
    );
    if let Some(copy) = layout.copy {
        fill_round_rect(buf, width, height, copy, radius, FIELD_BG);
        stroke_rect(buf, width, height, copy, LINE);
        let copy_label = ellipsize_to_width("Copy node", copy.w.saturating_sub(8), cap);
        draw_text_centered(buf, width, height, copy, &copy_label, INK, cap);
    }
}

fn focused_caret(pop: &Popover, field: PopoverField) -> Option<usize> {
    (pop.focus() == field).then_some(pop.caret())
}

fn paint_labeled(
    buf: &mut [u32],
    width: u32,
    height: u32,
    layout: &PopoverLayout,
    label: &str,
    field: Rect,
    value: &str,
    focused: bool,
    caret: Option<usize>,
    cap: f32,
    body: f32,
    scale: f32,
) {
    let label_x = layout.panel.x + dip(PAD, scale) as i32;
    let y = field.y + (field.h as i32 - em_height(cap) as i32).max(0) / 2;
    draw_text_px(buf, width, height, label_x, y, label, MUTED, cap);
    paint_field(
        buf, width, height, field, value, focused, caret, false, body, scale, 0.0,
    );
}

fn paint_field(
    buf: &mut [u32],
    width: u32,
    height: u32,
    rect: Rect,
    value: &str,
    focused: bool,
    caret: Option<usize>,
    multiline: bool,
    px: f32,
    scale: f32,
    scroll_px: f64,
) {
    fill_round_rect(buf, width, height, rect, 4, FIELD_BG);
    stroke_rect(buf, width, height, rect, if focused { FOCUS } else { LINE });
    if multiline || caret.is_some() {
        let view = field_view(
            value,
            rect,
            caret.unwrap_or(0),
            px,
            multiline,
            scale,
            scroll_px,
        );
        draw_field_text(buf, width, height, &view, caret);
        if multiline {
            draw_text_scrollbar(buf, width, height, rect, &view);
        }
        return;
    }
    let max_w = rect.w.saturating_sub((FIELD_PAD as u32) * 2);
    let shown = ellipsize_to_width(value, max_w, px);
    let y = rect.y + (rect.h as i32 - em_height(px) as i32).max(0) / 2;
    draw_text_px(buf, width, height, rect.x + FIELD_PAD, y, &shown, INK, px);
}

pub fn field_rect(layout: &PopoverLayout, field: PopoverField) -> Option<Rect> {
    match field {
        PopoverField::Text => layout.text,
        PopoverField::Role => layout.role,
        PopoverField::Variant => layout.variant,
    }
}

/// Char index in `value` under a click. `scroll_px` is the text box's vertical scroll.
pub fn input_click_index(
    layout: &PopoverLayout,
    field: PopoverField,
    value: &str,
    caret: usize,
    x: f64,
    y: f64,
    scale: f32,
    scroll_px: f64,
) -> Option<usize> {
    let rect = field_rect(layout, field)?;
    let px = BODY_PX * scale;
    Some(
        field_view(
            value,
            rect,
            caret,
            px,
            field == PopoverField::Text,
            scale,
            scroll_px,
        )
        .index_at(x, y),
    )
}

/// Window-pixel caret box for IME. macOS only delivers text input while this is set.
pub fn input_caret_area(
    layout: &PopoverLayout,
    field: PopoverField,
    value: &str,
    caret: usize,
    scale: f32,
    scroll_px: f64,
) -> Option<(f64, f64, f64, f64)> {
    let rect = field_rect(layout, field)?;
    let px = BODY_PX * scale;
    let view = field_view(
        value,
        rect,
        caret,
        px,
        field == PopoverField::Text,
        scale,
        scroll_px,
    );
    let (x, y) = view
        .caret_pos(caret)
        .unwrap_or((rect.x + FIELD_PAD, rect.y + FIELD_PAD));
    let h = em_height(px).max(1) as f64;
    Some((x as f64, y as f64, 2.0, h))
}

#[derive(Debug)]
struct LineSpan {
    start: usize,
    text: String,
}

struct FieldView {
    multiline: bool,
    lines: Vec<LineSpan>,
    first_line: usize,
    /// Lines that fit in the box, including ones past the end of the text.
    visible_cap: usize,
    visible_lines: usize,
    first_char: usize,
    origin_x: i32,
    origin_y: i32,
    line_h: i32,
    px: f32,
    max_w: u32,
}

impl FieldView {
    fn index_at(&self, x: f64, y: f64) -> usize {
        let local_x = x - f64::from(self.origin_x);
        if self.multiline {
            let n = self
                .visible_lines
                .min(self.lines.len().saturating_sub(self.first_line));
            if n == 0 {
                return 0;
            }
            let rel_y = y - f64::from(self.origin_y);
            let mut vis = if rel_y < 0.0 {
                0
            } else {
                (rel_y / f64::from(self.line_h.max(1))).floor() as usize
            };
            if vis >= n {
                vis = n - 1;
            }
            return index_on_line(&self.lines[self.first_line + vis], local_x, self.px);
        }
        let chars: Vec<char> = self.lines[0].text.chars().skip(self.first_char).collect();
        index_on_line(
            &LineSpan {
                start: self.first_char,
                text: chars.into_iter().collect(),
            },
            local_x,
            self.px,
        )
    }

    fn caret_pos(&self, caret: usize) -> Option<(i32, i32)> {
        if self.multiline {
            let line_i = self
                .lines
                .iter()
                .rposition(|l| l.start <= caret)
                .unwrap_or(0);
            if line_i < self.first_line || line_i >= self.first_line + self.visible_lines.max(1) {
                return None;
            }
            let line = &self.lines[line_i];
            let local = caret.saturating_sub(line.start);
            let prefix: String = line.text.chars().take(local).collect();
            let vis = line_i - self.first_line;
            let y = self.origin_y + vis as i32 * self.line_h;
            let x = self.origin_x + text_width_px(&prefix, self.px) as i32;
            return Some((x, y));
        }
        let chars: Vec<char> = self.lines[0].text.chars().collect();
        let caret = caret.min(chars.len());
        let start = self.first_char.min(caret);
        let prefix: String = chars[start..caret].iter().collect();
        Some((
            self.origin_x + text_width_px(&prefix, self.px) as i32,
            self.origin_y,
        ))
    }
}

fn field_view(
    value: &str,
    rect: Rect,
    caret: usize,
    px: f32,
    multiline: bool,
    scale: f32,
    scroll_px: f64,
) -> FieldView {
    let max_w = rect.w.saturating_sub((FIELD_PAD as u32) * 2);
    let origin_x = rect.x + FIELD_PAD;
    let caret = caret.min(value.chars().count());
    if multiline {
        let lines = line_spans(value, max_w, px);
        let visible = max_visible_lines(rect, px, scale);
        let line_h = line_box(px, scale).max(1);
        let max_first = lines.len().saturating_sub(visible);
        let max_px = max_first as f64 * f64::from(line_h);
        let scroll_px = if scroll_px.is_finite() {
            scroll_px.clamp(0.0, max_px)
        } else {
            0.0
        };
        let first = ((scroll_px / f64::from(line_h)).floor() as usize).min(max_first);
        let count = visible.min(lines.len().saturating_sub(first));
        return FieldView {
            multiline: true,
            lines,
            first_line: first,
            visible_cap: visible,
            visible_lines: count,
            first_char: 0,
            origin_x,
            origin_y: rect.y + FIELD_PAD,
            line_h,
            px,
            max_w,
        };
    }
    FieldView {
        multiline: false,
        lines: vec![LineSpan {
            start: 0,
            text: value.to_string(),
        }],
        first_line: 0,
        visible_cap: 1,
        visible_lines: 1,
        first_char: first_visible_char(value, caret, max_w, px),
        origin_x,
        origin_y: rect.y + (rect.h as i32 - em_height(px) as i32).max(0) / 2,
        line_h: line_box(px, scale),
        px,
        max_w,
    }
}

fn draw_field_text(
    buf: &mut [u32],
    width: u32,
    height: u32,
    view: &FieldView,
    caret: Option<usize>,
) {
    if view.multiline {
        for (i, line) in view
            .lines
            .iter()
            .enumerate()
            .skip(view.first_line)
            .take(view.visible_lines)
        {
            let y = view.origin_y + (i - view.first_line) as i32 * view.line_h;
            draw_text_px(
                buf,
                width,
                height,
                view.origin_x,
                y,
                &line.text,
                INK,
                view.px,
            );
        }
    } else if let Some(line) = view.lines.first() {
        let rest: String = line.text.chars().skip(view.first_char).collect();
        let shown = fitting_prefix(&rest, view.max_w, view.px);
        draw_text_px(
            buf,
            width,
            height,
            view.origin_x,
            view.origin_y,
            &shown,
            INK,
            view.px,
        );
    }
    if let Some(caret) = caret {
        let Some((x, y)) = view.caret_pos(caret) else {
            return;
        };
        let h = em_height(view.px).max(1) as f32;
        stroke_line(
            buf,
            width,
            height,
            x as f32,
            y as f32,
            x as f32,
            y as f32 + h,
            1.5,
            FOCUS,
        );
    }
}

fn line_spans(text: &str, max_w: u32, px: f32) -> Vec<LineSpan> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return vec![LineSpan {
            start: 0,
            text: String::new(),
        }];
    }
    let mut lines = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\n' {
            lines.push(LineSpan {
                start: i,
                text: String::new(),
            });
            i += 1;
            continue;
        }
        let start = i;
        let mut width = 0u32;
        let mut last_break: Option<usize> = None;
        let mut end = i;
        while end < chars.len() && chars[end] != '\n' {
            let adv = text_width_px(&chars[end].to_string(), px);
            if width.saturating_add(adv) > max_w && end > start {
                break;
            }
            width = width.saturating_add(adv);
            end += 1;
            if chars[end - 1] == ' ' {
                last_break = Some(end);
            }
        }
        if end < chars.len() && chars[end] != '\n' {
            if let Some(br) = last_break.filter(|b| *b > start && *b < end) {
                end = br;
            }
        }
        lines.push(LineSpan {
            start,
            text: chars[start..end].iter().collect(),
        });
        i = end;
        if i < chars.len() && chars[i] == '\n' {
            i += 1;
        }
    }
    if text.ends_with('\n') {
        lines.push(LineSpan {
            start: chars.len(),
            text: String::new(),
        });
    }
    if lines.is_empty() {
        lines.push(LineSpan {
            start: 0,
            text: String::new(),
        });
    }
    lines
}

fn first_visible_char(text: &str, caret: usize, max_w: u32, px: f32) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let caret = caret.min(chars.len());
    let mut start = 0;
    while start < caret {
        let slice: String = chars[start..caret].iter().collect();
        if text_width_px(&slice, px) <= max_w {
            break;
        }
        start += 1;
    }
    start
}

fn index_on_line(line: &LineSpan, local_x: f64, px: f32) -> usize {
    if local_x <= 0.0 {
        return line.start;
    }
    let mut w = 0.0f32;
    for (i, ch) in line.text.chars().enumerate() {
        let adv = text_width_px(&ch.to_string(), px) as f32;
        if local_x < f64::from(w + adv * 0.5) {
            return line.start + i;
        }
        w += adv;
    }
    line.start + line.text.chars().count()
}

fn fitting_prefix(text: &str, max_w: u32, px: f32) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        let mut next = out.clone();
        next.push(ch);
        if text_width_px(&next, px) > max_w && !out.is_empty() {
            break;
        }
        out = next;
    }
    out
}

fn line_box(px: f32, scale: f32) -> i32 {
    em_height(px).max(1) as i32 + (2.0 * scale).round() as i32
}

fn max_visible_lines(rect: Rect, px: f32, scale: f32) -> usize {
    let inner = rect.h as i32 - FIELD_PAD * 2;
    let lh = line_box(px, scale).max(1);
    (inner / lh).max(1) as usize
}

pub struct TextMetrics {
    pub line_count: usize,
    pub visible: usize,
    pub caret_line: usize,
    pub line_h: f64,
}

pub fn text_metrics(value: &str, rect: Rect, caret: usize, scale: f32) -> TextMetrics {
    let px = BODY_PX * scale;
    let max_w = rect.w.saturating_sub((FIELD_PAD as u32) * 2);
    let lines = line_spans(value, max_w, px);
    let caret = caret.min(value.chars().count());
    let caret_line = lines.iter().rposition(|l| l.start <= caret).unwrap_or(0);
    TextMetrics {
        line_count: lines.len(),
        visible: max_visible_lines(rect, px, scale),
        caret_line,
        line_h: f64::from(line_box(px, scale).max(1)),
    }
}

/// Positive `wheel_y` (scroll up) reveals earlier lines.
pub fn scroll_by_wheel(
    scroll_px: f64,
    wheel_y: f64,
    line_h: f64,
    line_count: usize,
    visible: usize,
) -> f64 {
    let line_h = line_h.max(1.0);
    let max = line_count.saturating_sub(visible) as f64 * line_h;
    let base = if scroll_px.is_finite() {
        scroll_px
    } else {
        0.0
    };
    let delta = if wheel_y.is_finite() { wheel_y } else { 0.0 };
    (base - delta).clamp(0.0, max.max(0.0))
}

/// Move `scroll_px` so `caret_line` sits inside the visible window.
pub fn reveal_scroll_px(
    scroll_px: f64,
    caret_line: usize,
    visible: usize,
    line_h: f64,
    line_count: usize,
) -> f64 {
    let line_h = line_h.max(1.0);
    let visible = visible.max(1);
    let max = line_count.saturating_sub(visible) as f64 * line_h;
    let mut s = if scroll_px.is_finite() {
        scroll_px
    } else {
        0.0
    };
    s = s.clamp(0.0, max.max(0.0));
    let top = caret_line as f64 * line_h;
    let bottom = top + line_h;
    let view = visible as f64 * line_h;
    if top < s {
        s = top;
    } else if bottom > s + view {
        s = (bottom - view).max(0.0);
    }
    s.clamp(0.0, max.max(0.0))
}

pub fn clamp_text_px(px: i32, scale: f32, win_h: u32) -> u32 {
    let min = TEXT_H as i32;
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let chrome = dip(TOOLBAR_HEIGHT, scale)
        + dip(STATUS_HEIGHT, scale)
        + dip(PAD, scale) * 2
        + dip(FIELD_H, scale)
        + dip(GAP, scale);
    let room = win_h.saturating_sub(chrome);
    let max_from_window = ((room as f32 / scale).floor() as i32).max(min);
    let max = max_from_window.min(TEXT_H_MAX as i32).max(min);
    px.clamp(min, max) as u32
}

/// Caret after moving `delta` visual lines, keeping the horizontal offset.
pub fn caret_after_line(value: &str, rect: Rect, caret: usize, scale: f32, delta: isize) -> usize {
    let px = BODY_PX * scale;
    let max_w = rect.w.saturating_sub((FIELD_PAD as u32) * 2);
    let lines = line_spans(value, max_w, px);
    if lines.is_empty() {
        return 0;
    }
    let caret = caret.min(value.chars().count());
    let line_i = lines.iter().rposition(|l| l.start <= caret).unwrap_or(0);
    let line = &lines[line_i];
    let local = caret.saturating_sub(line.start);
    let prefix: String = line.text.chars().take(local).collect();
    let x = f64::from(text_width_px(&prefix, px));
    let next = (line_i as isize + delta).clamp(0, lines.len() as isize - 1) as usize;
    index_on_line(&lines[next], x, px)
}

fn draw_resize_grip(buf: &mut [u32], width: u32, height: u32, rect: Rect, scale: f32, color: u32) {
    let s = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let x = (rect.x + rect.w as i32) as f32 - 5.0 * s;
    let y = (rect.y + rect.h as i32) as f32 - 5.0 * s;
    for i in 0..3 {
        let o = i as f32 * 3.0 * s;
        stroke_line(
            buf,
            width,
            height,
            x - 8.0 * s + o,
            y,
            x,
            y - 8.0 * s + o,
            s.max(1.0),
            color,
        );
    }
}

fn draw_text_scrollbar(buf: &mut [u32], width: u32, height: u32, rect: Rect, view: &FieldView) {
    let line_count = view.lines.len();
    let visible = view.visible_cap.max(1);
    if line_count <= visible {
        return;
    }
    let track_w = 3u32;
    let track_h = rect.h.saturating_sub((FIELD_PAD as u32) * 2).max(4);
    let track_x = rect.x + rect.w as i32 - FIELD_PAD - track_w as i32;
    let track_y = rect.y + FIELD_PAD;
    fill_rect(
        buf,
        width,
        height,
        Rect::new(track_x, track_y, track_w, track_h),
        0xE4E4E8,
    );
    let thumb_h = ((track_h as f32) * (visible as f32 / line_count as f32)).max(8.0) as u32;
    let thumb_h = thumb_h.min(track_h);
    let max_first = line_count.saturating_sub(visible).max(1) as f32;
    let t = (view.first_line as f32 / max_first).clamp(0.0, 1.0);
    let thumb_y = track_y + ((track_h - thumb_h) as f32 * t) as i32;
    fill_rect(
        buf,
        width,
        height,
        Rect::new(track_x, thumb_y, track_w, thumb_h),
        MUTED,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_on_second_line_lands_after_newline() {
        let rect = Rect::new(0, 0, 240, 80);
        let px = 15.0;
        let text = "ab\ncd";
        let y = f64::from(FIELD_PAD) + f64::from(line_box(px, 1.0)) + 1.0;
        let i =
            field_view(text, rect, 0, px, true, 1.0, 0.0).index_at(f64::from(FIELD_PAD) + 1.0, y);
        assert_eq!(i, 3, "second line starts at 'c'");
        let end = field_view(text, rect, 0, px, true, 1.0, 0.0)
            .index_at(200.0, f64::from(FIELD_PAD) + 1.0);
        assert_eq!(end, 2, "end of the first line is before the newline");
    }

    #[test]
    fn wrapped_lines_keep_every_character() {
        let px = 15.0;
        let w = text_width_px("hi", px);
        let lines = line_spans("hi there", w, px);
        assert!(lines.len() >= 2, "{lines:?}");
        let rebuilt: String = lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(rebuilt, "hi there");
    }

    #[test]
    fn scroll_px_reaches_lines_below_the_box() {
        let rect = Rect::new(0, 0, 120, 48);
        let px = 15.0;
        let text = "one\ntwo\nthree\nfour\nfive\nsix";
        let top = field_view(text, rect, 0, px, true, 1.0, 0.0);
        assert_eq!(top.first_line, 0);
        let scrolled = field_view(text, rect, 0, px, true, 1.0, f64::from(top.line_h) * 2.0);
        assert!(scrolled.first_line >= 2, "scroll shows later lines");
        let y = f64::from(FIELD_PAD) + 1.0;
        let i = scrolled.index_at(f64::from(FIELD_PAD) + 1.0, y);
        assert_ne!(text.chars().nth(i), Some('o'), "the click is not on 'one'");
    }

    #[test]
    fn reveal_keeps_the_caret_line_in_view() {
        let s = reveal_scroll_px(0.0, 5, 2, 20.0, 8);
        assert!(
            (s - 80.0).abs() < 0.01,
            "lines 4 and 5 stay visible, got {s}"
        );
        let stayed = reveal_scroll_px(s, 5, 2, 20.0, 8);
        assert!((stayed - s).abs() < 0.01);
    }

    #[test]
    fn wheel_up_moves_toward_earlier_lines() {
        let next = scroll_by_wheel(40.0, 15.0, 20.0, 10, 2);
        assert!((next - 25.0).abs() < 0.01);
        let stuck = scroll_by_wheel(0.0, 30.0, 20.0, 10, 2);
        assert_eq!(stuck, 0.0);
    }

    #[test]
    fn taller_text_box_pushes_buttons_and_exposes_a_drag_edge() {
        let anchor = Rect::new(10, 80, 40, 20);
        let short = layout_at(anchor, true, 1.0, 800, 600, TEXT_H);
        let tall = layout_at(anchor, true, 1.0, 800, 600, TEXT_H + 40);
        let short_text = short.text.expect("text");
        let tall_text = tall.text.expect("text");
        assert!(tall_text.h + 1 >= short_text.h + 40);
        assert!(tall.save.y >= short.save.y + 40);
        let resize = tall.resize.expect("resize");
        assert_eq!(
            hit(&tall, resize.x as f64 + 4.0, resize.y as f64 + 2.0),
            Some(PopoverHit::Resize)
        );
        assert_eq!(
            hit(&tall, tall_text.x as f64 + 8.0, tall_text.y as f64 + 8.0),
            Some(PopoverHit::Text)
        );
    }
}

fn stroke_rect(buf: &mut [u32], width: u32, height: u32, rect: Rect, color: u32) {
    let x0 = rect.x as f32;
    let y0 = rect.y as f32;
    let x1 = (rect.x + rect.w as i32) as f32;
    let y1 = (rect.y + rect.h as i32) as f32;
    stroke_line(buf, width, height, x0, y0, x1, y0, 1.0, color);
    stroke_line(buf, width, height, x1, y0, x1, y1, 1.0, color);
    stroke_line(buf, width, height, x0, y1, x1, y1, 1.0, color);
    stroke_line(buf, width, height, x0, y0, x0, y1, 1.0, color);
}
