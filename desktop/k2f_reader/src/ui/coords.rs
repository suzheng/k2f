use k2f_paint::OFFICIAL_PNG_SCALE;

/// Placement of the official PNG inside the window.
///
/// Window pixels → document pt: `(px - origin) / (zoom * OFFICIAL_PNG_SCALE)`.
#[derive(Debug, Clone, Copy)]
pub struct PageView {
    pub origin_x: f64,
    pub origin_y: f64,
    pub zoom: f32,
    pub png_w: u32,
    pub png_h: u32,
}

impl PageView {
    pub fn fitted(win_w: u32, win_h: u32, png_w: u32, png_h: u32, zoom: f32) -> Self {
        let (sw, sh) = scaled_png_size(png_w, png_h, zoom);
        Self {
            origin_x: (win_w as f64 - sw as f64) * 0.5,
            origin_y: (win_h as f64 - sh as f64) * 0.5,
            zoom,
            png_w,
            png_h,
        }
    }

    pub fn scaled_size(&self) -> (u32, u32) {
        scaled_png_size(self.png_w, self.png_h, self.zoom)
    }

    /// Shift the page down (reserved HUD strip above the lock PNG).
    pub fn nudge_y(mut self, dy: f64) -> Self {
        self.origin_y += dy;
        self
    }

    fn scale(&self) -> f64 {
        self.zoom as f64 * f64::from(OFFICIAL_PNG_SCALE)
    }

    pub fn window_to_pt(&self, x: f64, y: f64) -> (f64, f64) {
        let s = self.scale();
        ((x - self.origin_x) / s, (y - self.origin_y) / s)
    }

    pub fn pt_to_window(&self, x: f64, y: f64) -> (f64, f64) {
        let s = self.scale();
        (self.origin_x + x * s, self.origin_y + y * s)
    }
}

pub fn scaled_png_size(png_w: u32, png_h: u32, zoom: f32) -> (u32, u32) {
    let w = (png_w as f32 * zoom).round().max(1.0) as u32;
    let h = (png_h as f32 * zoom).round().max(1.0) as u32;
    (w, h)
}
