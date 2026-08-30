use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaintError {
    #[error("UNLOCKED: no published lock to paint")]
    Unlocked,
    #[error("scale must be a finite positive number")]
    InvalidScale,
    #[error("unresolved paint ref '{0}' (lock must inline fills and shadows)")]
    UnresolvedRef(String),
    #[error("page index {0} out of range")]
    PageOutOfRange(usize),
    #[error("missing render plan for page index {0}")]
    MissingRenderPlan(usize),
    #[error("ttf-parser failed: {0}")]
    Font(String),
    #[error("missing geometry for DrawText node '{0}'")]
    MissingGeometry(String),
    #[error("failed to create pixmap")]
    Pixmap,
    #[error("failed to build linear gradient shader")]
    Gradient,
    #[error("{0}")]
    Package(String),
    #[error("png encode: {0}")]
    Png(String),
    #[error("missing image '{0}'")]
    MissingImage(String),
    #[error("UNKNOWN_PAINT_OP: lock contains a paint instruction this engine cannot execute")]
    UnknownOp,
    #[error("image: {0}")]
    Image(String),
}

impl From<k2f_package::PackageError> for PaintError {
    fn from(e: k2f_package::PackageError) -> Self {
        PaintError::Package(e.to_string())
    }
}
