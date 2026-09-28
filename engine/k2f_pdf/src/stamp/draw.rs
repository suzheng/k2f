use pdf_writer::Name;

use crate::coord::pdf_y;
use crate::draw::PageDraw;
use crate::image::ImageRes;

pub fn draw_slice(page: &mut PageDraw, stamp: &ImageRes, crop: &k2f_core::Rect) {
    page.note_image(crop);
    let w = crop.width.as_f64_pt() as f32;
    let h = crop.height.as_f64_pt() as f32;
    let pdf_x = crop.x.as_f64_pt() as f32;
    let top = crop.y.as_f64_pt() + crop.height.as_f64_pt();
    let pdf_y = pdf_y(page.page_h, top);
    page.content.save_state();
    page.content.transform([w, 0.0, 0.0, h, pdf_x, pdf_y]);
    page.content.x_object(Name(stamp.name.as_bytes()));
    page.content.restore_state();
}
