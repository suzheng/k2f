use crate::error::{AgentError, IDML_IS_NOT_A_SOURCE, INVALID_ARGUMENT};
use k2f_idml::{export_bytes, IdmlError};

pub fn export_idml(package_bytes: &[u8]) -> Result<Vec<u8>, AgentError> {
    export_bytes(package_bytes).map_err(map_idml)
}

fn map_idml(e: IdmlError) -> AgentError {
    let msg = e.to_string();
    if msg.contains("IDML_IS_NOT_A_SOURCE") {
        return AgentError::new(IDML_IS_NOT_A_SOURCE, msg);
    }
    match e {
        IdmlError::Unlocked => AgentError::new("UNLOCKED", msg),
        IdmlError::UnknownOp => AgentError::new("UNKNOWN_PAINT_OP", msg),
        IdmlError::NotASource => AgentError::new(IDML_IS_NOT_A_SOURCE, msg),
        other => AgentError::new(INVALID_ARGUMENT, other.to_string()),
    }
}
