use tiny_skia::Pixmap;

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
