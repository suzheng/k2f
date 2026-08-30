use k2f_core::{K2FError, NodeEditError};
use k2f_package::PackageError;

pub const UNKNOWN_ROLE: &str = "UNKNOWN_ROLE";
pub const DUPLICATE_ID: &str = "DUPLICATE_ID";
pub const INVALID_ID: &str = "INVALID_ID";
pub const UNKNOWN_ID: &str = "UNKNOWN_ID";
pub const TABLE_ROW_MISMATCH: &str = "TABLE_ROW_MISMATCH";
pub const IMAGE_SIZE: &str = "IMAGE_SIZE";
pub const FONT_MISSING: &str = "FONT_MISSING";
pub const INVALID_MODIFIER: &str = "INVALID_MODIFIER";
pub const SCHEMA_INVALID: &str = "SCHEMA_INVALID";
pub const UNCLOSED_SECTION: &str = "UNCLOSED_SECTION";
pub const WRONG_CONTENT: &str = "WRONG_CONTENT";
pub const PDF_IS_NOT_A_SOURCE: &str = "PDF_IS_NOT_A_SOURCE";
pub const INVALID_ARGUMENT: &str = "INVALID_ARGUMENT";
pub const CONTENT_HASH_MISMATCH: &str = "CONTENT_HASH_MISMATCH";

#[derive(Debug, thiserror::Error)]
#[error("{code}: {message}")]
pub struct AgentError {
    pub code: &'static str,
    pub message: String,
}

impl AgentError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<NodeEditError> for AgentError {
    fn from(e: NodeEditError) -> Self {
        match e {
            NodeEditError::UnknownId(id) => {
                Self::new(UNKNOWN_ID, format!("no node with id '{id}'"))
            }
            NodeEditError::NotText(id) => {
                Self::new(WRONG_CONTENT, format!("node '{id}' is not text"))
            }
            NodeEditError::NotContainer(id) => {
                Self::new(WRONG_CONTENT, format!("node '{id}' is not a container"))
            }
            NodeEditError::InvalidIndex { parent, index, len } => Self::new(
                INVALID_ARGUMENT,
                format!("insert index {index} out of range for '{parent}' (len {len})"),
            ),
            NodeEditError::CannotDeleteRoot => Self::new(INVALID_ARGUMENT, "cannot delete root"),
            NodeEditError::DuplicateId(id) => {
                Self::new(DUPLICATE_ID, format!("duplicate node id '{id}'"))
            }
        }
    }
}

impl From<K2FError> for AgentError {
    fn from(e: K2FError) -> Self {
        let code = match &e {
            K2FError::UnknownRole { .. } | K2FError::UnknownVariant { .. } => UNKNOWN_ROLE,
            K2FError::DuplicateNodeId { .. } => DUPLICATE_ID,
            K2FError::EmptyNodeId | K2FError::InvalidNodeId { .. } => INVALID_ID,
            K2FError::TableRowLengthMismatch { .. }
            | K2FError::TableRequiresColumns { .. }
            | K2FError::TableHeaderRowsTooLarge { .. } => TABLE_ROW_MISMATCH,
            K2FError::InvalidImageSize { .. } => IMAGE_SIZE,
            K2FError::InvalidModifierRange { .. }
            | K2FError::ModifierRangeNotOnCharBoundary { .. } => INVALID_MODIFIER,
            K2FError::ColumnsInvalid { .. }
            | K2FError::ColumnsNested { .. }
            | K2FError::ColumnSpanOutsideColumns { .. } => SCHEMA_INVALID,
            _ => INVALID_ARGUMENT,
        };
        Self::new(code, e.to_string())
    }
}

impl From<PackageError> for AgentError {
    fn from(e: PackageError) -> Self {
        match e {
            PackageError::FontMissing(m) => Self::new(FONT_MISSING, m),
            PackageError::SchemaInvalid(m) => Self::new(SCHEMA_INVALID, m),
            PackageError::PdfIsNotASource => Self::new(
                PDF_IS_NOT_A_SOURCE,
                "PDF is a drawing of a lock, not a K2F source",
            ),
            PackageError::NodeId(m) => {
                if m.contains("duplicate") {
                    Self::new(DUPLICATE_ID, m)
                } else {
                    Self::new(INVALID_ID, m)
                }
            }
            PackageError::Other(m) | PackageError::UnexpectedPath(m) => map_compile_message(&m),
        }
    }
}

pub(crate) fn map_compile_message(msg: &str) -> AgentError {
    if msg.contains("FONT_MISSING") {
        AgentError::new(FONT_MISSING, msg)
    } else if msg.contains("unknown role") || msg.contains("unknown variant") {
        AgentError::new(UNKNOWN_ROLE, msg)
    } else if msg.contains("duplicate node id") {
        AgentError::new(DUPLICATE_ID, msg)
    } else if msg.contains("invalid node id") || msg.contains("node id is empty") {
        AgentError::new(INVALID_ID, msg)
    } else if msg.contains("row") && msg.contains("cells") {
        AgentError::new(TABLE_ROW_MISMATCH, msg)
    } else if msg.contains("image") && msg.contains("positive size") {
        AgentError::new(IMAGE_SIZE, msg)
    } else if msg.contains("modifier") {
        AgentError::new(INVALID_MODIFIER, msg)
    } else {
        AgentError::new(INVALID_ARGUMENT, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_missing_compile_message_keeps_code() {
        let err = map_compile_message(
            "FONT_MISSING: role 'body' font 'Missing' (key 'Missing') is not embedded",
        );
        assert_eq!(err.code, FONT_MISSING);
    }

    #[test]
    fn modifier_engine_error_keeps_code() {
        let err = AgentError::from(K2FError::InvalidModifierRange {
            node_id: "n".into(),
            range: [0, 9],
        });
        assert_eq!(err.code, INVALID_MODIFIER);
    }
}
