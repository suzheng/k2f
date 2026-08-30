//! winit + softbuffer lock viewer. Pixels come from `render_page` at
//! `OFFICIAL_PNG_SCALE`; zoom is UI scale of that bitmap.

mod blit;
mod chrome;
mod coords;
mod event_loop;
mod font;
mod hud;
mod input;
mod raster;
mod scroll;
mod session;
mod stack;

pub use chrome::{overlay_label, window_title};
pub use coords::PageView;
pub use crate::export::{
    default_export_path, default_pdf_path, ensure_extension, ensure_pdf_path, export_file_name,
    pdf_file_name, pick_save_path, ExportFormat,
};
pub use hud::{
    export_format_hit, export_format_label_x, export_hit, export_label_x, EXPORT_ACTION_LABEL,
    HUD_HEIGHT,
};
pub use input::{accept_key, key_action, Action, KeyBind};
pub use scroll::{clamp_scroll, line_delta_px, LINE_PX};
pub use session::Session;
pub use stack::{content_height, page_at_scroll, page_tops, PAGE_GAP};

use crate::AppState;
use std::path::PathBuf;

/// Open a native window and run until the user closes it.
pub fn run(app: AppState, source: Option<PathBuf>) -> anyhow::Result<()> {
    event_loop::run(app, source)
}
