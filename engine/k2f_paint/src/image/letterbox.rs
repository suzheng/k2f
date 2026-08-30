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

#[cfg(test)]
mod tests {
    use super::*;

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
}
