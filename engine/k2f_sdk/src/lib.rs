//! Agent-facing K2F SDK: edit by node id, Markdown bridge, PDF export.
//!
//! # Example
//!
//! ```
//! use k2f_sdk::Editor;
//!
//! let mut editor = Editor::open_template("report").unwrap();
//! editor.insert_node(
//!     "root",
//!     0,
//!     r#"{"id":"doc.title","role":"h1","content":{"type":"text","value":"Hello"}}"#,
//! )
//! .unwrap();
//! let bytes = editor.save_bytes().unwrap();
//! assert!(!bytes.is_empty());
//! ```

mod agent_json;
mod builder;
mod docx;
mod editor;
mod error;
mod lock;
mod markdown;
mod nodes;
mod page;
mod pdf;
mod pptx;
mod profile;
mod raster;
mod sign;
mod templates;
mod units;
mod vocab;

pub use docx::export_docx;
pub use editor::{Change, ChangeOp, ChangelogEntry, ChangelogFile, Editor, OutlineNode};
pub use error::{
    AgentError, CONTENT_HASH_MISMATCH, DOCX_IS_NOT_A_SOURCE, DUPLICATE_ID, FONT_MISSING,
    IMAGE_SIZE, INVALID_ARGUMENT, INVALID_ID, INVALID_MODIFIER, PDF_IS_NOT_A_SOURCE,
    PPTX_IS_NOT_A_SOURCE, SCHEMA_INVALID, TABLE_ROW_MISMATCH, UNKNOWN_ID, UNKNOWN_ROLE,
    WRONG_CONTENT,
};
pub use k2f_pdf::PdfScale;
pub use markdown::{
    k2f_to_markdown, markdown_to_k2f, ConversionReport, MarkdownOptions, MarkdownResult,
};
pub use page::PageSize;
pub use pdf::{export_pdf, export_pdf_at, parse_pdf_scale};
pub use pptx::export_pptx;
pub use sign::{generate_key, sign, GeneratedKey};
pub use templates::{bundled_root, copy_to, load_package, resolve, OFFICIAL_IDS};

pub const SYSTEM_PROMPT: &str = include_str!("../instructions/agent_v0.md");

/// Pack-path keys (e.g. `schema/nodes.schema.json`) → embedded format schema JSON text.
pub fn format_schemas() -> std::collections::BTreeMap<String, String> {
    k2f_package::bundled_schema_files()
        .into_iter()
        .map(|(path, text)| (path.to_string(), text.to_string()))
        .collect()
}

pub fn validate_agent_json(value: &serde_json::Value) -> Result<(), AgentError> {
    profile::validate_agent_root_json(value)
}
