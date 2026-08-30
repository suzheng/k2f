use k2f_paint::TextSpan;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectPt {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl RectPt {
    pub fn from_drag(ax: f64, ay: f64, bx: f64, by: f64) -> Self {
        Self {
            x0: ax.min(bx),
            y0: ay.min(by),
            x1: ax.max(bx),
            y1: ay.max(by),
        }
    }

    fn is_finite(&self) -> bool {
        self.x0.is_finite() && self.y0.is_finite() && self.x1.is_finite() && self.y1.is_finite()
    }

    /// Zero-area and non-finite drags match a collapsed web selection.
    pub fn is_empty(&self) -> bool {
        !self.is_finite() || self.x0 >= self.x1 || self.y0 >= self.y1
    }

    pub fn intersects_span(&self, s: &TextSpan) -> bool {
        if self.is_empty() || !span_rect_is_finite(s) {
            return false;
        }
        let sx1 = s.x_pt + s.width_pt;
        let sy1 = s.y_pt + s.height_pt;
        !(self.x1 < s.x_pt || self.x0 > sx1 || self.y1 < s.y_pt || self.y0 > sy1)
    }
}

fn span_rect_is_finite(s: &TextSpan) -> bool {
    s.x_pt.is_finite() && s.y_pt.is_finite() && s.width_pt.is_finite() && s.height_pt.is_finite()
}
