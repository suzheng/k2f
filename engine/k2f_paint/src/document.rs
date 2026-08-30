use k2f_core::{
    expand_manifest_tables_with_assets, AssetsMap, LockFile, RunningBlockNode, SemanticNode,
};
use k2f_package::{inspect_package, unpack_bytes, IntegrityReport, Package, VerifyStatus};

use crate::banner::Banner;
use crate::error::PaintError;
use crate::executor::{render_lockfile_page_rgb, render_lockfile_page_to_png};

/// Official pixel comparison uses this raster scale. Viewer CSS zoom is UI-only.
pub const OFFICIAL_PNG_SCALE: f32 = 2.0;

/// Opened `.K2F`: same inspect as CLI verify, paint the lock only. Never recompile.
pub struct OpenedDocument {
    pub(crate) package: Package,
    pub(crate) lock: Option<LockFile>,
    pub(crate) query_root: SemanticNode,
    pub(crate) query_running: Vec<RunningBlockNode>,
    integrity: IntegrityReport,
}

impl OpenedDocument {
    pub fn open(bytes: &[u8]) -> Result<Self, PaintError> {
        Self::from_package(unpack_bytes(bytes)?)
    }

    pub fn from_package(package: Package) -> Result<Self, PaintError> {
        let integrity = inspect_package(&package)?;
        let lock = package
            .lock_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|e| PaintError::Package(format!("lock: {e}")))?;
        let (query_root, query_running) = query_trees(&package);
        Ok(Self {
            package,
            lock,
            query_root,
            query_running,
            integrity,
        })
    }

    pub fn banner(&self) -> Banner {
        Banner::from_integrity(self.integrity.status)
    }

    pub fn status(&self) -> VerifyStatus {
        self.integrity.hash
    }

    pub fn status_code(&self) -> &'static str {
        self.integrity.status.code()
    }

    pub fn hash_code(&self) -> &'static str {
        self.integrity.hash.code()
    }

    pub fn fingerprint(&self) -> Option<&str> {
        self.integrity.fingerprint.as_deref()
    }

    pub fn signed_by(&self) -> Option<&str> {
        self.integrity.signed_by.as_deref()
    }

    pub fn signed_at(&self) -> Option<i64> {
        self.integrity.signed_at
    }

    pub fn generated_by(&self) -> Option<&str> {
        self.package.manifest.generated_by.as_deref()
    }

    pub fn page_count(&self) -> usize {
        self.lock
            .as_ref()
            .map(|l| l.geometry.pages.len())
            .unwrap_or(0)
    }

    pub fn page_size_pt(&self, page_idx: usize) -> Option<(f64, f64)> {
        let page = self.lock.as_ref()?.geometry.pages.get(page_idx)?;
        Some((page.width.as_f64_pt(), page.height.as_f64_pt()))
    }

    pub fn render_page(&self, page_idx: usize, scale: f32) -> Result<Vec<u8>, PaintError> {
        let lock = self.lock.as_ref().ok_or(PaintError::Unlocked)?;
        if self.package.fonts.is_empty() {
            return Err(PaintError::Font("package has no embedded font".into()));
        }
        render_lockfile_page_to_png(
            lock,
            page_idx,
            scale,
            &self.package.fonts,
            &self.package.assets,
        )
    }

    pub fn render_page_rgb(
        &self,
        page_idx: usize,
        scale: f32,
    ) -> Result<(u32, u32, Vec<u8>), PaintError> {
        let lock = self.lock.as_ref().ok_or(PaintError::Unlocked)?;
        if self.package.fonts.is_empty() {
            return Err(PaintError::Font("package has no embedded font".into()));
        }
        render_lockfile_page_rgb(
            lock,
            page_idx,
            scale,
            &self.package.fonts,
            &self.package.assets,
        )
    }

    pub fn lock(&self) -> Option<&LockFile> {
        self.lock.as_ref()
    }

    pub fn fonts(&self) -> &std::collections::BTreeMap<String, Vec<u8>> {
        &self.package.fonts
    }

    pub fn assets(&self) -> &std::collections::BTreeMap<String, Vec<u8>> {
        &self.package.assets
    }

    pub fn appearance_hash(&self) -> Option<&str> {
        self.lock.as_ref().map(|l| l.appearance_hash.as_str())
    }

    pub fn content_hash(&self) -> Option<&str> {
        self.lock.as_ref().map(|l| l.content_hash.as_str())
    }

    pub fn title(&self) -> &str {
        &self.package.manifest.title
    }

    pub fn semantic_root(&self) -> &SemanticNode {
        &self.query_root
    }

    pub fn running_blocks(&self) -> &[RunningBlockNode] {
        &self.query_running
    }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn document_markdown(&self) -> Result<String, PaintError> {
        crate::export::document_markdown(self)
    }

    pub fn export_pages_png(&self, scale: f32) -> Result<Vec<u8>, PaintError> {
        crate::export::export_pages_png(self, scale)
    }

    pub fn export_pages_jpeg(&self, scale: f32) -> Result<Vec<u8>, PaintError> {
        crate::export::export_pages_jpeg(self, scale)
    }
}

fn query_trees(package: &Package) -> (SemanticNode, Vec<RunningBlockNode>) {
    let assets: AssetsMap = package.assets.clone().into_iter().collect();
    match expand_manifest_tables_with_assets(&package.engine_manifest(), &assets) {
        Ok(m) => (m.root, m.running_blocks),
        Err(_) => (
            package.root.clone(),
            package.manifest.running_blocks.clone(),
        ),
    }
}
