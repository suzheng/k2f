use crate::error::PackageError;
use crate::package::Package;
use crate::paths::{self, FORMAT_SCHEMA_FILES};
use crate::schema::{validate_manifest_json, validate_root_json, validate_theme_json};
use crate::zip_store::write_zip;
use serde_json::Value;

pub fn pack_bytes(package: &Package) -> Result<Vec<u8>, PackageError> {
    let mut package = package.clone();
    package.finalize()?;
    validate_package_schema(&package)?;
    write_zip(&package.to_file_map()?)
}

pub fn validate_package_schema(package: &Package) -> Result<(), PackageError> {
    let manifest_v =
        serde_json::to_value(&package.manifest).map_err(|e| PackageError::Other(e.to_string()))?;
    validate_manifest_json(&manifest_v)?;
    let root_v =
        serde_json::to_value(&package.root).map_err(|e| PackageError::Other(e.to_string()))?;
    validate_root_json(&root_v)?;
    let theme_v: Value = serde_json::from_str(&package.theme_json)
        .map_err(|e| PackageError::Other(format!("theme: {e}")))?;
    validate_theme_json(&theme_v)?;
    if let Some(sig) = &package.signatures_json {
        let sig_v: Value = serde_json::from_str(sig)
            .map_err(|e| PackageError::Other(format!("signatures: {e}")))?;
        crate::schema::validate_signatures_json(&sig_v)?;
    }
    for path in package.schemas.keys() {
        if !paths::is_schema_path(path) {
            return Err(PackageError::UnexpectedPath(path.clone()));
        }
    }
    for path in FORMAT_SCHEMA_FILES {
        if !package.schemas.contains_key(*path) {
            return Err(PackageError::Other(format!("missing {path}")));
        }
    }
    if !paths::has_font_face(&package.fonts) {
        return Err(PackageError::FontMissing(
            "at least one font is required under assets/fonts/".to_string(),
        ));
    }
    Ok(())
}
