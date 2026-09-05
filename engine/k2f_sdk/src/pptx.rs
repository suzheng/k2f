use crate::error::{AgentError, INVALID_ARGUMENT, PPTX_IS_NOT_A_SOURCE};
use k2f_pptx::{export_bytes, PptxError};

pub fn export_pptx(package_bytes: &[u8]) -> Result<Vec<u8>, AgentError> {
    export_bytes(package_bytes).map_err(map_pptx)
}

fn map_pptx(e: PptxError) -> AgentError {
    let msg = e.to_string();
    if msg.contains("PPTX_IS_NOT_A_SOURCE") {
        return AgentError::new(PPTX_IS_NOT_A_SOURCE, msg);
    }
    match e {
        PptxError::Unlocked => AgentError::new("UNLOCKED", msg),
        PptxError::UnknownOp => AgentError::new("UNKNOWN_PAINT_OP", msg),
        PptxError::NotASource => AgentError::new(PPTX_IS_NOT_A_SOURCE, msg),
        other => AgentError::new(INVALID_ARGUMENT, other.to_string()),
    }
}
