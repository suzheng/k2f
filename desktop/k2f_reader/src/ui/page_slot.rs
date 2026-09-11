use super::raster::Raster;

/// Official 2× baseline plus optional higher-density display raster.
#[derive(Debug, Clone)]
pub struct PageSlot {
    pub baseline: Raster,
    /// `(quantize bucket, pixels)` when denser than baseline.
    pub display: Option<(f32, Raster)>,
}

impl PageSlot {
    pub fn from_baseline(baseline: Raster) -> Self {
        Self {
            baseline,
            display: None,
        }
    }

    pub fn layout_size(&self) -> (u32, u32) {
        (self.baseline.width, self.baseline.height)
    }

    pub fn blit_src(&self) -> &Raster {
        match &self.display {
            Some((_, r)) => r,
            None => &self.baseline,
        }
    }

    pub fn display_bucket(&self) -> Option<f32> {
        self.display.as_ref().map(|(s, _)| *s)
    }

    pub fn set_display(&mut self, bucket: f32, raster: Raster) {
        self.display = Some((bucket, raster));
    }

    pub fn clear_display(&mut self) {
        self.display = None;
    }
}
