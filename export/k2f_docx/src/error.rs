use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocxError {
    #[error("UNLOCKED: no published lock to draw")]
    Unlocked,
    #[error("UNKNOWN_PAINT_OP: lock contains a paint instruction this exporter cannot execute")]
    UnknownOp,
    #[error("DOCX_IS_NOT_A_SOURCE")]
    NotASource,
    #[error("package has no embedded font")]
    NoFont,
    #[error("{0}")]
    Paint(#[from] k2f_paint::PaintError),
    #[error("docx: {0}")]
    Write(String),
}
