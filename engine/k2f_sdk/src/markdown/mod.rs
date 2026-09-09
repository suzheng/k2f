mod export;
mod id;
mod import;
mod inline;
mod report;

use crate::error::{AgentError, INVALID_ARGUMENT};
use crate::page::PageSize;
use std::path::{Path, PathBuf};

pub use export::k2f_to_markdown;
pub use import::markdown_to_k2f;
pub use report::ConversionReport;

#[derive(Debug, Clone)]
pub struct MarkdownOptions {
    pub title: String,
    pub page_size: PageSize,
    /// Author directory used as the document shell (theme + fonts). Ignored when `template_bytes` is set.
    pub template_dir: PathBuf,
    /// Packed `.K2F` used as the document shell (WASM / in-memory). Takes precedence over `template_dir`.
    pub template_bytes: Option<Vec<u8>>,
    pub image_base: PathBuf,
    /// If set, replaces the shell primary font bytes.
    pub font_bytes: Option<Vec<u8>>,
}

impl MarkdownOptions {
    /// Shell is an existing author directory (manifest, theme, fonts).
    pub fn new(
        title: impl Into<String>,
        template_dir: impl AsRef<Path>,
    ) -> Result<Self, AgentError> {
        let dir = template_dir.as_ref();
        if !dir.is_dir() {
            return Err(AgentError::new(
                INVALID_ARGUMENT,
                format!(
                    "markdown template must be an author directory: {}",
                    dir.display()
                ),
            ));
        }
        Ok(Self {
            title: title.into(),
            page_size: PageSize::A4,
            template_dir: dir.to_path_buf(),
            template_bytes: None,
            image_base: PathBuf::new(),
            font_bytes: None,
        })
    }

    /// Shell is a packed `.K2F` (used by the WASM binding).
    pub fn from_bytes(title: impl Into<String>, template_bytes: Vec<u8>) -> Self {
        Self {
            title: title.into(),
            page_size: PageSize::A4,
            template_dir: PathBuf::new(),
            template_bytes: Some(template_bytes),
            image_base: PathBuf::new(),
            font_bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarkdownResult {
    pub bytes: Vec<u8>,
    pub report: ConversionReport,
}
