use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("UNLOCKED: no published lock to draw")]
    Unlocked,
    #[error("UNKNOWN_PAINT_OP: lock contains a paint instruction this exporter cannot execute")]
    UnknownOp,
    #[error("PDF cannot execute raster-only paint op '{0}' without a second layout pass")]
    RasterOp(&'static str),
    #[error("PDF export scale must be 2, 3, or 4 (got {0})")]
    InvalidScale(f32),
    #[error("{0}")]
    Paint(#[from] k2f_paint::PaintError),
    #[error("package: {0}")]
    Package(String),
    #[error("pdf: {0}")]
    Write(String),
}

impl From<k2f_package::PackageError> for PdfError {
    fn from(e: k2f_package::PackageError) -> Self {
        PdfError::Package(e.to_string())
    }
}
