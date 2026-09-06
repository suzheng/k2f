use super::blit::{blend_rect, blit_raster, page_in_window, LETTERBOX};
use super::chrome::{page_inset_y_at, window_chrome_h_at, window_title};
use super::coords::PageView;
use super::draw::{fill_rect, Rect};
use super::hud::{
    chrome_hit_at, dip, draw_hud, draw_scrollbar, in_chrome, ChromeHit, ChromePaint,
    SCROLLBAR_WIDTH, STATUS_HEIGHT,
};
use super::input::{step_zoom, Action};
use super::raster::{decode_png, Raster};
use super::scroll::clamp_scroll;
use super::stack::{content_height, hit_index, origin_y, page_at_scroll, page_tops, page_view};
use crate::copy::{slices_at, span_contains, CopyPayload, RectPt};
use crate::AppState;

/// I-beam over lock text, pointer on chrome — same cues as the web viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerCursor {
    Default,
    Pointer,
    Text,
}

/// Window-independent viewer: stacked pages, zoom, drag-select, PNG blit.
pub struct Session {
    app: AppState,
    pages: Vec<Raster>,
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
}

impl Session {
    pub fn new(app: AppState) -> anyhow::Result<Self> {
        let mut s = Self {
            app,
            pages: Vec::new(),
            scroll_y: 0.0,
            win_w: 1,
            win_h: 1,
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
        };
        s.reload_pages()?;
        let (w, h) = s.scaled_size();
        s.win_w = w;
        s.win_h = h;
        Ok(s)
    }

    pub fn app(&self) -> &AppState {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut AppState {
        &mut self.app
    }

    pub fn window_title(&self) -> String {
        window_title(&self.app)
    }

    pub fn scaled_size(&self) -> (u32, u32) {
        match self.pages.first() {
            Some(p) => {
                let (w, h) = super::coords::scaled_png_size(p.width, p.height, self.app.zoom());
                (
                    w.saturating_add(SCROLLBAR_WIDTH),
                    h.saturating_add(window_chrome_h_at(&self.app, self.scale)),
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
        self.view_at(self.app.page(), win_w, win_h)
    }

    pub fn set_window_size(&mut self, w: u32, h: u32) {
        self.win_w = w.max(1);
        self.win_h = h.max(1);
        self.clamp_scroll();
        self.sync_page();
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.5);
        self.clamp_scroll();
        self.sync_page();
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn official_pixel(&self, x: u32, y: u32) -> Option<u32> {
        self.official_pixel_at(self.app.page(), x, y)
    }

    pub fn official_pixel_at(&self, page: usize, x: u32, y: u32) -> Option<u32> {
        self.pages.get(page)?.get(x, y)
    }

    pub fn scroll_by(&mut self, dy: f64) {
        self.close_export_menu();
        self.scroll_y += dy;
        self.clamp_scroll();
        self.sync_page();
    }

    pub fn jump_to_page(&mut self, page: usize) {
        self.close_export_menu();
        let tops = self.tops();
        if page >= tops.len() {
            return;
        }
        let view_h = self.viewport_h();
        self.scroll_y = clamp_scroll(tops[page], self.content_height(), view_h);
        self.clear_drag();
        self.app.set_page(page);
    }

    pub fn apply(&mut self, action: Action) -> Option<CopyPayload> {
        match action {
            Action::PrevPage => {
                if self.app.page() > 0 {
                    self.jump_to_page(self.app.page() - 1);
                }
                None
            }
            Action::NextPage => {
                if self.app.page() + 1 < self.app.page_count() {
                    self.jump_to_page(self.app.page() + 1);
                }
                None
            }
            Action::ZoomIn => {
                self.close_export_menu();
                step_zoom(&mut self.app, true);
                self.clamp_scroll();
                self.sync_page();
                None
            }
            Action::ZoomOut => {
                self.close_export_menu();
                step_zoom(&mut self.app, false);
                self.clamp_scroll();
                self.sync_page();
                None
            }
            Action::Copy => self.active_copy(),
            Action::Export => None,
        }
    }

    pub fn pointer_down(&mut self, x: f64, y: f64) {
        if in_chrome(
            &self.app,
            self.win_w,
            self.win_h,
            x,
            y,
            self.scale,
            self.export_menu_open,
        ) {
            self.pressed = chrome_hit_at(
                &self.app,
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
        if self.drag_from.is_some() {
            self.drag_to = Some((x, y));
            return true;
        }
        let next = chrome_hit_at(
            &self.app,
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
        if self.pressed.is_some() {
            return None;
        }
        let from = self.drag_from.take()?;
        self.drag_to = None;
        let page = self.drag_page.take()?;
        let view = self.view_at(page, self.win_w, self.win_h);
        let (ax, ay) = view.window_to_pt(from.0, from.1);
        let (bx, by) = view.window_to_pt(x, y);
        let payload = self.app.copy_points_at(page, ax, ay, bx, by);
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
        let now = chrome_hit_at(
            &self.app,
            self.win_w,
            self.win_h,
            x,
            y,
            self.scale,
            self.export_menu_open,
        );
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

    pub fn pointer_over_text(&self, x: f64, y: f64) -> bool {
        let views = self.all_views(self.win_w, self.win_h);
        let Some(page) = hit_index(x, y, &views) else {
            return false;
        };
        let (px, py) = views[page].window_to_pt(x, y);
        self.app
            .text_layer_at(page)
            .iter()
            .any(|s| span_contains(s, px, py))
    }

    pub fn pointer_cursor(&self, x: f64, y: f64) -> PointerCursor {
        if self.chrome_hot() && !self.is_dragging() {
            PointerCursor::Pointer
        } else if self.is_dragging() || self.pointer_over_text(x, y) {
            PointerCursor::Text
        } else {
            PointerCursor::Default
        }
    }

    pub fn set_selection(&mut self, sel: RectPt) {
        if sel.is_empty() {
            self.sel_from = None;
            self.sel_to = None;
            self.selection_page = None;
        } else {
            self.sel_from = Some((sel.x0, sel.y0));
            self.sel_to = Some((sel.x1, sel.y1));
            self.selection_page = Some(self.app.page());
        }
    }

    pub fn active_copy(&self) -> Option<CopyPayload> {
        let page = self.selection_page?;
        let (ax, ay) = self.sel_from?;
        let (bx, by) = self.sel_to?;
        self.app.copy_points_at(page, ax, ay, bx, by)
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_from.is_some()
    }

    pub fn compose_frame(&self, win_w: u32, win_h: u32) -> Vec<u32> {
        let n = win_w as usize * win_h as usize;
        let mut buf = vec![LETTERBOX; n];
        let views = self.all_views(win_w, win_h);
        for (page, raster) in self.pages.iter().enumerate() {
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
            blit_raster(&mut buf, win_w, win_h, raster, view);
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
            &self.app,
            self.scroll_y,
            self.content_height(),
            self.viewport_h(),
            self.scale,
        );
        draw_hud(
            &mut buf,
            win_w,
            win_h,
            &self.app,
            ChromePaint {
                hover: self.hover,
                pressed: self.pressed,
                export_menu_open: self.export_menu_open,
            },
            self.scale,
        );
        buf
    }

    fn live_slices(&self, page: usize, view: &PageView) -> Vec<k2f_paint::TextSpan> {
        let spans = self.app.text_layer_at(page);
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
        for i in 0..self.app.page_count() {
            match self.app.render_page_png(i) {
                Ok(png) => self.pages.push(Raster::from_rgba(&decode_png(&png)?)),
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn scaled_heights(&self) -> Vec<u32> {
        let z = self.app.zoom();
        self.pages
            .iter()
            .map(|p| super::coords::scaled_png_size(p.width, p.height, z).1)
            .collect()
    }

    fn tops(&self) -> Vec<f64> {
        page_tops(&self.scaled_heights())
    }

    fn viewport_h(&self) -> f64 {
        f64::from(
            self.win_h.saturating_sub(
                super::chrome::chrome_top_at(&self.app, self.scale)
                    .saturating_add(dip(STATUS_HEIGHT, self.scale)),
            ),
        )
    }

    fn clamp_scroll(&mut self) {
        self.scroll_y = clamp_scroll(self.scroll_y, self.content_height(), self.viewport_h());
    }

    fn sync_page(&mut self) {
        let n = page_at_scroll(self.scroll_y, self.viewport_h(), &self.tops());
        self.app.set_page(n);
    }

    fn view_at(&self, page: usize, win_w: u32, _win_h: u32) -> PageView {
        let png = self.pages.get(page);
        let (pw, ph) = png.map(|p| (p.width, p.height)).unwrap_or((1, 1));
        page_view(
            win_w,
            pw,
            ph,
            self.app.zoom(),
            origin_y(
                page,
                &self.tops(),
                self.scroll_y,
                f64::from(page_inset_y_at(&self.app, self.scale)),
            ),
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
