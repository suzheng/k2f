use crate::error::{AgentError, IMAGE_SIZE};
use k2f_core::Pt;

/// Convert millimetres to engine Pt (1/1000 pt). Agents never see the integer.
pub fn mm_to_pt(mm: f64) -> Result<Pt, AgentError> {
    if !mm.is_finite() || mm <= 0.0 {
        return Err(AgentError::new(
            IMAGE_SIZE,
            format!("declared width must be a positive number of mm, got {mm}"),
        ));
    }
    let milli = (mm * 72_000.0 / 25.4).round() as i128;
    if milli < 1 {
        return Err(AgentError::new(
            IMAGE_SIZE,
            format!("declared width {mm} mm is smaller than 0.001 pt"),
        ));
    }
    Ok(Pt(milli))
}

pub fn scale_height(width: Pt, px_w: u32, px_h: u32) -> Result<Pt, AgentError> {
    if px_w == 0 || px_h == 0 {
        return Err(AgentError::new(
            IMAGE_SIZE,
            "image pixel size must be positive",
        ));
    }
    let h = width.0.saturating_mul(px_h as i128) / px_w as i128;
    if h < 1 {
        return Err(AgentError::new(
            IMAGE_SIZE,
            "computed image height is smaller than 0.001 pt",
        ));
    }
    Ok(Pt(h))
}
