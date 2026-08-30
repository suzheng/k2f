use thiserror::Error;

pub const CODE_VALID: &str = "VALID";
pub const CODE_UNLOCKED: &str = "UNLOCKED";
pub const CODE_UNSIGNED: &str = "UNSIGNED";
pub const CODE_SIGNED: &str = "SIGNED";
pub const CODE_SIGNED_BUT_BROKEN: &str = "SIGNED_BUT_BROKEN";
pub const CODE_CONTENT_CHANGED: &str = "CONTENT_CHANGED";
pub const CODE_APPEARANCE_CHANGED: &str = "APPEARANCE_CHANGED";
pub const CODE_ENGINE_MISMATCH: &str = "ENGINE_MISMATCH";
pub const CODE_FONT_MISSING: &str = "FONT_MISSING";
pub const CODE_SCHEMA_INVALID: &str = "SCHEMA_INVALID";
pub const CODE_NODE_ID: &str = "NODE_ID";
pub const CODE_UNEXPECTED_PATH: &str = "UNEXPECTED_PATH";
pub const CODE_UNKNOWN_PAINT_OP: &str = "UNKNOWN_PAINT_OP";
pub const CODE_PDF_IS_NOT_A_SOURCE: &str = "PDF_IS_NOT_A_SOURCE";

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("{CODE_FONT_MISSING}: {0}")]
    FontMissing(String),
    #[error("{CODE_SCHEMA_INVALID}: {0}")]
    SchemaInvalid(String),
    #[error("{CODE_NODE_ID}: {0}")]
    NodeId(String),
    #[error("{CODE_UNEXPECTED_PATH}: {0}")]
    UnexpectedPath(String),
    #[error("{CODE_PDF_IS_NOT_A_SOURCE}: PDF is a drawing of a lock, not a K2F source")]
    PdfIsNotASource,
    #[error("package error: {0}")]
    Other(String),
}

impl From<std::io::Error> for PackageError {
    fn from(e: std::io::Error) -> Self {
        PackageError::Other(e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyStatus {
    Valid,
    Unlocked,
    ContentChanged,
    AppearanceChanged,
    EngineMismatch,
    FontMissing,
}

impl VerifyStatus {
    pub fn code(self) -> &'static str {
        match self {
            Self::Valid => CODE_VALID,
            Self::Unlocked => CODE_UNLOCKED,
            Self::ContentChanged => CODE_CONTENT_CHANGED,
            Self::AppearanceChanged => CODE_APPEARANCE_CHANGED,
            Self::EngineMismatch => CODE_ENGINE_MISMATCH,
            Self::FontMissing => CODE_FONT_MISSING,
        }
    }
}
