#[derive(Debug, Clone, Default)]
pub struct ConversionReport {
    pub warnings: Vec<String>,
}

impl ConversionReport {
    pub fn warn(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }
}
