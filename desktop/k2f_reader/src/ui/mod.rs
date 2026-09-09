//! winit + softbuffer lock viewer. Pixels come from `render_page` at
//! `OFFICIAL_PNG_SCALE`; zoom is blit-time UI scale of that bitmap.
//! Chrome text uses Roboto with NotoSansSC fallback for CJK.

mod blit;
mod chrome;
mod coords;
mod draw;
mod event_loop;
mod font;
mod hud;
mod input;
mod pdf_dialog;
mod raster;
mod scroll;
mod session;
mod stack;

pub use crate::export::{
    default_export_path, default_pdf_path, ensure_extension, ensure_pdf_path, export_file_name,
    pdf_file_name, pick_save_path, ExportFormat,
};
pub use chrome::{banner_copy, overlay_label, page_inset_y, window_chrome_h, window_title};
pub use coords::PageView;
pub use hud::{
    copy_format_hit, export_hit, export_label_x, export_menu_hit, export_menu_item_hit,
    EXPORT_ACTION_LABEL, HUD_HEIGHT, STATUS_HEIGHT,
};
pub use input::{accept_key, key_action, Action, KeyBind};
pub use scroll::{clamp_scroll, line_delta_px, wheel_y_to_scroll, LINE_PX};
pub use session::{PointerCursor, Session};
pub use stack::{content_height, page_at_scroll, page_tops, PAGE_GAP};

use crate::AppState;
use std::path::PathBuf;

/// Open a native window and run until the user closes it.
pub fn run(app: AppState, source: Option<PathBuf>) -> anyhow::Result<()> {
    event_loop::run(app, source)
}
