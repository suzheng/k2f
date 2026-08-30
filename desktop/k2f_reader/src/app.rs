use crate::copy::{CopyFormat, CopyPayload, RectPt};
use crate::export::ExportFormat;
use anyhow::Context;
use k2f_package::VerifyStatus;
use k2f_package::pack_bytes;
use k2f_paint::{Banner, OpenedDocument, TextSpan, OFFICIAL_PNG_SCALE};
use k2f_pdf::export_opened;
use std::path::Path;

const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 3.0;

/// Session over a published lock. Opening never recompiles.
pub struct AppState {
    doc: OpenedDocument,
    page: usize,
    zoom: f32,
    copy_format: CopyFormat,
    export_format: ExportFormat,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("title", &self.title())
            .field("banner", &self.banner())
            .field("status_code", &self.status_code())
            .field("page", &self.page)
            .field("page_count", &self.page_count())
            .field("zoom", &self.zoom)
            .field("copy_format", &self.copy_format)
            .field("export_format", &self.export_format)
            .finish()
    }
}

impl AppState {
    pub fn open(bytes: &[u8]) -> anyhow::Result<Self> {
        let doc = OpenedDocument::open(bytes)?;
        Ok(Self {
            doc,
            page: 0,
            zoom: 1.0,
            copy_format: CopyFormat::Markdown,
            export_format: ExportFormat::Pdf,
        })
    }

    pub fn title(&self) -> &str {
        self.doc.title()
    }

    pub fn banner(&self) -> Banner {
        self.doc.banner()
    }

    pub fn banner_str(&self) -> &'static str {
        self.banner().as_str()
    }

    /// Hash-chain result. Finer than `banner()` (`ENGINE_MISMATCH` vs `BROKEN_INTEGRITY`).
    pub fn status(&self) -> VerifyStatus {
        self.doc.status()
    }

    /// Integrity code shown beside the web banner (`ENGINE_MISMATCH` under `BROKEN_INTEGRITY`).
    pub fn status_code(&self) -> &'static str {
        self.doc.status_code()
    }

    pub fn page_count(&self) -> usize {
        self.doc.page_count()
    }

    pub fn page(&self) -> usize {
        self.page
    }

    pub fn set_page(&mut self, page: usize) {
        if page < self.page_count() {
            self.page = page;
        }
    }

    pub fn next_page(&mut self) {
        if self.page + 1 < self.page_count() {
            self.page += 1;
        }
    }

    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, z: f32) {
        if z.is_nan() {
            return;
        }
        self.zoom = z.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    pub fn copy_format(&self) -> CopyFormat {
        self.copy_format
    }

    pub fn set_copy_format(&mut self, format: CopyFormat) {
        self.copy_format = format;
    }

    pub fn toggle_copy_format(&mut self) {
        self.copy_format = self.copy_format.toggle();
    }

    pub fn export_format(&self) -> ExportFormat {
        self.export_format
    }

    pub fn set_export_format(&mut self, format: ExportFormat) {
        self.export_format = format;
    }

    pub fn toggle_export_format(&mut self) {
        self.export_format = self.export_format.toggle();
    }

    pub fn export_k2f_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(pack_bytes(self.doc.package())?)
    }

    pub fn export_markdown(&self) -> anyhow::Result<String> {
        Ok(self.doc.document_markdown()?)
    }

    pub fn export_pages_png_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.doc.export_pages_png(OFFICIAL_PNG_SCALE)?)
    }

    pub fn export_pages_jpeg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.doc.export_pages_jpeg(OFFICIAL_PNG_SCALE)?)
    }

    pub fn export_bytes(&self, format: ExportFormat) -> anyhow::Result<Vec<u8>> {
        match format {
            ExportFormat::K2f => self.export_k2f_bytes(),
            ExportFormat::Pdf => self.export_pdf_bytes(),
            ExportFormat::Markdown => self
                .export_markdown()
                .map(|s| s.into_bytes()),
            ExportFormat::Png => self.export_pages_png_bytes(),
            ExportFormat::Jpg => self.export_pages_jpeg_bytes(),
        }
    }

    pub fn export_to(&self, format: ExportFormat, path: &Path) -> anyhow::Result<()> {
        let bytes = self.export_bytes(format)?;
        std::fs::write(path, &bytes).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    /// Draw the published lock to `path`. Same bytes as `--export-pdf`.
    pub fn export_pdf_to(&self, path: &Path) -> anyhow::Result<()> {
        self.export_to(ExportFormat::Pdf, path)
    }
    /// Zoom is UI scale of this bitmap; it does not change lock pixels.
    pub fn render_current_png(&self) -> anyhow::Result<Vec<u8>> {
        self.render_page_png(self.page)
    }

    pub fn render_page_png(&self, page: usize) -> anyhow::Result<Vec<u8>> {
        Ok(self.doc.render_page(page, OFFICIAL_PNG_SCALE)?)
    }

    pub fn text_layer(&self) -> Vec<TextSpan> {
        self.text_layer_at(self.page)
    }

    pub fn text_layer_at(&self, page: usize) -> Vec<TextSpan> {
        self.doc.text_layer(page)
    }

    /// Drag-rect copy of the current page's lock text layer (document pt).
    pub fn copy_selection(&self, sel: RectPt) -> Option<CopyPayload> {
        self.copy_selection_at(self.page, sel)
    }

    pub fn copy_selection_at(&self, page: usize, sel: RectPt) -> Option<CopyPayload> {
        CopyPayload::from_spans(&self.text_layer_at(page), sel)?
            .with_format(&self.doc, self.copy_format)
    }

    pub fn export_pdf_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(export_opened(&self.doc, k2f_pdf::PdfScale::DEFAULT)?)
    }

    pub fn doc(&self) -> &OpenedDocument {
        &self.doc
    }
}
