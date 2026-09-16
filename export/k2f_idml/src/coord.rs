use k2f_core::{Pt, Rect};

pub const MIME: &str = "application/vnd.adobe.indesign-idml-package";
pub const DOM: &str = "16.0";
pub const NS: &str = "http://ns.adobe.com/AdobeInDesign/idml/1.0/packaging";

pub fn millipt_to_pt(millipt: i128) -> f64 {
    millipt as f64 / 1000.0
}

pub fn pt_val(pt: Pt) -> f64 {
    millipt_to_pt(pt.0)
}

pub fn fmt_pt(v: f64) -> String {
    format!("{v:.3}")
}

#[derive(Clone, Copy, Debug)]
pub struct SpreadSpace {
    pub page_w: f64,
    pub page_h: f64,
}

impl SpreadSpace {
    pub fn new(page_w: Pt, page_h: Pt) -> Self {
        Self {
            page_w: pt_val(page_w),
            page_h: pt_val(page_h),
        }
    }

    pub fn cx(&self) -> f64 {
        self.page_w / 2.0
    }

    pub fn cy(&self) -> f64 {
        self.page_h / 2.0
    }

    pub fn page_geometric_bounds(&self) -> String {
        // IDML GeometricBounds is "top left bottom right". InDesign spread Y
        // increases downward, so page top is -cy and page bottom is +cy.
        format!(
            "{} {} {} {}",
            fmt_pt(-self.cy()),
            fmt_pt(-self.cx()),
            fmt_pt(self.cy()),
            fmt_pt(self.cx())
        )
    }

    /// K2F top-left Y-down pt → IDML pasteboard (spread center, Y-down).
    ///
    /// Adobe's IDML cookbook places the page top at `ty = -pageHeight/2`.
    /// InDesign 2026 PDF export matches that: negative Y is toward the top of
    /// the page. A Y-up mapping mirrored every object.
    pub fn k2f_to_idml(&self, x_pt: f64, y_pt: f64) -> (f64, f64) {
        (x_pt - self.cx(), y_pt - self.cy())
    }

    pub fn box_center(&self, rect: &Rect) -> (f64, f64) {
        let x = pt_val(rect.x) + pt_val(rect.width) / 2.0;
        let y = pt_val(rect.y) + pt_val(rect.height) / 2.0;
        self.k2f_to_idml(x, y)
    }
}

/// Local path around object center, Y-down. Returns TL, TR, BR, BL as `"x y"`.
pub fn local_rect_path(w: f64, h: f64) -> [String; 4] {
    let hw = w / 2.0;
    let hh = h / 2.0;
    [
        format!("{} {}", fmt_pt(-hw), fmt_pt(-hh)),
        format!("{} {}", fmt_pt(hw), fmt_pt(-hh)),
        format!("{} {}", fmt_pt(hw), fmt_pt(hh)),
        format!("{} {}", fmt_pt(-hw), fmt_pt(hh)),
    ]
}

pub fn item_transform(tx: f64, ty: f64) -> String {
    format!("1 0 0 1 {} {}", fmt_pt(tx), fmt_pt(ty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_and_widescreen_pt() {
        assert_eq!(fmt_pt(millipt_to_pt(595_000)), "595.000");
        assert_eq!(fmt_pt(millipt_to_pt(842_000)), "842.000");
        assert_eq!(fmt_pt(millipt_to_pt(960_000)), "960.000");
        assert_eq!(fmt_pt(millipt_to_pt(540_000)), "540.000");
    }

    #[test]
    fn page_top_left_maps_to_upper_left_quadrant() {
        let sp = SpreadSpace {
            page_w: 595.0,
            page_h: 842.0,
        };
        let (x, y) = sp.k2f_to_idml(0.0, 0.0);
        assert!((x + 297.5).abs() < 1e-9);
        assert!((y + 421.0).abs() < 1e-9);
        let (x2, y2) = sp.k2f_to_idml(595.0, 842.0);
        assert!((x2 - 297.5).abs() < 1e-9);
        assert!((y2 - 421.0).abs() < 1e-9);
        assert_eq!(
            sp.page_geometric_bounds(),
            "-421.000 -297.500 421.000 297.500"
        );
    }

    #[test]
    fn full_page_center_is_origin() {
        let sp = SpreadSpace {
            page_w: 595.0,
            page_h: 842.0,
        };
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(595_000),
            height: Pt(842_000),
        };
        let (tx, ty) = sp.box_center(&rect);
        assert!(tx.abs() < 1e-9);
        assert!(ty.abs() < 1e-9);
    }
}
