mod export;
mod id;
mod import;
mod inline;
mod report;

use crate::error::AgentError;
use crate::page::PageSize;
use crate::templates;
use std::path::PathBuf;

pub use export::k2f_to_markdown;
pub use import::markdown_to_k2f;
pub use report::ConversionReport;

#[derive(Debug, Clone)]
pub struct MarkdownOptions {
    pub title: String,
    pub page_size: PageSize,
    pub template_dir: PathBuf,
    pub image_base: PathBuf,
    /// If set, replaces the template primary font bytes.
    pub font_bytes: Option<Vec<u8>>,
}

impl MarkdownOptions {
    pub fn new(
        title: impl Into<String>,
        template: impl AsRef<std::path::Path>,
    ) -> Result<Self, AgentError> {
        Ok(Self {
            title: title.into(),
            page_size: PageSize::A4,
            template_dir: templates::resolve(template)?,
            image_base: PathBuf::new(),
            font_bytes: None,
        })
    }
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            title: "Document".into(),
            page_size: PageSize::A4,
            template_dir: templates::resolve("report").expect("bundled report template"),
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
