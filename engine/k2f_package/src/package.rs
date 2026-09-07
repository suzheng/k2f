use crate::error::PackageError;
use crate::includes::collapse_content_tree;
use crate::manifest::PackageManifest;
use crate::paths;
use crate::schema::validate_content_json;
use k2f_core::{
    canonical_json_string, engine_version, normalize_manifest_nfc, validate_manifest_node_ids,
    LockFile, Manifest, SemanticNode,
};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Package {
    pub manifest: PackageManifest,
    pub root: SemanticNode,
    /// Expanded semantic tree entry id → on-disk include file path.
    pub include_map: BTreeMap<String, String>,
    pub theme_json: String,
    pub fonts: BTreeMap<String, Vec<u8>>,
    pub assets: BTreeMap<String, Vec<u8>>,
    pub schemas: BTreeMap<String, String>,
    pub changelog_json: String,
    pub tokens_json: Option<String>,
    pub lock_json: Option<String>,
    pub signatures_json: Option<String>,
}

impl Package {
    pub fn engine_manifest(&self) -> Manifest {
        Manifest {
            title: self.manifest.title.clone(),
            canvas_mode: self.manifest.canvas_mode,
            page_config: self.manifest.page_config.clone(),
            root: self.root.clone(),
            running_blocks: self.manifest.running_blocks.clone(),
        }
    }

    pub fn finalize(&mut self) -> Result<(), PackageError> {
        let mut engine = self.engine_manifest();
        normalize_manifest_nfc(&mut engine);
        validate_manifest_node_ids(&engine).map_err(|e| PackageError::NodeId(e.to_string()))?;
        self.root = engine.root;
        self.manifest.running_blocks = engine.running_blocks;
        if !paths::has_font_face(&self.fonts) {
            return Err(PackageError::FontMissing(
                "at least one font is required under assets/fonts/".to_string(),
            ));
        }
        k2f_core::validate_svg_assets(&self.assets).map_err(|e| {
            let rest = e.strip_prefix("IMAGE_SIZE: ").unwrap_or(e.as_str());
            PackageError::ImageSize(rest.to_string())
        })?;
        Ok(())
    }

    pub fn set_lock(&mut self, lock: &LockFile) -> Result<(), PackageError> {
        self.manifest.engine_version = lock.engine_version.clone();
        self.lock_json =
            Some(canonical_json_string(lock).map_err(|e| PackageError::Other(e.to_string()))?);
        Ok(())
    }

    pub fn from_engine_parts(
        title: String,
        author: Option<String>,
        created_at: Option<i64>,
        engine: Manifest,
        theme_json: String,
        fonts: BTreeMap<String, Vec<u8>>,
        assets: BTreeMap<String, Vec<u8>>,
    ) -> Result<Self, PackageError> {
        if engine.canvas_mode != k2f_core::CanvasMode::Paged {
            return Err(PackageError::Other(
                "formal documents require canvas_mode=paged".to_string(),
            ));
        }
        let mut schemas = BTreeMap::new();
        for (path, text) in crate::schema::bundled_schema_files() {
            schemas.insert(path.to_string(), text.to_string());
        }
        let mut pkg = Self {
            manifest: PackageManifest {
                title,
                author,
                created_at,
                canvas_mode: engine.canvas_mode,
                page_config: engine.page_config,
                engine_version: engine_version().to_string(),
                generated_by: None,
                running_blocks: engine.running_blocks,
            },
            root: engine.root,
            include_map: BTreeMap::new(),
            theme_json,
            fonts,
            assets,
            schemas,
            changelog_json: paths::EMPTY_CHANGELOG.to_string(),
            tokens_json: None,
            lock_json: None,
            signatures_json: None,
        };
        pkg.finalize()?;
        Ok(pkg)
    }

    pub fn to_file_map(&self) -> Result<BTreeMap<String, Vec<u8>>, PackageError> {
        let mut files = BTreeMap::new();
        files.insert(
            paths::MANIFEST.to_string(),
            canonical_bytes(&self.manifest)?,
        );

        if self.include_map.is_empty() {
            files.insert(paths::ROOT.to_string(), canonical_bytes(&self.root)?);
        } else {
            let collapsed = collapse_content_tree(&self.root, &self.include_map)?;
            for (path, text) in &collapsed.files {
                let v: Value = serde_json::from_str(text)
                    .map_err(|e| PackageError::Other(format!("{path}: {e}")))?;
                validate_content_json(&v, path)?;
                files.insert(path.clone(), text.as_bytes().to_vec());
            }
        }

        files.insert(
            paths::THEME.to_string(),
            canonical_json_text(&self.theme_json, "theme")?,
        );
        files.insert(
            paths::CHANGELOG.to_string(),
            canonical_json_text(&self.changelog_json, "changelog")?,
        );
        if let Some(tokens) = &self.tokens_json {
            files.insert(
                paths::TOKENS.to_string(),
                canonical_json_text(tokens, "tokens")?,
            );
        }
        for (path, text) in &self.schemas {
            files.insert(path.clone(), canonical_json_text(text, path)?);
        }
        for (path, bytes) in &self.fonts {
            files.insert(path.clone(), bytes.clone());
        }
        for (path, bytes) in &self.assets {
            files.insert(path.clone(), bytes.clone());
        }
        if let Some(lock) = &self.lock_json {
            files.insert(paths::LOCK.to_string(), canonical_json_text(lock, "lock")?);
        }
        if let Some(sig) = &self.signatures_json {
            files.insert(
                paths::SIGNATURES.to_string(),
                canonical_json_text(sig, "signatures")?,
            );
        }
        Ok(files)
    }
}

fn canonical_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, PackageError> {
    canonical_json_string(value)
        .map(|s| s.into_bytes())
        .map_err(|e| PackageError::Other(e.to_string()))
}

fn canonical_json_text(text: &str, label: &str) -> Result<Vec<u8>, PackageError> {
    let v: Value =
        serde_json::from_str(text).map_err(|e| PackageError::Other(format!("{label}: {e}")))?;
    canonical_json_string(&v)
        .map(|s| s.into_bytes())
        .map_err(|e| PackageError::Other(e.to_string()))
}
