use super::blit::{blend_rect, blit_raster, page_in_window, LETTERBOX};
use super::chrome::{page_inset_y_at, window_chrome_h_at, window_title};
use super::coords::PageView;
use super::discard_dialog::{self, DiscardHit, DiscardPrompt};
use super::display_scale::{needed_paint_scale, quantize_paint_scale, DISPLAY_PAINT_DEBOUNCE};
use super::draw::{fill_rect, Rect};
use super::edit::{EditState, PopoverField};
use super::empty;
use super::form_fill::FillState;
use super::form_overlay::{draw_fields, hit_field};
use super::hud::{
    chrome_hit_at, dip, draw_hud, draw_scrollbar, in_chrome, ChromeHit, ChromePaint,
    SCROLLBAR_WIDTH, STATUS_HEIGHT,
};
use super::input::{next_zoom_step, Action};
use super::page_slot::PageSlot;
use super::pdf_dialog::{
    draw as draw_pdf_dialog, hit_at as pdf_dialog_hit_at, PdfDialogHit, PdfDialogState,
};
use super::popover::{self, anchor_rect, PopoverHit};
use super::raster::{decode_png, Raster};
use super::scroll::clamp_scroll;
use super::stack::{content_height, hit_index, origin_y, page_at_scroll, page_tops, page_view};
use super::zoom::scroll_to_keep_anchor;
use crate::copy::{slices_at, span_contains, CopyPayload, RectPt};
use crate::AppState;
use anyhow::Context;
use k2f_paint::OFFICIAL_PNG_SCALE;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Result of a relock that the window still has to finish.
#[derive(Debug)]
pub enum StagedSave {
    /// Package bytes are already the file at the open path, and the session shows them.
    Written,
    /// No open path. The window must pick a file, write these bytes, then [`Session::commit_package`].
    NeedsPath(Vec<u8>),
}

/// I-beam over lock text, pointer on chrome — same cues as the web viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerCursor {
    Default,
    Pointer,
    Text,
    /// Vertical resize, for the node text box's bottom edge.
    NsResize,
}

#[derive(Clone, Copy)]
struct TextResize {
    y: f64,
    px: u32,
}

/// Window-independent viewer: stacked pages, zoom, drag-select, PNG blit.
pub struct Session {
    app: Option<AppState>,
    open_error: Option<String>,
    pages: Vec<PageSlot>,
    scroll_y: f64,
    win_w: u32,
    win_h: u32,
    drag_from: Option<(f64, f64)>,
    drag_to: Option<(f64, f64)>,
    drag_page: Option<usize>,
    sel_from: Option<(f64, f64)>,
    sel_to: Option<(f64, f64)>,
    selection_page: Option<usize>,
    hover: Option<ChromeHit>,
    pressed: Option<ChromeHit>,
    scale: f32,
    export_menu_open: bool,
    pdf_dialog: PdfDialogState,
    discard_prompt: DiscardPrompt,
    fill: FillState,
    edit: EditState,
    popover_press: Option<PopoverHit>,
    /// Design-pixel height of the node text box. Dragging its bottom edge changes this.
    text_box_px: u32,
    /// Vertical scroll of the node text box, in window pixels.
    text_scroll_px: f64,
    text_resize: Option<TextResize>,
    node_clipboard: Option<String>,
    /// Open `.K2F`. Save and relock replaces this file before the session reloads.
    source_path: Option<PathBuf>,
    staged_save: Option<StagedSave>,
    /// Last UI zoom change; display LOD waits [`DISPLAY_PAINT_DEBOUNCE`] after this.
    zoom_changed_at: Option<Instant>,
    /// Disk copy is newer and unsaved form values blocked an automatic reload.
    disk_newer: bool,
}

impl Session {
    pub fn empty() -> Self {
        Self {
            app: None,
            open_error: None,
            pages: Vec::new(),
            scroll_y: 0.0,
            win_w: super::hud::DEFAULT_INNER_W,
            win_h: super::hud::DEFAULT_INNER_H,
            drag_from: None,
            drag_to: None,
            drag_page: None,
            sel_from: None,
            sel_to: None,
            selection_page: None,
            hover: None,
            pressed: None,
            scale: 1.0,
            export_menu_open: false,
            pdf_dialog: PdfDialogState::new(),
            discard_prompt: DiscardPrompt::default(),
            fill: FillState::default(),
            edit: EditState::default(),
            popover_press: None,
            text_box_px: popover::TEXT_H,
            text_scroll_px: 0.0,
            text_resize: None,
            node_clipboard: None,
            source_path: None,
            staged_save: None,
            zoom_changed_at: None,
            disk_newer: false,
        }
    }

    pub fn new(app: AppState) -> anyhow::Result<Self> {
        let mut s = Self::empty();
        s.replace_doc(app)?;
        Ok(s)
    }

    pub fn load(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.replace_doc(AppState::open(bytes)?)
    }

    pub fn load_path(&mut self, path: &Path) -> anyhow::Result<()> {
        let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        self.load(&bytes)
    }

    /// Replace the open lock from `bytes` without jumping back to the top.
    /// Zoom, page, scroll, and copy/export format stay. Selection and unsaved
    /// form values are dropped. A bad package leaves the current document.
    pub fn reload_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        let Some(current) = self.app.as_ref() else {
            return self.load(bytes);
        };
        let zoom = current.zoom();
        let page = current.page();
        let copy_format = current.copy_format();
        let export_format = current.export_format();
        let scroll = self.scroll_y;
        let mut app = AppState::open(bytes)?;
        app.set_zoom(zoom);
        app.set_copy_format(copy_format);
        app.set_export_format(export_format);
        if app.page_count() > 0 {
            app.set_page(page.min(app.page_count() - 1));
        }
        self.app = Some(app);
        self.open_error = None;
        self.clear_drag();
        self.scroll_y = scroll;
        self.hover = None;
        self.pressed = None;
        self.export_menu_open = false;
        self.pdf_dialog = PdfDialogState::new();
        self.discard_prompt = DiscardPrompt::default();
        self.edit.leave();
        self.fill.reset();
        self.popover_press = None;
        self.reset_text_box();
        self.node_clipboard = None;
        self.disk_newer = false;
        self.reload_pages()?;
        self.clamp_scroll();
        self.sync_page();
        Ok(())
    }

    pub fn disk_newer(&self) -> bool {
        self.disk_newer
    }

    pub fn set_disk_newer(&mut self, newer: bool) {
        self.disk_newer = newer;
    }

    pub fn set_open_error(&mut self, msg: String) {
        self.open_error = Some(msg);
    }

    pub fn set_edit_error(&mut self, msg: String) {
        self.edit.set_error(msg);
    }

    pub fn edit_error(&self) -> Option<&str> {
        self.edit.error()
    }

    pub fn set_source_path(&mut self, path: Option<PathBuf>) {
        self.source_path = path;
    }

    pub fn take_staged_save(&mut self) -> Option<StagedSave> {
        self.staged_save.take()
    }

    /// Show package bytes that were just written by a Save As.
    pub fn commit_package(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.load(bytes)
    }

    pub fn open_error(&self) -> Option<&str> {
        self.open_error.as_deref()
    }

    fn replace_doc(&mut self, app: AppState) -> anyhow::Result<()> {
        self.app = Some(app);
        self.open_error = None;
        self.clear_drag();
        self.scroll_y = 0.0;
        self.hover = None;
        self.pressed = None;
        self.export_menu_open = false;
        self.pdf_dialog = PdfDialogState::new();
        self.discard_prompt = DiscardPrompt::default();
        self.edit.leave();
        self.fill.reset();
        self.popover_press = None;
        self.reset_text_box();
        self.node_clipboard = None;
        self.reload_pages()?;
        let (w, h) = self.scaled_size();
        self.win_w = w;
        self.win_h = h;
        Ok(())
    }

    pub fn app(&self) -> Option<&AppState> {
        self.app.as_ref()
    }

    pub fn app_mut(&mut self) -> Option<&mut AppState> {
        self.app.as_mut()
    }

    pub fn window_title(&self) -> String {
        self.app
            .as_ref()
            .map(window_title)
            .unwrap_or_else(|| "K2F Reader".into())
    }

    pub fn scaled_size(&self) -> (u32, u32) {
        let Some(app) = self.app.as_ref() else {
            return (super::hud::DEFAULT_INNER_W, super::hud::DEFAULT_INNER_H);
        };
        match self.pages.first() {
            Some(p) => {
                let (bw, bh) = p.layout_size();
                let (w, h) = super::coords::scaled_png_size(bw, bh, app.zoom());
                (
                    w.saturating_add(SCROLLBAR_WIDTH),
                    h.saturating_add(window_chrome_h_at(app, self.scale)),
                )
            }
            None => (super::hud::DEFAULT_INNER_W, super::hud::DEFAULT_INNER_H),
        }
    }

    pub fn content_height(&self) -> f64 {
        content_height(&self.scaled_heights())
    }

    pub fn scroll_y(&self) -> f64 {
        self.scroll_y
    }

    pub fn page_view(&self, win_w: u32, win_h: u32) -> PageView {
        self.view_at(
            self.app.as_ref().map(|a| a.page()).unwrap_or(0),
            win_w,
            win_h,
        )
    }

    pub fn set_window_size(&mut self, w: u32, h: u32) {
        self.win_w = w.max(1);
        self.win_h = h.max(1);
        self.clamp_scroll();
        self.sync_page();
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.1);
        self.clamp_scroll();
        self.sync_page();
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn official_pixel(&self, x: u32, y: u32) -> Option<u32> {
        self.official_pixel_at(self.app.as_ref()?.page(), x, y)
    }

    pub fn official_pixel_at(&self, page: usize, x: u32, y: u32) -> Option<u32> {
        self.pages.get(page)?.baseline.get(x, y)
    }

    /// Continuous UI zoom. Layout updates immediately; display paint is debounced.
    pub fn set_zoom(&mut self, z: f32) {
        self.set_zoom_about(z, None);
    }

    /// Zoom while keeping the document point under `focus_y` (window Y) fixed.
    pub fn set_zoom_about(&mut self, z: f32, focus_y: Option<f64>) {
        if self.app.is_none() {
            return;
        }
        let old_heights = self.scaled_heights();
        let old_scroll = self.scroll_y;
        let inset = f64::from(self.inset_y());
        {
            let app = self.app.as_mut().expect("checked above");
            app.set_zoom(z);
        }
        self.note_zoom_changed();
        if let Some(fy) = focus_y {
            let new_heights = self.scaled_heights();
            self.scroll_y =
                scroll_to_keep_anchor(old_scroll, fy, inset, &old_heights, &new_heights);
        }
        self.clamp_scroll();
        self.sync_page();
    }

    /// Window Y at the vertical center of the page viewport (for stepped zoom).
    pub fn viewport_focus_y(&self) -> f64 {
        f64::from(self.inset_y()) + self.viewport_h() * 0.5
    }

    pub fn display_bucket_at(&self, page: usize) -> Option<f32> {
        self.pages.get(page)?.display_bucket()
    }

    /// Force display LOD now (tests / after debounce). Returns true if any slot changed.
    pub fn flush_display_lod(&mut self) -> bool {
        self.zoom_changed_at = None;
        self.refresh_display_lod()
    }

    /// Apply pending display upgrades when debounce elapsed. Returns true if rasters changed.
    pub fn tick_display_lod(&mut self) -> bool {
        let Some(changed) = self.zoom_changed_at else {
            return false;
        };
        if changed.elapsed() < DISPLAY_PAINT_DEBOUNCE {
            return false;
        }
        self.zoom_changed_at = None;
        self.refresh_display_lod()
    }

    /// When `Some`, the event loop should wake by this instant to finish LOD paint.
    pub fn display_wake_at(&self) -> Option<Instant> {
        Some(*self.zoom_changed_at.as_ref()? + DISPLAY_PAINT_DEBOUNCE)
    }

    fn note_zoom_changed(&mut self) {
        self.zoom_changed_at = Some(Instant::now());
    }

    pub fn scroll_by(&mut self, dy: f64) {
        if self.app.is_none() {
            return;
        }
        self.close_export_menu();
        self.scroll_y += dy;
        self.clamp_scroll();
        self.sync_page();
    }

    pub fn jump_to_page(&mut self, page: usize) {
        if self.app.is_none() {
            return;
        }
        self.close_export_menu();
        let tops = self.tops();
        if page >= tops.len() {
            return;
        }
        let view_h = self.viewport_h();
        self.scroll_y = clamp_scroll(tops[page], self.content_height(), view_h);
        self.clear_drag();
        if let Some(app) = self.app.as_mut() {
            app.set_page(page);
        }
    }

    pub fn apply(&mut self, action: Action) -> Option<CopyPayload> {
        match action {
            Action::PrevPage => {
                let page = self.app.as_ref()?.page();
                if page > 0 {
                    self.jump_to_page(page - 1);
                }
                None
            }
            Action::NextPage => {
                let (page, count) = {
                    let app = self.app.as_ref()?;
                    (app.page(), app.page_count())
                };
                if page + 1 < count {
                    self.jump_to_page(page + 1);
                }
                None
            }
            Action::ZoomIn => {
                self.close_export_menu();
                let next = next_zoom_step(self.app.as_ref()?.zoom(), true);
                self.set_zoom(next);
                None
            }
            Action::ZoomOut => {
                self.close_export_menu();
                let next = next_zoom_step(self.app.as_ref()?.zoom(), false);
                self.set_zoom(next);
                None
            }
            Action::Copy => self.active_copy(),
            Action::Export | Action::Open | Action::Save | Action::Reload => None,
        }
    }

    pub fn pointer_down(&mut self, x: f64, y: f64) {
        if self.app.is_none() {
            self.pressed = empty::hit(self.win_w, self.win_h, x, y, self.scale);
            return;
        }
        if self.pdf_dialog.open {
            self.pdf_dialog.pressed =
                pdf_dialog_hit_at(&self.pdf_dialog, self.win_w, self.win_h, x, y, self.scale);
            return;
        }
        if self.discard_prompt.open {
            self.discard_prompt.pressed =
                discard_dialog::hit_at(true, self.win_w, self.win_h, x, y, self.scale);
            return;
        }
        let app = self.app.as_ref().expect("document");
        if in_chrome(
            app,
            self.win_w,
            self.win_h,
            x,
            y,
            self.scale,
            self.export_menu_open,
            self.edit.is_editing(),
        ) {
            self.pressed = chrome_hit_at(
                app,
                self.win_w,
                self.win_h,
                x,
                y,
                self.scale,
                self.export_menu_open,
                self.edit.is_editing(),
            );
            return;
        }
        if self.export_menu_open {
            self.export_menu_open = false;
            self.pressed = None;
            return;
        }
        self.pressed = None;
        self.popover_press = None;
        self.text_resize = None;
        if self.edit.is_editing() {
            if let Some(hit) = self.popover_hit_at(x, y) {
                self.popover_press = Some(hit);
                if hit == PopoverHit::Resize {
                    self.text_resize = Some(TextResize {
                        y,
                        px: self.text_box_px,
                    });
                }
                return;
            }
            if self.fill.is_filling() {
                let views = self.all_views(self.win_w, self.win_h);
                let fields = self
                    .app
                    .as_ref()
                    .map(|a| a.form_fields())
                    .unwrap_or_default();
                if let Some(field) = hit_field(&fields, &views, x, y) {
                    self.edit.clear_popover();
                    self.fill.click(field);
                    return;
                }
            }
        }
        let views = self.all_views(self.win_w, self.win_h);
        self.drag_page = hit_index(x, y, &views);
        self.drag_from = Some((x, y));
        self.drag_to = Some((x, y));
        self.sel_from = None;
        self.sel_to = None;
        self.selection_page = None;
    }

    pub fn pointer_move(&mut self, x: f64, y: f64) -> bool {
        if self.app.is_none() {
            let next = empty::hit(self.win_w, self.win_h, x, y, self.scale);
            if next != self.hover {
                self.hover = next;
                return true;
            }
            return false;
        }
        if self.pdf_dialog.open {
            let next =
                pdf_dialog_hit_at(&self.pdf_dialog, self.win_w, self.win_h, x, y, self.scale);
            if next != self.pdf_dialog.hover {
                self.pdf_dialog.hover = next;
                return true;
            }
            return false;
        }
        if self.text_resize.is_some() {
            self.drag_text_resize(y);
            return true;
        }
        if self.drag_from.is_some() {
            self.drag_to = Some((x, y));
            return true;
        }
        let app = self.app.as_ref().expect("document");
        let next = chrome_hit_at(
            app,
            self.win_w,
            self.win_h,
            x,
            y,
            self.scale,
            self.export_menu_open,
            self.edit.is_editing(),
        );
        if next != self.hover {
            self.hover = next;
            true
        } else {
            false
        }
    }

    pub fn pointer_up(&mut self, x: f64, y: f64) -> Option<CopyPayload> {
        if self.take_discard_click(x, y) {
            return None;
        }
        let resizing = self.text_resize.take().is_some();
        if let Some(pressed) = self.popover_press.take() {
            if !resizing
                && pressed != PopoverHit::Resize
                && self.popover_hit_at(x, y) == Some(pressed)
            {
                self.activate_popover(pressed, x, y);
            }
            return None;
        }
        if self.app.is_none() || self.pdf_dialog.open || self.pressed.is_some() {
            return None;
        }
        let from = self.drag_from.take()?;
        self.drag_to = None;
        let page = self.drag_page.take()?;
        if self.edit.is_editing() && click_not_drag(from, (x, y)) {
            self.sel_from = None;
            self.sel_to = None;
            self.selection_page = None;
            self.open_node_at(page, x, y);
            return None;
        }
        let view = self.view_at(page, self.win_w, self.win_h);
        let (ax, ay) = view.window_to_pt(from.0, from.1);
        let (bx, by) = view.window_to_pt(x, y);
        let payload = self.app.as_ref()?.copy_points_at(page, ax, ay, bx, by);
        if payload.is_some() {
            self.sel_from = Some((ax, ay));
            self.sel_to = Some((bx, by));
            self.selection_page = Some(page);
        } else {
            self.sel_from = None;
            self.sel_to = None;
            self.selection_page = None;
        }
        payload
    }

    pub fn take_chrome_click(&mut self, x: f64, y: f64) -> Option<ChromeHit> {
        let pressed = self.pressed.take()?;
        let now = if self.app.is_none() {
            empty::hit(self.win_w, self.win_h, x, y, self.scale)
        } else {
            chrome_hit_at(
                self.app.as_ref()?,
                self.win_w,
                self.win_h,
                x,
                y,
                self.scale,
                self.export_menu_open,
                self.edit.is_editing(),
            )
        };
        (now == Some(pressed)).then_some(pressed)
    }

    pub fn hover(&self) -> Option<ChromeHit> {
        self.hover
    }

    pub fn pressed(&self) -> Option<ChromeHit> {
        self.pressed
    }

    pub fn chrome_hot(&self) -> bool {
        self.hover.is_some()
    }

    pub fn export_menu_open(&self) -> bool {
        self.export_menu_open
    }

    pub fn toggle_export_menu(&mut self) {
        self.export_menu_open = !self.export_menu_open;
    }

    pub fn close_export_menu(&mut self) -> bool {
        let was = self.export_menu_open;
        self.export_menu_open = false;
        was
    }

    pub fn pdf_dialog_open(&self) -> bool {
        self.pdf_dialog.open
    }

    pub fn open_pdf_dialog(&mut self) {
        self.close_export_menu();
        self.pdf_dialog.open_dialog();
    }

    pub fn close_pdf_dialog(&mut self) -> bool {
        self.pdf_dialog.close_dialog()
    }

    pub fn selected_pdf_scale(&self) -> k2f_pdf::PdfScale {
        self.pdf_dialog.selected_pdf_scale()
    }

    pub fn take_pdf_dialog_click(&mut self, x: f64, y: f64) -> Option<PdfDialogHit> {
        if !self.pdf_dialog.open {
            return None;
        }
        let pressed = self.pdf_dialog.pressed.take()?;
        let now = pdf_dialog_hit_at(&self.pdf_dialog, self.win_w, self.win_h, x, y, self.scale);
        if now != Some(pressed) {
            return None;
        }
        match pressed {
            PdfDialogHit::Scale2 => self.pdf_dialog.selected_scale = 2.0,
            PdfDialogHit::Scale3 => self.pdf_dialog.selected_scale = 3.0,
            PdfDialogHit::Scale4 => self.pdf_dialog.selected_scale = 4.0,
            PdfDialogHit::Cancel | PdfDialogHit::Confirm => {}
        }
        Some(pressed)
    }

    pub fn is_filling(&self) -> bool {
        self.fill.is_filling()
    }

    pub fn is_fill_dirty(&self) -> bool {
        self.fill.is_dirty()
    }

    pub fn fill_editing(&self) -> bool {
        self.fill.active().is_some()
    }

    pub fn is_editing(&self) -> bool {
        self.edit.is_editing()
    }

    pub fn popover_id(&self) -> Option<&str> {
        self.edit.popover().map(|p| p.id.as_str())
    }

    pub fn popover_open(&self) -> bool {
        self.edit.popover().is_some()
    }

    pub fn toggle_edit(&mut self) {
        if self.app.is_none() {
            return;
        }
        if self.edit.is_editing() {
            self.leave_edit();
            return;
        }
        self.edit.enter();
        if self.app.as_ref().is_some_and(|a| a.has_form_fields()) {
            self.fill.set_filling(true);
        }
    }

    pub fn chrome_hit(&self, x: f64, y: f64) -> Option<ChromeHit> {
        let app = self.app.as_ref()?;
        chrome_hit_at(
            app,
            self.win_w,
            self.win_h,
            x,
            y,
            self.scale,
            self.export_menu_open,
            self.edit.is_editing(),
        )
    }

    pub fn popover_save_point(&self) -> Option<(f64, f64)> {
        let layout = self.popover_layout()?;
        Some(rect_center(layout.save))
    }

    pub fn popover_copy_point(&self) -> Option<(f64, f64)> {
        let layout = self.popover_layout()?;
        layout.copy.map(rect_center)
    }

    pub fn popover_cancel_point(&self) -> Option<(f64, f64)> {
        let layout = self.popover_layout()?;
        Some(rect_center(layout.cancel))
    }

    pub fn discard_prompt_open(&self) -> bool {
        self.discard_prompt.open
    }

    pub fn discard_keep_point(&self) -> Option<(f64, f64)> {
        self.discard_prompt
            .open
            .then(|| discard_dialog::keep_point(self.win_w, self.win_h, self.scale))
    }

    pub fn discard_discard_point(&self) -> Option<(f64, f64)> {
        self.discard_prompt
            .open
            .then(|| discard_dialog::discard_point(self.win_w, self.win_h, self.scale))
    }

    /// A point on the text box's bottom edge, where a drag grows the box.
    pub fn popover_resize_point(&self) -> Option<(f64, f64)> {
        let rect = self.popover_layout()?.resize?;
        Some((
            rect.x as f64 + rect.w as f64 / 2.0,
            rect.y as f64 + rect.h as f64 / 2.0,
        ))
    }

    pub fn popover_text_scroll(&self) -> f64 {
        self.text_scroll_px
    }

    /// Scroll the node text box when the pointer is over it and the text overflows.
    /// Returns whether the wheel was consumed.
    pub fn scroll_text_field(&mut self, x: f64, y: f64, wheel_y: f64) -> bool {
        if !matches!(
            self.popover_hit_at(x, y),
            Some(PopoverHit::Text | PopoverHit::Resize)
        ) {
            return false;
        }
        let Some(metrics) = self.text_metrics() else {
            return false;
        };
        if metrics.line_count <= metrics.visible {
            return false;
        }
        self.text_scroll_px = popover::scroll_by_wheel(
            self.text_scroll_px,
            wheel_y,
            metrics.line_h,
            metrics.line_count,
            metrics.visible,
        );
        true
    }

    /// A point inside the node text box, near the first glyph.
    pub fn popover_text_point(&self) -> Option<(f64, f64)> {
        let rect = self.popover_layout()?.text?;
        Some((rect.x as f64 + 8.0, rect.y as f64 + 12.0))
    }

    pub fn popover_role_point(&self) -> Option<(f64, f64)> {
        let layout = self.popover_layout()?;
        layout.role.map(rect_center)
    }

    /// Clipboard JSON for the open node. The Copy node button is hidden;
    /// the save path still uses this.
    pub fn copy_open_node(&mut self) -> Option<String> {
        let json = self.node_clipboard_json()?;
        self.node_clipboard = Some(json.clone());
        Some(json)
    }

    pub fn type_text(&mut self, text: &str) {
        self.fill_insert(text);
    }

    pub fn save_node(&mut self) -> anyhow::Result<Vec<u8>> {
        let (id, role, variant, text) = {
            let pop = self.edit.popover_mut().context("no node")?;
            pop.commit_fields();
            (
                pop.id.clone(),
                pop.role_text().to_string(),
                pop.variant_text().to_string(),
                pop.saved_text(),
            )
        };
        let bytes = self.app.as_ref().context("no document")?.relock_node_edit(
            &id,
            &role,
            &variant,
            text.as_deref(),
        )?;
        self.finish_package(bytes)
    }

    /// Write `bytes` to the open `.K2F`, then load them. With no path, stage a Save As
    /// and leave the current document on screen.
    fn finish_package(&mut self, bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        if let Some(path) = self.source_path.clone() {
            super::persist::replace_file(&path, &bytes)
                .with_context(|| format!("write {}", path.display()))?;
            self.load(&bytes)?;
            self.staged_save = Some(StagedSave::Written);
            return Ok(bytes);
        }
        self.staged_save = Some(StagedSave::NeedsPath(bytes.clone()));
        Ok(bytes)
    }

    pub fn take_node_clipboard(&mut self) -> Option<String> {
        self.node_clipboard.take()
    }

    fn leave_edit(&mut self) {
        self.edit.leave();
        self.fill.reset();
        self.popover_press = None;
        self.reset_text_box();
        self.discard_prompt = DiscardPrompt::default();
    }

    fn reset_text_box(&mut self) {
        self.text_box_px = popover::TEXT_H;
        self.text_scroll_px = 0.0;
        self.text_resize = None;
    }

    fn drag_text_resize(&mut self, y: f64) {
        let Some(drag) = self.text_resize else {
            return;
        };
        let scale = if self.scale.is_finite() && self.scale > 0.0 {
            self.scale
        } else {
            1.0
        };
        let delta = ((y - drag.y) / f64::from(scale)).round() as i32;
        let raw = drag.px as i32 + delta;
        let requested = popover::clamp_text_px(raw, scale, self.win_h);
        self.text_box_px = requested;
        let shown = self
            .popover_layout()
            .and_then(|l| l.text)
            .map(|r| (r.h as f32 / scale).round() as u32)
            .unwrap_or(requested)
            .max(popover::TEXT_H);
        self.text_box_px = shown;
        if shown as i32 != raw {
            self.text_resize = Some(TextResize { y, px: shown });
        }
    }

    fn text_metrics(&self) -> Option<popover::TextMetrics> {
        let (value, caret) = {
            let pop = self.edit.popover()?;
            if !pop.text_enabled() {
                return None;
            }
            let caret = if pop.focus() == PopoverField::Text {
                pop.caret()
            } else {
                0
            };
            (pop.shown(PopoverField::Text), caret)
        };
        let rect = self.popover_layout()?.text?;
        Some(popover::text_metrics(&value, rect, caret, self.scale))
    }

    fn reveal_text_caret(&mut self) {
        let focus_text = self
            .edit
            .popover()
            .is_some_and(|p| p.focus() == PopoverField::Text && p.text_enabled());
        if !focus_text {
            return;
        }
        let Some(metrics) = self.text_metrics() else {
            return;
        };
        self.text_scroll_px = popover::reveal_scroll_px(
            self.text_scroll_px,
            metrics.caret_line,
            metrics.visible,
            metrics.line_h,
            metrics.line_count,
        );
    }

    fn cancel_popover(&mut self) {
        if let Some(pop) = self.edit.popover_mut() {
            pop.commit_fields();
        }
        if self.edit.popover().is_some_and(|p| p.text_dirty()) {
            self.discard_prompt.open = true;
            self.discard_prompt.pressed = None;
            return;
        }
        self.edit.clear_popover();
    }

    fn keep_popover_edits(&mut self) {
        self.discard_prompt = DiscardPrompt::default();
    }

    fn discard_popover(&mut self) {
        self.discard_prompt = DiscardPrompt::default();
        self.edit.clear_popover();
    }

    fn take_discard_click(&mut self, x: f64, y: f64) -> bool {
        if !self.discard_prompt.open {
            return false;
        }
        let pressed = self.discard_prompt.pressed.take();
        let now = discard_dialog::hit_at(true, self.win_w, self.win_h, x, y, self.scale);
        if pressed.is_some() && pressed == now {
            match pressed {
                Some(DiscardHit::Keep) => self.keep_popover_edits(),
                Some(DiscardHit::Discard) => self.discard_popover(),
                None => {}
            }
        }
        true
    }

    pub fn save_fill(&mut self) -> anyhow::Result<Vec<u8>> {
        self.fill.commit_active();
        let dirty = self.fill.dirty().clone();
        let app = self.app.as_ref().context("no document")?;
        let bytes = if dirty.is_empty() {
            app.export_k2f_bytes()?
        } else {
            app.relock_form_values(&dirty)?
        };
        self.finish_package(bytes)
    }

    pub fn fill_insert(&mut self, text: &str) {
        if self.discard_prompt.open {
            return;
        }
        if let Some(pop) = self.edit.popover_mut() {
            pop.insert(text);
        } else {
            self.fill.insert_text(text);
        }
        self.reveal_text_caret();
    }

    pub fn fill_backspace(&mut self) {
        if self.discard_prompt.open {
            return;
        }
        if let Some(pop) = self.edit.popover_mut() {
            pop.backspace();
        } else {
            self.fill.backspace();
        }
        self.reveal_text_caret();
    }

    pub fn fill_newline(&mut self) {
        if self.discard_prompt.open {
            return;
        }
        if let Some(pop) = self.edit.popover_mut() {
            pop.newline();
        } else {
            self.fill.insert_newline();
        }
        self.reveal_text_caret();
    }

    pub fn edit_tab(&mut self) {
        if self.discard_prompt.open {
            return;
        }
        if let Some(pop) = self.edit.popover_mut() {
            pop.tab();
        }
    }

    pub fn edit_arrow(&mut self, delta: isize) {
        if self.discard_prompt.open {
            return;
        }
        if let Some(pop) = self.edit.popover_mut() {
            pop.move_caret(delta);
        }
        self.reveal_text_caret();
    }

    /// Move the text-box caret up or down one visual line.
    pub fn edit_line(&mut self, delta: isize) {
        if self.discard_prompt.open {
            return;
        }
        let (value, caret) = {
            let Some(pop) = self.edit.popover() else {
                return;
            };
            if pop.focus() != PopoverField::Text || !pop.text_enabled() {
                return;
            }
            (pop.shown(PopoverField::Text), pop.caret())
        };
        let Some(rect) = self.popover_layout().and_then(|l| l.text) else {
            return;
        };
        let next = popover::caret_after_line(&value, rect, caret, self.scale, delta);
        if let Some(pop) = self.edit.popover_mut() {
            pop.place_caret(PopoverField::Text, next);
        }
        self.reveal_text_caret();
    }

    pub fn edit_escape(&mut self) -> bool {
        if self.discard_prompt.open {
            self.discard_popover();
            return true;
        }
        if self.edit.popover().is_some() {
            self.edit.clear_popover();
            return true;
        }
        if self.fill.active().is_some() {
            self.fill.blur();
            return true;
        }
        if self.edit.is_editing() {
            self.leave_edit();
            return true;
        }
        false
    }

    pub fn fill_set_preedit(&mut self, preedit: String) {
        if self.discard_prompt.open {
            return;
        }
        if let Some(pop) = self.edit.popover_mut() {
            pop.set_preedit(preedit);
        } else {
            self.fill.set_preedit(preedit);
        }
        self.reveal_text_caret();
    }

    pub fn fill_commit_ime(&mut self, text: String) {
        if self.discard_prompt.open {
            return;
        }
        if let Some(pop) = self.edit.popover_mut() {
            pop.commit_ime(text);
        } else {
            self.fill.commit_ime(text);
        }
        self.reveal_text_caret();
    }

    pub fn fill_cancel_ime(&mut self) {
        if let Some(pop) = self.edit.popover_mut() {
            pop.cancel_ime();
        } else {
            self.fill.cancel_ime();
        }
    }

    pub fn fill_click_id(&mut self, id: &str) -> bool {
        if !self.fill.is_filling() {
            return false;
        }
        let Some(app) = self.app.as_ref() else {
            return false;
        };
        let Some(field) = app.form_fields().into_iter().find(|f| f.id == id) else {
            return false;
        };
        self.fill.click(&field);
        true
    }

    /// Window-pixel caret box for IME, if a text field is focused.
    /// The popover must report one: macOS only delivers typed text while IME is allowed.
    pub fn ime_cursor_area(&self) -> Option<(f64, f64, f64, f64)> {
        if let Some(area) = self.popover_ime_area() {
            return Some(area);
        }
        let active = self.fill.active()?;
        let app = self.app.as_ref()?;
        let field = app.form_fields().into_iter().find(|f| f.id == active.id)?;
        let views = self.all_views(self.win_w, self.win_h);
        let view = views.get(field.page)?;
        let r = super::form_overlay::field_window_rect(view, &field);
        Some((r.x as f64, r.y as f64, r.w as f64, r.h as f64))
    }

    pub fn pointer_over_text(&self, x: f64, y: f64) -> bool {
        let Some(app) = self.app.as_ref() else {
            return false;
        };
        let views = self.all_views(self.win_w, self.win_h);
        let Some(page) = hit_index(x, y, &views) else {
            return false;
        };
        let (px, py) = views[page].window_to_pt(x, y);
        app.text_layer_at(page)
            .iter()
            .any(|s| span_contains(s, px, py))
    }

    pub fn pointer_cursor(&self, x: f64, y: f64) -> PointerCursor {
        if self.app.is_none() {
            return if self.chrome_hot() {
                PointerCursor::Pointer
            } else {
                PointerCursor::Default
            };
        }
        if self.pdf_dialog.open || self.discard_prompt.open {
            return PointerCursor::Pointer;
        }
        if self.text_resize.is_some() {
            return PointerCursor::NsResize;
        }
        match self.popover_hit_at(x, y) {
            Some(PopoverHit::Resize) => return PointerCursor::NsResize,
            Some(PopoverHit::Text | PopoverHit::Role | PopoverHit::Variant) => {
                return PointerCursor::Text;
            }
            Some(_) => return PointerCursor::Pointer,
            None => {}
        }
        if self.chrome_hot() && !self.is_dragging() {
            PointerCursor::Pointer
        } else if self.fill.is_filling() {
            let views = self.all_views(self.win_w, self.win_h);
            let fields = self
                .app
                .as_ref()
                .map(|a| a.form_fields())
                .unwrap_or_default();
            match hit_field(&fields, &views, x, y) {
                Some(f) if f.kind.is_checkbox() => PointerCursor::Pointer,
                Some(_) => PointerCursor::Text,
                None => PointerCursor::Default,
            }
        } else if self.is_dragging() || self.pointer_over_text(x, y) {
            PointerCursor::Text
        } else {
            PointerCursor::Default
        }
    }

    pub fn set_selection(&mut self, sel: RectPt) {
        let Some(app) = self.app.as_ref() else {
            return;
        };
        if sel.is_empty() {
            self.sel_from = None;
            self.sel_to = None;
            self.selection_page = None;
        } else {
            self.sel_from = Some((sel.x0, sel.y0));
            self.sel_to = Some((sel.x1, sel.y1));
            self.selection_page = Some(app.page());
        }
    }

    pub fn active_copy(&self) -> Option<CopyPayload> {
        let page = self.selection_page?;
        let (ax, ay) = self.sel_from?;
        let (bx, by) = self.sel_to?;
        self.app.as_ref()?.copy_points_at(page, ax, ay, bx, by)
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_from.is_some()
    }

    pub fn compose_frame(&self, win_w: u32, win_h: u32) -> Vec<u32> {
        let Some(app) = self.app.as_ref() else {
            return empty::compose_frame(
                win_w,
                win_h,
                self.open_error.as_deref(),
                self.hover,
                self.pressed,
                self.scale,
            );
        };
        let n = win_w as usize * win_h as usize;
        let mut buf = vec![LETTERBOX; n];
        let views = self.all_views(win_w, win_h);
        for (page, slot) in self.pages.iter().enumerate() {
            let Some(view) = views.get(page) else {
                continue;
            };
            if !page_in_window(view, win_w, win_h) {
                continue;
            }
            let (sw, sh) = view.scaled_size();
            fill_rect(
                &mut buf,
                win_w,
                win_h,
                Rect::new(
                    view.origin_x.round() as i32 + 2,
                    view.origin_y.round() as i32 + 2,
                    sw,
                    sh,
                ),
                0xE4E4E8,
            );
            blit_raster(&mut buf, win_w, win_h, slot.blit_src(), view);
            if self.fill.is_filling() {
                if let Some(app) = self.app.as_ref() {
                    let fields: Vec<_> = app
                        .form_fields()
                        .into_iter()
                        .filter(|f| f.page == page)
                        .collect();
                    draw_fields(
                        &mut buf, win_w, win_h, &views, &fields, &self.fill, self.scale,
                    );
                }
            }
            for s in self.live_slices(page, view) {
                let (x0, y0) = view.pt_to_window(s.x_pt, s.y_pt);
                let (x1, y1) = view.pt_to_window(s.x_pt + s.width_pt, s.y_pt + s.height_pt);
                blend_rect(&mut buf, win_w, win_h, x0, y0, x1, y1);
            }
        }
        self.paint_popover(&mut buf, win_w, win_h);
        draw_scrollbar(
            &mut buf,
            win_w,
            win_h,
            app,
            self.scroll_y,
            self.content_height(),
            self.viewport_h(),
            self.scale,
        );
        draw_hud(
            &mut buf,
            win_w,
            win_h,
            app,
            ChromePaint {
                hover: self.hover,
                pressed: self.pressed,
                export_menu_open: self.export_menu_open,
                editing: self.edit.is_editing(),
                dirty: self.fill.is_dirty(),
                disk_newer: self.disk_newer,
                note: self.edit.error().map(str::to_string),
            },
            self.scale,
        );
        discard_dialog::draw(&mut buf, win_w, win_h, &self.discard_prompt, self.scale);
        draw_pdf_dialog(&mut buf, win_w, win_h, &self.pdf_dialog, self.scale);
        buf
    }

    fn paint_popover(&self, buf: &mut [u32], win_w: u32, win_h: u32) {
        let Some(pop) = self.edit.popover() else {
            return;
        };
        let id = pop.id.clone();
        let Some(app) = self.app.as_ref() else {
            return;
        };
        for b in app.doc().boxes_for(&id) {
            let view = self.view_at(b.page, win_w, win_h);
            let rect = anchor_rect(&view, &b);
            blend_rect(
                buf,
                win_w,
                win_h,
                rect.x as f64,
                rect.y as f64,
                (rect.x + rect.w as i32) as f64,
                (rect.y + rect.h as i32) as f64,
            );
        }
        let Some(layout) = self.popover_layout_at(win_w, win_h) else {
            return;
        };
        let Some(pop) = self.edit.popover() else {
            return;
        };
        popover::draw(
            buf,
            win_w,
            win_h,
            pop,
            &layout,
            self.popover_press,
            self.scale,
            self.text_scroll_px,
        );
    }

    fn popover_layout(&self) -> Option<popover::PopoverLayout> {
        self.popover_layout_at(self.win_w, self.win_h)
    }

    fn popover_layout_at(&self, win_w: u32, win_h: u32) -> Option<popover::PopoverLayout> {
        let pop = self.edit.popover()?;
        let id = pop.id.clone();
        let page = pop.page;
        let text_on = pop.text_enabled();
        let app = self.app.as_ref()?;
        let boxes = app.doc().boxes_for(&id);
        let b = boxes.iter().find(|b| b.page == page).or(boxes.first())?;
        let view = self.view_at(b.page, win_w, win_h);
        Some(popover::layout_at(
            anchor_rect(&view, b),
            text_on,
            self.scale,
            win_w,
            win_h,
            self.text_box_px,
        ))
    }

    fn popover_hit_at(&self, x: f64, y: f64) -> Option<PopoverHit> {
        popover::hit(&self.popover_layout()?, x, y)
    }

    fn popover_ime_area(&self) -> Option<(f64, f64, f64, f64)> {
        let (field, caret, value) = {
            let pop = self.edit.popover()?;
            let field = pop.focus();
            (field, pop.caret(), pop.shown(field))
        };
        let layout = self.popover_layout()?;
        popover::input_caret_area(
            &layout,
            field,
            &value,
            caret,
            self.scale,
            self.text_scroll_px,
        )
    }

    fn place_popover_caret(&mut self, field: PopoverField, x: f64, y: f64) {
        let (value, caret) = {
            let Some(pop) = self.edit.popover() else {
                return;
            };
            let caret = if pop.focus() == field { pop.caret() } else { 0 };
            (pop.shown(field), caret)
        };
        let Some(layout) = self.popover_layout() else {
            return;
        };
        let Some(index) = popover::input_click_index(
            &layout,
            field,
            &value,
            caret,
            x,
            y,
            self.scale,
            self.text_scroll_px,
        ) else {
            return;
        };
        if let Some(pop) = self.edit.popover_mut() {
            pop.place_caret(field, index);
        }
        self.reveal_text_caret();
    }

    fn activate_popover(&mut self, hit: PopoverHit, x: f64, y: f64) {
        match hit {
            PopoverHit::Text => self.place_popover_caret(PopoverField::Text, x, y),
            PopoverHit::Role => self.place_popover_caret(PopoverField::Role, x, y),
            PopoverHit::Variant => self.place_popover_caret(PopoverField::Variant, x, y),
            PopoverHit::Panel | PopoverHit::Resize => {}
            PopoverHit::Cancel => self.cancel_popover(),
            PopoverHit::Save => {
                if let Err(err) = self.save_node() {
                    self.edit.set_error(format!("{err:#}"));
                }
            }
            PopoverHit::Copy => {
                if let Some(json) = self.node_clipboard_json() {
                    self.node_clipboard = Some(json);
                }
            }
        }
    }

    fn node_clipboard_json(&self) -> Option<String> {
        let id = self.edit.popover()?.id.clone();
        let clip = self.app.as_ref()?.doc().clipboard(&id)?;
        serde_json::to_string(&clip).ok()
    }

    fn open_node_at(&mut self, page: usize, x: f64, y: f64) {
        let Some(app) = self.app.as_ref() else {
            return;
        };
        let view = self.view_at(page, self.win_w, self.win_h);
        let (px, py) = view.window_to_pt(x, y);
        let Some(hit) = app.doc().hit_test(page, milli_pt(px), milli_pt(py)) else {
            self.fill.blur();
            self.edit.clear_popover();
            return;
        };
        let Some(sel) = app.doc().selection_from_hit(&hit) else {
            self.fill.blur();
            self.edit.clear_popover();
            return;
        };
        if app.form_fields().iter().any(|f| f.id == sel.id) {
            let field = app.form_fields().into_iter().find(|f| f.id == sel.id);
            self.edit.clear_popover();
            if let Some(field) = field {
                self.fill.click(&field);
            }
            return;
        }
        self.fill.blur();
        self.text_scroll_px = 0.0;
        self.text_resize = None;
        self.edit.show(&sel, page);
        self.reveal_text_caret();
    }

    fn live_slices(&self, page: usize, view: &PageView) -> Vec<k2f_paint::TextSpan> {
        let Some(app) = self.app.as_ref() else {
            return Vec::new();
        };
        let spans = app.text_layer_at(page);
        if self.drag_page == Some(page) {
            if let (Some(a), Some(b)) = (self.drag_from, self.drag_to) {
                let (ax, ay) = view.window_to_pt(a.0, a.1);
                let (bx, by) = view.window_to_pt(b.0, b.1);
                return slices_at(&spans, ax, ay, bx, by);
            }
        }
        if self.selection_page == Some(page) {
            if let (Some(a), Some(b)) = (self.sel_from, self.sel_to) {
                return slices_at(&spans, a.0, a.1, b.0, b.1);
            }
        }
        Vec::new()
    }

    fn reload_pages(&mut self) -> anyhow::Result<()> {
        self.pages.clear();
        self.zoom_changed_at = None;
        let Some(app) = self.app.as_ref() else {
            return Ok(());
        };
        for i in 0..app.page_count() {
            match app.render_page_png(i) {
                Ok(png) => {
                    self.pages
                        .push(PageSlot::from_baseline(Raster::from_rgba(&decode_png(
                            &png,
                        )?)))
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn refresh_display_lod(&mut self) -> bool {
        let Some(zoom) = self.app.as_ref().map(|a| a.zoom()) else {
            return false;
        };
        let bucket = quantize_paint_scale(needed_paint_scale(zoom));
        let views = self.all_views(self.win_w, self.win_h);
        let win_w = self.win_w;
        let win_h = self.win_h;
        let mut jobs: Vec<(usize, bool)> = Vec::new();
        for (page, slot) in self.pages.iter().enumerate() {
            let Some(view) = views.get(page) else {
                continue;
            };
            if !page_in_window(view, win_w, win_h) {
                continue;
            }
            if (bucket - OFFICIAL_PNG_SCALE).abs() < 1e-6 {
                if slot.display.is_some() {
                    jobs.push((page, true));
                }
                continue;
            }
            if slot
                .display_bucket()
                .is_some_and(|b| (b - bucket).abs() < 1e-6)
            {
                continue;
            }
            jobs.push((page, false));
        }
        let mut changed = false;
        for (page, clear) in jobs {
            if clear {
                if let Some(slot) = self.pages.get_mut(page) {
                    slot.clear_display();
                    changed = true;
                }
                continue;
            }
            let Some(png) = self
                .app
                .as_ref()
                .and_then(|a| a.render_page_png_at(page, bucket).ok())
            else {
                continue;
            };
            let Ok(img) = decode_png(&png) else {
                continue;
            };
            if let Some(slot) = self.pages.get_mut(page) {
                slot.set_display(bucket, Raster::from_rgba(&img));
                changed = true;
            }
        }
        changed
    }

    fn scaled_heights(&self) -> Vec<u32> {
        let z = self.app.as_ref().map(|a| a.zoom()).unwrap_or(1.0);
        self.pages
            .iter()
            .map(|p| {
                let (bw, bh) = p.layout_size();
                super::coords::scaled_png_size(bw, bh, z).1
            })
            .collect()
    }

    fn tops(&self) -> Vec<f64> {
        page_tops(&self.scaled_heights())
    }

    fn viewport_h(&self) -> f64 {
        let Some(app) = self.app.as_ref() else {
            let top = dip(super::hud::TOOLBAR_HEIGHT, self.scale);
            return f64::from(
                self.win_h
                    .saturating_sub(top.saturating_add(dip(STATUS_HEIGHT, self.scale))),
            );
        };
        // Scroll range must match the painted stage: page_inset_y .. status bar top.
        let inset = page_inset_y_at(app, self.scale);
        f64::from(
            self.win_h
                .saturating_sub(inset.saturating_add(dip(STATUS_HEIGHT, self.scale))),
        )
    }

    fn inset_y(&self) -> u32 {
        self.app
            .as_ref()
            .map(|a| page_inset_y_at(a, self.scale))
            .unwrap_or(super::hud::HUD_HEIGHT)
    }

    fn clamp_scroll(&mut self) {
        self.scroll_y = clamp_scroll(self.scroll_y, self.content_height(), self.viewport_h());
    }

    fn sync_page(&mut self) {
        if self.app.is_none() {
            return;
        }
        let n = page_at_scroll(self.scroll_y, self.viewport_h(), &self.tops());
        if let Some(app) = self.app.as_mut() {
            app.set_page(n);
        }
    }

    fn view_at(&self, page: usize, win_w: u32, _win_h: u32) -> PageView {
        let (pw, ph) = self
            .pages
            .get(page)
            .map(|p| p.layout_size())
            .unwrap_or((1, 1));
        let zoom = self.app.as_ref().map(|a| a.zoom()).unwrap_or(1.0);
        page_view(
            win_w,
            pw,
            ph,
            zoom,
            origin_y(page, &self.tops(), self.scroll_y, f64::from(self.inset_y())),
        )
    }

    fn all_views(&self, win_w: u32, win_h: u32) -> Vec<PageView> {
        (0..self.pages.len())
            .map(|i| self.view_at(i, win_w, win_h))
            .collect()
    }

    fn clear_drag(&mut self) {
        self.drag_from = None;
        self.drag_to = None;
        self.drag_page = None;
        self.sel_from = None;
        self.sel_to = None;
        self.selection_page = None;
    }
}

fn rect_center(rect: super::draw::Rect) -> (f64, f64) {
    (
        rect.x as f64 + rect.w as f64 * 0.5,
        rect.y as f64 + rect.h as f64 * 0.5,
    )
}

fn milli_pt(pt: f64) -> i64 {
    (pt * 1000.0).round() as i64
}

fn click_not_drag(from: (f64, f64), to: (f64, f64)) -> bool {
    let dx = from.0 - to.0;
    let dy = from.1 - to.1;
    dx * dx + dy * dy < 16.0
}
