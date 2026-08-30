use crate::error::PackageError;
use crate::package::Package;
use crate::parse::parse_package_content;
use crate::paths::{self, PathKind};
use crate::zip_store::read_zip;
use std::collections::BTreeMap;

pub fn unpack_bytes(bytes: &[u8]) -> Result<Package, PackageError> {
    if bytes.starts_with(b"%PDF") {
        return Err(PackageError::PdfIsNotASource);
    }
    package_from_files(read_zip(bytes)?)
}

fn package_from_files(files: BTreeMap<String, Vec<u8>>) -> Result<Package, PackageError> {
    for path in files.keys() {
        if paths::classify_path(path).is_none() {
            return Err(PackageError::UnexpectedPath(path.clone()));
        }
    }

    let manifest_raw = file_str(&files, paths::MANIFEST)?;
    let root_raw = file_str(&files, paths::ROOT)?;
    let theme_raw = file_str(&files, paths::THEME)?;

    let mut content_files = BTreeMap::new();
    for (path, bytes) in &files {
        if matches!(paths::classify_path(path), Some(PathKind::ContentJson)) {
            content_files.insert(
                path.clone(),
                String::from_utf8(bytes.clone()).map_err(|e| PackageError::Other(e.to_string()))?,
            );
        }
    }

    let parsed = parse_package_content(&manifest_raw, &root_raw, &theme_raw, &content_files)?;

    let mut fonts = BTreeMap::new();
    let mut assets = BTreeMap::new();
    let mut schemas = BTreeMap::new();
    let mut changelog_json = paths::EMPTY_CHANGELOG.to_string();
    let mut tokens_json = None;
    let mut lock_json = None;
    let mut signatures_json = None;

    for (path, bytes) in files {
        match paths::classify_path(&path) {
            Some(PathKind::Font) => {
                fonts.insert(path, bytes);
            }
            Some(PathKind::ExtraAsset) => {
                assets.insert(path, bytes);
            }
            Some(PathKind::Schema) => {
                schemas.insert(
                    path,
                    String::from_utf8(bytes).map_err(|e| PackageError::Other(e.to_string()))?,
                );
            }
            Some(PathKind::Changelog) => {
                changelog_json =
                    String::from_utf8(bytes).map_err(|e| PackageError::Other(e.to_string()))?;
            }
            Some(PathKind::Tokens) => {
                tokens_json =
                    Some(String::from_utf8(bytes).map_err(|e| PackageError::Other(e.to_string()))?);
            }
            Some(PathKind::Lock) => {
                lock_json =
                    Some(String::from_utf8(bytes).map_err(|e| PackageError::Other(e.to_string()))?);
            }
            Some(PathKind::Signatures) => {
                signatures_json =
                    Some(String::from_utf8(bytes).map_err(|e| PackageError::Other(e.to_string()))?);
            }
            Some(PathKind::Manifest | PathKind::Root | PathKind::Theme | PathKind::ContentJson) => {
            }
            None => {}
        }
    }

    let mut pkg = Package {
        manifest: parsed.manifest,
        root: parsed.root,
        include_map: parsed.include_map,
        theme_json: theme_raw,
        fonts,
        assets,
        schemas,
        changelog_json,
        tokens_json,
        lock_json,
        signatures_json,
    };
    pkg.finalize()?;
    Ok(pkg)
}

fn file_str(files: &BTreeMap<String, Vec<u8>>, path: &str) -> Result<String, PackageError> {
    let bytes = files
        .get(path)
        .ok_or_else(|| PackageError::Other(format!("missing {path}")))?;
    String::from_utf8(bytes.clone()).map_err(|e| PackageError::Other(format!("{path}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zip_store::write_zip;

    #[test]
    fn unpack_rejects_pdf_bytes() {
        let err = unpack_bytes(b"%PDF-1.7\n").unwrap_err();
        assert!(
            err.to_string().contains(crate::CODE_PDF_IS_NOT_A_SOURCE),
            "got {err}"
        );
    }

    #[test]
    fn unpack_rejects_unknown_zip_path() {
        let mut files = BTreeMap::new();
        files.insert("evil.txt".to_string(), b"nope".to_vec());
        let bytes = write_zip(&files).unwrap();
        let err = unpack_bytes(&bytes).unwrap_err();
        assert!(
            err.to_string().contains(crate::CODE_UNEXPECTED_PATH),
            "got {err}"
        );
    }

    #[test]
    fn unpack_rejects_non_format_schema_path() {
        let mut files = BTreeMap::new();
        files.insert("schema/agent_v0.schema.json".to_string(), b"{}".to_vec());
        let bytes = write_zip(&files).unwrap();
        let err = unpack_bytes(&bytes).unwrap_err();
        assert!(
            err.to_string().contains(crate::CODE_UNEXPECTED_PATH),
            "got {err}"
        );
    }
}
