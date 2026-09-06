use k2f_core::{Pt, Rect};

/// Integer letterbox (contain, centered) of an image into a destination box.
/// Returns (x, y, width, height) inside the box.
pub fn letterbox_dest(
    img_w: u32,
    img_h: u32,
    box_w: u32,
    box_h: u32,
) -> Option<(u32, u32, u32, u32)> {
    if img_w == 0 || img_h == 0 || box_w == 0 || box_h == 0 {
        return None;
    }
    let img_w = img_w as u64;
    let img_h = img_h as u64;
    let box_w = box_w as u64;
    let box_h = box_h as u64;
    let fit_h = img_h.saturating_mul(box_w) / img_w;
    if fit_h <= box_h {
        let y = (box_h - fit_h) / 2;
        Some((0, y as u32, box_w as u32, fit_h.max(1) as u32))
    } else {
        let fit_w = img_w.saturating_mul(box_h) / img_h;
        let x = (box_w - fit_w) / 2;
        Some((x as u32, 0, fit_w.max(1) as u32, box_h as u32))
    }
}

/// Same contain/center fit as [`letterbox_dest`], in lock millipt space.
///
/// Paint and PDF letterbox DrawImage into the lock rect. Office exporters must
/// place the picture on this dest, not stretch-fill the lock box — otherwise a
/// portrait asset in a wide layout box is distorted.
pub fn letterbox_rect(img_w: u32, img_h: u32, rect: &Rect) -> Option<Rect> {
    if img_w == 0 || img_h == 0 || rect.width.0 <= 0 || rect.height.0 <= 0 {
        return None;
    }
    let img_w = img_w as i128;
    let img_h = img_h as i128;
    let box_w = rect.width.0;
    let box_h = rect.height.0;
    let fit_h = img_h.saturating_mul(box_w) / img_w;
    if fit_h <= box_h {
        let y = (box_h - fit_h) / 2;
        Some(Rect {
            x: rect.x,
            y: Pt(rect.y.0 + y),
            width: rect.width,
            height: Pt(fit_h.max(1)),
        })
    } else {
        let fit_w = img_w.saturating_mul(box_h) / img_h;
        let x = (box_w - fit_w) / 2;
        Some(Rect {
            x: Pt(rect.x.0 + x),
            y: rect.y,
            width: Pt(fit_w.max(1)),
            height: rect.height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{Pt, Rect};

    #[test]
    fn wide_image_in_square_has_vertical_bars() {
        assert_eq!(letterbox_dest(20, 10, 100, 100), Some((0, 25, 100, 50)));
    }

    #[test]
    fn tall_image_in_square_has_horizontal_bars() {
        assert_eq!(letterbox_dest(10, 20, 100, 100), Some((25, 0, 50, 100)));
    }

    #[test]
    fn matching_ratio_fills_box() {
        assert_eq!(letterbox_dest(50, 50, 100, 100), Some((0, 0, 100, 100)));
    }

    #[test]
    fn zero_rejects() {
        assert_eq!(letterbox_dest(0, 10, 100, 100), None);
    }

    #[test]
    fn letterbox_rect_tall_image_in_wide_box() {
        let rect = Rect {
            x: Pt(71_000),
            y: Pt(71_000),
            width: Pt(470_000),
            height: Pt(270_000),
        };
        let dest = letterbox_rect(2160, 3238, &rect).unwrap();
        assert_eq!(dest.y, rect.y);
        assert_eq!(dest.height, rect.height);
        assert!(dest.width.0 < rect.width.0);
        assert_eq!(dest.width.0, 2160i128 * 270_000 / 3238);
        assert_eq!(dest.x.0, rect.x.0 + (rect.width.0 - dest.width.0) / 2);
    }

    #[test]
    fn letterbox_rect_matching_ratio_keeps_box() {
        let rect = Rect {
            x: Pt(10),
            y: Pt(20),
            width: Pt(100),
            height: Pt(50),
        };
        assert_eq!(letterbox_rect(20, 10, &rect).unwrap(), rect);
    }
}
