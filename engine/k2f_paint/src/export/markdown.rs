use k2f_markdown::{document_to_markdown, MarkdownEmitOptions};

use crate::document::OpenedDocument;
use crate::error::PaintError;

/// Full-document Markdown for human export (no k2f hint comments).
pub fn document_markdown(doc: &OpenedDocument) -> Result<String, PaintError> {
    Ok(document_to_markdown(
        doc.semantic_root(),
        doc.running_blocks(),
        MarkdownEmitOptions { hints: false },
    ))
}
