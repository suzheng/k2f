mod detect;
mod draw;
mod scale;

pub use detect::page_needs_stamp;
pub use draw::draw_full_page_stamp;
pub use scale::{PdfScale, DEFAULT_EXPORT_SCALE};
