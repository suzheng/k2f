use crate::error::{AgentError, DOCX_IS_NOT_A_SOURCE, INVALID_ARGUMENT};
use k2f_docx::{export_bytes, DocxError};

pub fn export_docx(package_bytes: &[u8]) -> Result<Vec<u8>, AgentError> {
    export_bytes(package_bytes).map_err(map_docx)
}

fn map_docx(e: DocxError) -> AgentError {
    let msg = e.to_string();
    if msg.contains("DOCX_IS_NOT_A_SOURCE") {
        return AgentError::new(DOCX_IS_NOT_A_SOURCE, msg);
    }
    match e {
        DocxError::Unlocked => AgentError::new("UNLOCKED", msg),
        DocxError::UnknownOp => AgentError::new("UNKNOWN_PAINT_OP", msg),
        DocxError::NotASource => AgentError::new(DOCX_IS_NOT_A_SOURCE, msg),
        other => AgentError::new(INVALID_ARGUMENT, other.to_string()),
    }
}
