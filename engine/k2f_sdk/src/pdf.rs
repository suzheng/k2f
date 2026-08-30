use crate::error::{AgentError, INVALID_ARGUMENT, PDF_IS_NOT_A_SOURCE};
use k2f_pdf::{export_bytes_at, PdfError, PdfScale};

pub fn export_pdf(package_bytes: &[u8]) -> Result<Vec<u8>, AgentError> {
    export_pdf_at(package_bytes, PdfScale::DEFAULT)
}

pub fn export_pdf_at(package_bytes: &[u8], scale: PdfScale) -> Result<Vec<u8>, AgentError> {
    export_bytes_at(package_bytes, scale).map_err(map_pdf)
}

pub fn parse_pdf_scale(scale: f32) -> Result<PdfScale, AgentError> {
    PdfScale::from_f32(scale).map_err(|e| AgentError::new(INVALID_ARGUMENT, e.to_string()))
}

fn map_pdf(e: PdfError) -> AgentError {
    let msg = e.to_string();
    if msg.contains("PDF_IS_NOT_A_SOURCE") {
        return AgentError::new(PDF_IS_NOT_A_SOURCE, msg);
    }
    match e {
        PdfError::Unlocked => AgentError::new("UNLOCKED", msg),
        PdfError::UnknownOp => AgentError::new("UNKNOWN_PAINT_OP", msg),
        PdfError::RasterOp(op) => AgentError::new(
            "RASTER_PAINT_OP",
            format!("PDF cannot fake raster-only paint op '{op}'"),
        ),
        other => AgentError::new(INVALID_ARGUMENT, other.to_string()),
    }
}
