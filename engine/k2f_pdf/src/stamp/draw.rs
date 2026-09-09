use pdf_writer::Name;

use crate::coord::pdf_y;
use crate::draw::PageDraw;
use crate::image::ImageRes;

pub fn draw_full_page_stamp(page: &mut PageDraw, stamp: &ImageRes) {
    page.note_image_full(stamp.w, stamp.h);
    let pdf_x = 0.0_f32;
    let pdf_y = pdf_y(page.page_h, page.page_h);
    let w = page.page_w as f32;
    let h = page.page_h as f32;
    page.content.save_state();
    page.content.transform([w, 0.0, 0.0, h, pdf_x, pdf_y]);
    page.content.x_object(Name(stamp.name.as_bytes()));
    page.content.restore_state();
}
