use k2f_paint::{OpenedDocument, OFFICIAL_PNG_SCALE};
use wasm_bindgen::prelude::*;

/// Locked-file viewer: unpack, inspect with the CLI's Rust function, paint the lock.
#[wasm_bindgen]
pub struct K2fViewer {
    doc: OpenedDocument,
}

#[wasm_bindgen]
impl K2fViewer {
    #[wasm_bindgen(constructor)]
    pub fn open(bytes: &[u8]) -> Result<K2fViewer, JsValue> {
        let doc = OpenedDocument::open(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(K2fViewer { doc })
    }

    pub fn banner(&self) -> String {
        self.doc.banner().as_str().to_string()
    }

    pub fn status_code(&self) -> String {
        self.doc.status_code().to_string()
    }

    pub fn hash_code(&self) -> String {
        self.doc.hash_code().to_string()
    }

    pub fn fingerprint(&self) -> Option<String> {
        self.doc.fingerprint().map(|s| s.to_string())
    }

    pub fn signed_by(&self) -> Option<String> {
        self.doc.signed_by().map(|s| s.to_string())
    }

    pub fn signed_at(&self) -> Option<i64> {
        self.doc.signed_at()
    }

    pub fn generated_by(&self) -> Option<String> {
        self.doc.generated_by().map(|s| s.to_string())
    }

    pub fn content_hash(&self) -> Option<String> {
        self.doc.content_hash().map(|s| s.to_string())
    }

    pub fn appearance_hash(&self) -> Option<String> {
        self.doc.appearance_hash().map(|s| s.to_string())
    }

    pub fn page_count(&self) -> u32 {
        self.doc.page_count() as u32
    }

    pub fn title(&self) -> String {
        self.doc.title().to_string()
    }

    pub fn official_scale() -> f32 {
        OFFICIAL_PNG_SCALE
    }

    pub fn page_width_pt(&self, page: u32) -> f64 {
        self.doc
            .page_size_pt(page as usize)
            .map(|(w, _)| w)
            .unwrap_or(0.0)
    }

    pub fn page_height_pt(&self, page: u32) -> f64 {
        self.doc
            .page_size_pt(page as usize)
            .map(|(_, h)| h)
            .unwrap_or(0.0)
    }

    /// Rasterize one lock page. Does not recompile.
    pub fn render_page(&self, page: u32, scale: f32) -> Result<Vec<u8>, JsValue> {
        self.doc
            .render_page(page as usize, scale)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn hit_test(&self, page: u32, x_pt: f64, y_pt: f64) -> Option<String> {
        let hit = self.doc.hit_test(page as usize, milli(x_pt), milli(y_pt))?;
        serde_json::to_string(&hit).ok()
    }

    pub fn hit_selection(&self, page: u32, x_pt: f64, y_pt: f64) -> Option<String> {
        let hit = self.doc.hit_test(page as usize, milli(x_pt), milli(y_pt))?;
        let sel = self.doc.selection_from_hit(&hit)?;
        serde_json::to_string(&sel).ok()
    }

    pub fn selection(&self, id: &str) -> Option<String> {
        let sel = self.doc.selection(id)?;
        serde_json::to_string(&sel).ok()
    }

    pub fn clipboard(&self, id: &str) -> Option<String> {
        let clip = self.doc.clipboard(id)?;
        serde_json::to_string(&clip).ok()
    }

    pub fn search(&self, query: &str) -> String {
        serde_json::to_string(&self.doc.search(query)).unwrap_or_else(|_| "[]".into())
    }

    pub fn boxes_for(&self, id: &str) -> String {
        serde_json::to_string(&self.doc.boxes_for(id)).unwrap_or_else(|_| "[]".into())
    }

    pub fn text_layer(&self, page: u32) -> String {
        serde_json::to_string(&self.doc.text_layer(page as usize)).unwrap_or_else(|_| "[]".into())
    }

    /// `ranges_json`: `[{node_id, char_start, char_end, ...}]` from the text-layer copy path.
    pub fn selection_markdown(&self, ranges_json: &str) -> Result<String, JsValue> {
        self.doc
            .selection_markdown_json(ranges_json)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Full-document Markdown (no k2f hint comments).
    pub fn document_markdown(&self) -> Result<String, JsValue> {
        self.doc
            .document_markdown()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// All pages as PNG (single file) or zip of `page-N.png`.
    pub fn export_pages_png_zip(&self, scale: f32) -> Result<Vec<u8>, JsValue> {
        self.doc
            .export_pages_png(scale)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// All pages as JPEG (single file) or zip of `page-N.jpg`.
    pub fn export_pages_jpeg_zip(&self, scale: f32) -> Result<Vec<u8>, JsValue> {
        self.doc
            .export_pages_jpeg(scale)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Draw the published lock into a PDF. Not a second layout engine.
    pub fn export_pdf(&self) -> Result<Vec<u8>, JsValue> {
        self.export_pdf_at(k2f_pdf::PdfScale::DEFAULT.as_f32())
    }

    pub fn export_pdf_at(&self, scale: f32) -> Result<Vec<u8>, JsValue> {
        let scale = k2f_pdf::PdfScale::from_f32(scale)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        k2f_pdf::export_opened(&self.doc, scale).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

fn milli(pt: f64) -> i64 {
    (pt * 1000.0).round() as i64
}
