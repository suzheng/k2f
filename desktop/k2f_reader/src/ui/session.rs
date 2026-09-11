use super::blit::{blend_rect, blit_raster, page_in_window, LETTERBOX};
use super::chrome::{page_inset_y_at, window_chrome_h_at, window_title};
use super::coords::PageView;
use super::display_scale::{
    needed_paint_scale, quantize_paint_scale, DISPLAY_PAINT_DEBOUNCE,
};
use super::draw::{fill_rect, Rect};
use super::empty;
use super::hud::{
    chrome_hit_at, dip, draw_hud, draw_scrollbar, in_chrome, ChromeHit, ChromePaint,
    SCROLLBAR_WIDTH, STATUS_HEIGHT,
};
use super::input::{next_zoom_step, Action};
use super::page_slot::PageSlot;
use super::pdf_dialog::{
    draw as draw_pdf_dialog, hit_at as pdf_dialog_hit_at, PdfDialogHit, PdfDialogState,
};
use super::raster::{decode_png, Raster};
use super::scroll::clamp_scroll;
use super::stack::{content_height, hit_index, origin_y, page_at_scroll, page_tops, page_view};
use super::zoom::scroll_to_keep_anchor;
use crate::copy::{slices_at, span_contains, CopyPayload, RectPt};
use crate::AppState;
use anyhow::Context;
use k2f_paint::OFFICIAL_PNG_SCALE;
use std::path::Path;
use std::time::Instant;

/// I-beam over lock text, pointer on chrome — same cues as the web viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerCursor {
    Default,
    Pointer,
    Text,
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
    /// Last UI zoom change; display LOD waits [`DISPLAY_PAINT_DEBOUNCE`] after this.
    zoom_changed_at: Option<Instant>,
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
            zoom_changed_at: None,
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

    pub fn set_open_error(&mut self, msg: String) {
        self.open_error = Some(msg);
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
            Action::Export | Action::Open => None,
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
        let app = self.app.as_ref().expect("document");
        if in_chrome(
            app,
            self.win_w,
            self.win_h,
            x,
            y,
            self.scale,
            self.export_menu_open,
        ) {
            self.pressed = chrome_hit_at(
                app,
                self.win_w,
                self.win_h,
                x,
                y,
                self.scale,
                self.export_menu_open,
            );
            return;
        }
        if self.export_menu_open {
            self.export_menu_open = false;
            self.pressed = None;
            return;
        }
        self.pressed = None;
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
        );
        if next != self.hover {
            self.hover = next;
            true
        } else {
            false
        }
    }

    pub fn pointer_up(&mut self, x: f64, y: f64) -> Option<CopyPayload> {
        if self.app.is_none() || self.pdf_dialog.open || self.pressed.is_some() {
            return None;
        }
        let from = self.drag_from.take()?;
        self.drag_to = None;
        let page = self.drag_page.take()?;
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
        if self.pdf_dialog.open {
            return PointerCursor::Pointer;
        }
        if self.chrome_hot() && !self.is_dragging() {
            PointerCursor::Pointer
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
            for s in self.live_slices(page, view) {
                let (x0, y0) = view.pt_to_window(s.x_pt, s.y_pt);
                let (x1, y1) = view.pt_to_window(s.x_pt + s.width_pt, s.y_pt + s.height_pt);
                blend_rect(&mut buf, win_w, win_h, x0, y0, x1, y1);
            }
        }
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
            },
            self.scale,
        );
        draw_pdf_dialog(&mut buf, win_w, win_h, &self.pdf_dialog, self.scale);
        buf
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
                Ok(png) => self
                    .pages
                    .push(PageSlot::from_baseline(Raster::from_rgba(&decode_png(&png)?))),
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
        let top = match self.app.as_ref() {
            Some(app) => super::chrome::chrome_top_at(app, self.scale),
            None => dip(super::hud::TOOLBAR_HEIGHT, self.scale),
        };
        f64::from(
            self.win_h
                .saturating_sub(top.saturating_add(dip(STATUS_HEIGHT, self.scale))),
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
