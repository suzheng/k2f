use super::blit::{blend_rect, blit_raster, LETTERBOX};
use super::chrome::window_title;
use super::coords::PageView;
use super::hud::{draw_hud, HUD_HEIGHT};
use super::input::{step_zoom, Action};
use super::raster::{decode_png, rgba_xrgb, scale_rgba, Raster};
use super::scroll::clamp_scroll;
use super::stack::{content_height, hit_index, origin_y, page_at_scroll, page_tops, page_view};
use crate::copy::{CopyPayload, RectPt};
use crate::AppState;
use image::RgbaImage;

/// Window-independent viewer: stacked pages, zoom, drag-copy, PNG blit.
pub struct Session {
    app: AppState,
    pages: Vec<RgbaImage>,
    scaled: Vec<Raster>,
    scroll_y: f64,
    win_w: u32,
    win_h: u32,
    drag_from: Option<(f64, f64)>,
    drag_to: Option<(f64, f64)>,
    drag_page: Option<usize>,
    selection: Option<RectPt>,
    selection_page: Option<usize>,
}

impl Session {
    pub fn new(app: AppState) -> anyhow::Result<Self> {
        let mut s = Self {
            app,
            pages: Vec::new(),
            scaled: Vec::new(),
            scroll_y: 0.0,
            win_w: 1,
            win_h: 1,
            drag_from: None,
            drag_to: None,
            drag_page: None,
            selection: None,
            selection_page: None,
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
                let (w, h) = super::coords::scaled_png_size(p.width(), p.height(), self.app.zoom());
                (w, h.saturating_add(HUD_HEIGHT))
            }
            None => (640, 480 + HUD_HEIGHT),
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

    pub fn official_pixel(&self, x: u32, y: u32) -> Option<u32> {
        self.official_pixel_at(self.app.page(), x, y)
    }

    pub fn official_pixel_at(&self, page: usize, x: u32, y: u32) -> Option<u32> {
        let p = self.pages.get(page)?;
        if x >= p.width() || y >= p.height() {
            return None;
        }
        Some(rgba_xrgb(*p.get_pixel(x, y)))
    }

    pub fn scroll_by(&mut self, dy: f64) {
        self.scroll_y += dy;
        self.clamp_scroll();
        self.sync_page();
    }

    pub fn jump_to_page(&mut self, page: usize) {
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
                step_zoom(&mut self.app, true);
                self.rebuild_scaled();
                self.clamp_scroll();
                self.sync_page();
                let (w, h) = self.scaled_size();
                self.win_w = w;
                self.win_h = h;
                None
            }
            Action::ZoomOut => {
                step_zoom(&mut self.app, false);
                self.rebuild_scaled();
                self.clamp_scroll();
                self.sync_page();
                let (w, h) = self.scaled_size();
                self.win_w = w;
                self.win_h = h;
                None
            }
            Action::Copy => self.active_copy(),
            Action::Export => None,
        }
    }

    pub fn pointer_down(&mut self, x: f64, y: f64) {
        if y < f64::from(HUD_HEIGHT) {
            return;
        }
        let views = self.all_views(self.win_w, self.win_h);
        self.drag_page = hit_index(x, y, &views);
        self.drag_from = Some((x, y));
        self.drag_to = Some((x, y));
        self.selection = None;
        self.selection_page = None;
    }

    pub fn pointer_move(&mut self, x: f64, y: f64) {
        if self.drag_from.is_some() {
            self.drag_to = Some((x, y));
        }
    }

    pub fn pointer_up(&mut self, x: f64, y: f64) -> Option<CopyPayload> {
        let from = self.drag_from.take()?;
        self.drag_to = None;
        let page = self.drag_page.take()?;
        let view = self.view_at(page, self.win_w, self.win_h);
        let (ax, ay) = view.window_to_pt(from.0, from.1);
        let (bx, by) = view.window_to_pt(x, y);
        let sel = RectPt::from_drag(ax, ay, bx, by);
        if sel.is_empty() {
            self.selection = None;
            self.selection_page = None;
            return None;
        }
        self.selection = Some(sel);
        self.selection_page = Some(page);
        self.app.copy_selection_at(page, sel)
    }

    pub fn set_selection(&mut self, sel: RectPt) {
        if sel.is_empty() {
            self.selection = None;
            self.selection_page = None;
        } else {
            self.selection = Some(sel);
            self.selection_page = Some(self.app.page());
        }
    }

    pub fn active_copy(&self) -> Option<CopyPayload> {
        self.app
            .copy_selection_at(self.selection_page?, self.selection?)
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_from.is_some()
    }

    pub fn compose_frame(&self, win_w: u32, win_h: u32) -> Vec<u32> {
        let n = win_w as usize * win_h as usize;
        let mut buf = vec![LETTERBOX; n];
        let views = self.all_views(win_w, win_h);
        for (page, raster) in self.scaled.iter().enumerate() {
            let Some(view) = views.get(page) else {
                continue;
            };
            blit_raster(&mut buf, win_w, win_h, raster, view);
            if self.live_page() == Some(page) {
                if let Some(sel) = self.live_sel(view) {
                    let (x0, y0) = view.pt_to_window(sel.x0, sel.y0);
                    let (x1, y1) = view.pt_to_window(sel.x1, sel.y1);
                    blend_rect(&mut buf, win_w, win_h, x0, y0, x1, y1);
                }
            }
        }
        draw_hud(&mut buf, win_w, win_h, &self.app);
        buf
    }

    fn live_page(&self) -> Option<usize> {
        self.drag_page.or(self.selection_page)
    }

    fn live_sel(&self, view: &PageView) -> Option<RectPt> {
        if let (Some(a), Some(b)) = (self.drag_from, self.drag_to) {
            let (ax, ay) = view.window_to_pt(a.0, a.1);
            let (bx, by) = view.window_to_pt(b.0, b.1);
            let sel = RectPt::from_drag(ax, ay, bx, by);
            return (!sel.is_empty()).then_some(sel);
        }
        self.selection
    }

    fn reload_pages(&mut self) -> anyhow::Result<()> {
        self.pages.clear();
        for i in 0..self.app.page_count() {
            match self.app.render_page_png(i) {
                Ok(png) => self.pages.push(decode_png(&png)?),
                Err(_) => break,
            }
        }
        self.rebuild_scaled();
        Ok(())
    }

    fn rebuild_scaled(&mut self) {
        let z = self.app.zoom();
        self.scaled = self.pages.iter().map(|p| scale_rgba(p, z)).collect();
    }

    fn scaled_heights(&self) -> Vec<u32> {
        self.scaled.iter().map(|r| r.height).collect()
    }

    fn tops(&self) -> Vec<f64> {
        page_tops(&self.scaled_heights())
    }

    fn viewport_h(&self) -> f64 {
        f64::from(self.win_h.saturating_sub(HUD_HEIGHT))
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
        let (pw, ph) = png.map(|p| (p.width(), p.height())).unwrap_or((1, 1));
        page_view(
            win_w,
            pw,
            ph,
            self.app.zoom(),
            origin_y(page, &self.tops(), self.scroll_y),
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
        self.selection = None;
        self.selection_page = None;
    }
}
