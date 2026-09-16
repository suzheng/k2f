use pdf_writer::Rect;

pub fn pdf_y(page_h: f64, top_y: f64) -> f32 {
    (page_h - top_y) as f32
}

/// K2F lock box (top-left millipt) → PDF Rect (bottom-left pt).
pub fn pdf_rect(page_h: f64, x: i64, y: i64, width: i64, height: i64) -> Rect {
    let x1 = x as f32 / 1000.0;
    let w = width as f32 / 1000.0;
    let h = height as f32 / 1000.0;
    let y_top = y as f32 / 1000.0;
    let y2 = page_h as f32 - y_top;
    Rect::new(x1, y2 - h, x1 + w, y2)
}
