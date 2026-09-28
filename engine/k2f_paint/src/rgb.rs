use tiny_skia::Pixmap;

/// Straight RGBA8888 from a premultiplied pixmap. Fully transparent pixels stay
/// alpha 0 (they are not flattened onto white).
pub fn pixmap_to_rgba8(pixmap: &Pixmap) -> Vec<u8> {
    let data = pixmap.data();
    let mut rgba = Vec::with_capacity(data.len());
    for chunk in data.chunks_exact(4) {
        let a = chunk[3];
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let af = a as f32 / 255.0;
            rgba.push((chunk[0] as f32 / af).min(255.0) as u8);
            rgba.push((chunk[1] as f32 / af).min(255.0) as u8);
            rgba.push((chunk[2] as f32 / af).min(255.0) as u8);
            rgba.push(a);
        }
    }
    rgba
}

/// Straight RGB888 from a premultiplied RGBA pixmap (white where alpha is zero).
pub fn pixmap_to_rgb8(pixmap: &Pixmap) -> Vec<u8> {
    let data = pixmap.data();
    let mut rgb = Vec::with_capacity((data.len() / 4) * 3);
    for chunk in data.chunks_exact(4) {
        let a = chunk[3] as f32 / 255.0;
        if a <= 0.0 {
            rgb.extend_from_slice(&[255, 255, 255]);
        } else {
            rgb.push((chunk[0] as f32 / a).min(255.0) as u8);
            rgb.push((chunk[1] as f32 / a).min(255.0) as u8);
            rgb.push((chunk[2] as f32 / a).min(255.0) as u8);
        }
    }
    rgb
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::{Color, Pixmap};

    #[test]
    fn transparent_pixmap_rgba_is_zero_and_rgb_is_white() {
        let mut pixmap = Pixmap::new(1, 1).unwrap();
        pixmap.fill(Color::from_rgba8(0, 0, 0, 0));
        assert_eq!(pixmap_to_rgba8(&pixmap), vec![0, 0, 0, 0]);
        assert_eq!(pixmap_to_rgb8(&pixmap), vec![255, 255, 255]);
    }
}
