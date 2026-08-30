#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    #[default]
    K2f,
    Pdf,
    Markdown,
    Png,
    Jpg,
}

impl ExportFormat {
    pub const ALL: [Self; 5] = [
        Self::K2f,
        Self::Pdf,
        Self::Markdown,
        Self::Png,
        Self::Jpg,
    ];

    pub fn toggle(self) -> Self {
        let i = Self::ALL.iter().position(|&f| f == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn hud_label(self) -> &'static str {
        match self {
            Self::K2f => "K2F",
            Self::Pdf => "PDF",
            Self::Markdown => "MD",
            Self::Png => "PNG",
            Self::Jpg => "JPG",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::K2f => "K2F",
            Self::Pdf => "pdf",
            Self::Markdown => "md",
            Self::Png => "png",
            Self::Jpg => "jpg",
        }
    }

    pub fn dialog_filter(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::K2f => ("K2F", &["K2F", "k2f"]),
            Self::Pdf => ("PDF", &["pdf"]),
            Self::Markdown => ("Markdown", &["md"]),
            Self::Png => ("PNG", &["png", "zip"]),
            Self::Jpg => ("JPEG", &["jpg", "jpeg", "zip"]),
        }
    }
}
