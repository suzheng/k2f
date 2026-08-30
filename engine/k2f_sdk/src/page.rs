use crate::error::{AgentError, INVALID_ARGUMENT};
use k2f_core::{PageConfig, Pt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    A4,
    Letter,
}

impl PageSize {
    pub fn parse(name: &str) -> Result<Self, AgentError> {
        match name {
            "A4" | "a4" => Ok(Self::A4),
            "Letter" | "letter" => Ok(Self::Letter),
            other => Err(AgentError::new(
                INVALID_ARGUMENT,
                format!("page_size must be A4 or Letter, got {other}"),
            )),
        }
    }

    pub fn page_config(self) -> PageConfig {
        let (width, height) = match self {
            Self::A4 => (Pt(595_000), Pt(842_000)),
            Self::Letter => (Pt(612_000), Pt(792_000)),
        };
        PageConfig {
            width,
            height,
            margin: [Pt(56_000); 4],
        }
    }
}
