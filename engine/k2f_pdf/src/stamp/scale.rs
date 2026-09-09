use crate::error::PdfError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfScale {
    X2,
    X3,
    X4,
}

/// Default raster stamp scale for PDF export (`--scale` / GUI).
pub const DEFAULT_EXPORT_SCALE: f32 = 4.0;

impl PdfScale {
    pub const DEFAULT: Self = Self::X4;

    pub fn as_f32(self) -> f32 {
        match self {
            Self::X2 => 2.0,
            Self::X3 => 3.0,
            Self::X4 => 4.0,
        }
    }

    pub fn from_f32(scale: f32) -> Result<Self, PdfError> {
        if !scale.is_finite() {
            return Err(PdfError::InvalidScale(scale));
        }
        match scale {
            s if (s - 2.0).abs() < f32::EPSILON => Ok(Self::X2),
            s if (s - 3.0).abs() < f32::EPSILON => Ok(Self::X3),
            s if (s - 4.0).abs() < f32::EPSILON => Ok(Self::X4),
            _ => Err(PdfError::InvalidScale(scale)),
        }
    }
}

impl Default for PdfScale {
    fn default() -> Self {
        Self::DEFAULT
    }
}
