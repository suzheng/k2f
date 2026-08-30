use crate::error::PackageError;
use crate::package::Package;
use crate::parse::parse_package_content;
use crate::paths;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn load_dir(dir: &Path) -> Result<Package, PackageError> {
    validate_assets_tree(dir)?;
    validate_content_tree(dir)?;
    let manifest_raw = fs::read_to_string(dir.join(paths::MANIFEST))?;
    let root_raw = fs::read_to_string(dir.join(paths::ROOT))?;
    let theme_json = fs::read_to_string(dir.join(paths::THEME))?;
    let content_files = read_content_json_files(dir)?;

    let parsed = parse_package_content(&manifest_raw, &root_raw, &theme_json, &content_files)?;

    let fonts = read_dir_files(
        dir.join(paths::FONTS_DIR.trim_end_matches('/')),
        paths::FONTS_DIR,
    )?;
    let mut assets = read_dir_files(
        dir.join(paths::IMAGES_DIR.trim_end_matches('/')),
        paths::IMAGES_DIR,
    )?;
    assets.extend(read_dir_files(
        dir.join(paths::DATA_DIR.trim_end_matches('/')),
        paths::DATA_DIR,
    )?);

    let mut schemas = BTreeMap::new();
    for (path, text) in crate::schema::bundled_schema_files() {
        schemas.insert(path.to_string(), text.to_string());
    }

    let changelog_json = match fs::read_to_string(dir.join(paths::CHANGELOG)) {
        Ok(s) => s,
        Err(_) => paths::EMPTY_CHANGELOG.to_string(),
    };
    let tokens_json = fs::read_to_string(dir.join(paths::TOKENS)).ok();
    let lock_json = fs::read_to_string(dir.join(paths::LOCK)).ok();
    let signatures_json = fs::read_to_string(dir.join(paths::SIGNATURES)).ok();

    let mut pkg = Package {
        manifest: parsed.manifest,
        root: parsed.root,
        include_map: parsed.include_map,
        theme_json,
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

/// Load an author-source tree from an in-memory path → bytes map (WASM / embedded templates).
pub fn load_from_file_map(files: &BTreeMap<String, Vec<u8>>) -> Result<Package, PackageError> {
    validate_file_map_assets(files)?;
    validate_file_map_content(files)?;
    let manifest_raw = read_map_text(files, paths::MANIFEST)?;
    let root_raw = read_map_text(files, paths::ROOT)?;
    let theme_json = read_map_text(files, paths::THEME)?;
    let content_files = read_map_content_json(files)?;

    let parsed = parse_package_content(&manifest_raw, &root_raw, &theme_json, &content_files)?;

    let fonts = read_map_prefix_files(files, paths::FONTS_DIR)?;
    let mut assets = read_map_prefix_files(files, paths::IMAGES_DIR)?;
    assets.extend(read_map_prefix_files(files, paths::DATA_DIR)?);

    let mut schemas = BTreeMap::new();
    for (path, text) in crate::schema::bundled_schema_files() {
        schemas.insert(path.to_string(), text.to_string());
    }

    let changelog_json = read_map_text(files, paths::CHANGELOG)
        .unwrap_or_else(|_| paths::EMPTY_CHANGELOG.to_string());
    let tokens_json = read_map_text(files, paths::TOKENS).ok();
    let lock_json = read_map_text(files, paths::LOCK).ok();
    let signatures_json = read_map_text(files, paths::SIGNATURES).ok();

    let mut pkg = Package {
        manifest: parsed.manifest,
        root: parsed.root,
        include_map: parsed.include_map,
        theme_json,
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

fn read_map_text(files: &BTreeMap<String, Vec<u8>>, path: &str) -> Result<String, PackageError> {
    let bytes = files
        .get(path)
        .ok_or_else(|| PackageError::Other(format!("missing {path}")))?;
    String::from_utf8(bytes.clone()).map_err(|e| PackageError::Other(e.to_string()))
}

fn read_map_content_json(files: &BTreeMap<String, Vec<u8>>) -> Result<BTreeMap<String, String>, PackageError> {
    let mut out = BTreeMap::new();
    for (path, bytes) in files {
        if !path.starts_with("content/") || !path.ends_with(".json") {
            continue;
        }
        if !paths::is_content_json_path(path) && path.as_str() != paths::ROOT {
            return Err(PackageError::UnexpectedPath(format!("{path}")));
        }
        let text = String::from_utf8(bytes.clone()).map_err(|e| PackageError::Other(e.to_string()))?;
        out.insert(path.clone(), text);
    }
    Ok(out)
}

fn read_map_prefix_files(
    files: &BTreeMap<String, Vec<u8>>,
    prefix: &str,
) -> Result<BTreeMap<String, Vec<u8>>, PackageError> {
    let mut out = BTreeMap::new();
    for (path, bytes) in files {
        if !path.starts_with(prefix) || path.len() <= prefix.len() {
            continue;
        }
        let rel = &path[prefix.len()..];
        if rel.is_empty() || rel.ends_with('/') {
            continue;
        }
        out.insert(path.clone(), bytes.clone());
    }
    Ok(out)
}

fn validate_file_map_content(files: &BTreeMap<String, Vec<u8>>) -> Result<(), PackageError> {
    for path in files.keys() {
        if path.starts_with("content/") && path.ends_with(".json") {
            continue;
        }
        if path == "content" || path.starts_with("content/") {
            return Err(PackageError::UnexpectedPath(format!(
                "{path} (only .json files are allowed under content/)"
            )));
        }
    }
    Ok(())
}

fn validate_file_map_assets(files: &BTreeMap<String, Vec<u8>>) -> Result<(), PackageError> {
    for path in files.keys() {
        if !path.starts_with("assets/") {
            continue;
        }
        if paths::is_font_path(path)
            || paths::is_extra_asset_path(path)
            || path == "assets/fonts"
            || path == "assets/images"
            || path == "assets/data"
            || path.starts_with("assets/fonts/")
            || path.starts_with("assets/images/")
            || path.starts_with("assets/data/")
        {
            continue;
        }
        let rest = path.strip_prefix("assets/").unwrap_or(path);
        let top = rest.split('/').next().unwrap_or_default();
        return Err(PackageError::UnexpectedPath(format!(
            "assets/{top}/ (only fonts/, images/, and data/ are allowed)"
        )));
    }
    Ok(())
}

fn read_content_json_files(dir: &Path) -> Result<BTreeMap<String, String>, PackageError> {
    let content_dir = dir.join("content");
    let mut out = BTreeMap::new();
    if !content_dir.exists() {
        return Ok(out);
    }
    read_content_json_rec(dir, &content_dir, &mut out)?;
    Ok(out)
}

fn read_content_json_rec(
    package_dir: &Path,
    dir: &Path,
    out: &mut BTreeMap<String, String>,
) -> Result<(), PackageError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            read_content_json_rec(package_dir, &path, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(package_dir)
                .map_err(|e| PackageError::Other(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            if !rel.ends_with(".json") {
                return Err(PackageError::UnexpectedPath(format!(
                    "{rel} (only .json files are allowed under content/)"
                )));
            }
            if !paths::is_content_json_path(&rel) && rel != paths::ROOT {
                return Err(PackageError::UnexpectedPath(format!("{rel}")));
            }
            out.insert(rel, fs::read_to_string(&path)?);
        }
    }
    Ok(())
}

/// Reject non-json files under content/.
pub fn validate_content_tree(package_dir: &Path) -> Result<(), PackageError> {
    let content_dir = package_dir.join("content");
    if !content_dir.exists() {
        return Ok(());
    }
    validate_content_entries(package_dir, &content_dir)
}

fn validate_content_entries(package_dir: &Path, dir: &Path) -> Result<(), PackageError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            validate_content_entries(package_dir, &path)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(package_dir)
                .map_err(|e| PackageError::Other(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            if !rel.ends_with(".json") {
                return Err(PackageError::UnexpectedPath(format!(
                    "{rel} (only .json files are allowed under content/)"
                )));
            }
        }
    }
    Ok(())
}

fn read_dir_files(
    dir: impl AsRef<Path>,
    prefix: &str,
) -> Result<BTreeMap<String, Vec<u8>>, PackageError> {
    let dir = dir.as_ref();
    let mut out = BTreeMap::new();
    if !dir.exists() {
        return Ok(out);
    }
    read_dir_files_rec(dir, dir, prefix, &mut out)?;
    Ok(out)
}

fn read_dir_files_rec(
    root: &Path,
    dir: &Path,
    prefix: &str,
    out: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), PackageError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            read_dir_files_rec(root, &path, prefix, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| PackageError::Other(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(format!("{prefix}{rel}"), fs::read(&path)?);
        }
    }
    Ok(())
}

/// Reject asset files outside `assets/fonts/`, `assets/images/`, and `assets/data/`.
pub fn validate_assets_tree(package_dir: &Path) -> Result<(), PackageError> {
    let assets_dir = package_dir.join("assets");
    if !assets_dir.exists() {
        return Ok(());
    }
    validate_assets_entries(&assets_dir, &assets_dir)
}

fn validate_assets_entries(assets_root: &Path, dir: &Path) -> Result<(), PackageError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if dir == assets_root {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if name != "fonts" && name != "images" && name != "data" {
                    return Err(PackageError::UnexpectedPath(format!(
                        "assets/{name}/ (only fonts/, images/, and data/ are allowed)"
                    )));
                }
            }
            validate_assets_entries(assets_root, &path)?;
        } else if path.is_file() && dir == assets_root {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            return Err(PackageError::UnexpectedPath(format!(
                "assets/{name} (place files under assets/fonts/, assets/images/, or assets/data/)"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CODE_UNEXPECTED_PATH;
    use std::fs;

    #[test]
    fn rejects_assets_outside_allowed_subdirs() {
        let base = std::env::temp_dir().join(format!(
            "k2f_pack_assets_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("assets/fonts")).unwrap();
        fs::write(base.join("assets/logo.png"), b"fake").unwrap();

        let err = validate_assets_tree(&base).unwrap_err();
        assert!(
            err.to_string().contains(CODE_UNEXPECTED_PATH),
            "got {err}"
        );
        let _ = fs::remove_dir_all(&base);
    }
}
