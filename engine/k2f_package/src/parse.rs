use crate::error::PackageError;
use crate::includes::{collect_orphan_content_paths, resolve_content_files};
use crate::manifest::PackageManifest;
use crate::paths;
use crate::schema::{validate_content_json, validate_manifest_json, validate_theme_json};
use k2f_core::SemanticNode;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub struct ParsedPackage {
    pub manifest: PackageManifest,
    pub root: SemanticNode,
    pub include_map: BTreeMap<String, String>,
}

pub fn parse_package_content(
    manifest_raw: &str,
    root_raw: &str,
    theme_raw: &str,
    content_files: &BTreeMap<String, String>,
) -> Result<ParsedPackage, PackageError> {
    let manifest_v: Value = serde_json::from_str(manifest_raw)
        .map_err(|e| PackageError::Other(format!("manifest: {e}")))?;
    let root_v: Value =
        serde_json::from_str(root_raw).map_err(|e| PackageError::Other(format!("root: {e}")))?;
    let theme_v: Value =
        serde_json::from_str(theme_raw).map_err(|e| PackageError::Other(format!("theme: {e}")))?;

    validate_manifest_json(&manifest_v)?;
    validate_content_json(&root_v, paths::ROOT)?;
    validate_theme_json(&theme_v)?;

    for (path, raw) in content_files {
        if path == paths::ROOT {
            continue;
        }
        let v: Value =
            serde_json::from_str(raw).map_err(|e| PackageError::Other(format!("{path}: {e}")))?;
        validate_content_json(&v, path)?;
    }

    let resolved = resolve_content_files(root_raw, content_files)?;
    reject_orphan_content(content_files, resolved.referenced_paths)?;

    let manifest: PackageManifest = serde_json::from_value(manifest_v)
        .map_err(|e| PackageError::Other(format!("manifest: {e}")))?;
    if manifest.canvas_mode != k2f_core::CanvasMode::Paged {
        return Err(PackageError::Other(
            "formal documents require canvas_mode=paged".to_string(),
        ));
    }

    Ok(ParsedPackage {
        manifest,
        root: resolved.root,
        include_map: resolved.include_map,
    })
}

fn reject_orphan_content(
    content_files: &BTreeMap<String, String>,
    referenced: BTreeSet<String>,
) -> Result<(), PackageError> {
    let on_disk: BTreeSet<String> = content_files.keys().cloned().collect();
    for orphan in collect_orphan_content_paths(&on_disk, &referenced) {
        return Err(PackageError::UnexpectedPath(format!(
            "{orphan} (unreferenced content file; add an include in content/root.json or remove the file)"
        )));
    }
    Ok(())
}
