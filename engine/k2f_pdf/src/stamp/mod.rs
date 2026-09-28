mod compose;
mod detect;
mod draw;
mod raster;
mod scale;
mod slice;

pub(crate) use compose::compose_page;
pub use detect::page_needs_stamp;
pub use raster::raster_slice;
pub use scale::{PdfScale, DEFAULT_EXPORT_SCALE};
pub use slice::{box_is_slice, slices_for_page, Slice};
