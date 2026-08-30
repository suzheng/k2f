//! Native lock executor for `.K2F` packages.
//!
//! Same open / paint / PDF-export rules as the web viewer.

pub mod app;
pub mod cli;
pub mod copy;
pub mod export;
pub mod ui;

pub use app::AppState;
pub use k2f_package::VerifyStatus;
