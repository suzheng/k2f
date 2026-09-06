#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    #[default]
    K2f,
    Pdf,
    Pptx,
    Docx,
    Markdown,
    Png,
    Jpg,
}

impl ExportFormat {
    pub const ALL: [Self; 7] = [
        Self::K2f,
        Self::Pdf,
        Self::Pptx,
        Self::Docx,
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
            Self::Pptx => "PPTX",
            Self::Docx => "DOCX",
            Self::Markdown => "MD",
            Self::Png => "PNG",
            Self::Jpg => "JPG",
        }
    }

    /// Same labels as `sdk/js/viewer/export-format.js` (`EXPORT_FORMATS`).
    pub fn action_label(self) -> &'static str {
        match self {
            Self::K2f => "Export as K2F",
            Self::Pdf => "Export as PDF",
            Self::Pptx => "Export as PowerPoint",
            Self::Docx => "Export as Word",
            Self::Markdown => "Export as Markdown",
            Self::Png => "Export as PNG",
            Self::Jpg => "Export as JPG",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::K2f => "K2F",
            Self::Pdf => "pdf",
            Self::Pptx => "pptx",
            Self::Docx => "docx",
            Self::Markdown => "md",
            Self::Png => "png",
            Self::Jpg => "jpg",
        }
    }

    pub fn dialog_filter(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::K2f => ("K2F", &["K2F", "k2f"]),
            Self::Pdf => ("PDF", &["pdf"]),
            Self::Pptx => ("PowerPoint", &["pptx"]),
            Self::Docx => ("Word", &["docx"]),
            Self::Markdown => ("Markdown", &["md"]),
            Self::Png => ("PNG", &["png", "zip"]),
            Self::Jpg => ("JPEG", &["jpg", "jpeg", "zip"]),
        }
    }
}
