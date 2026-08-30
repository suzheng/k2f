//! Resolve official template directories and copy author sources.
use crate::error::{AgentError, INVALID_ARGUMENT};
use k2f_package::{load_dir, Package};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_arch = "wasm32")]
use std::collections::BTreeMap;

pub const OFFICIAL_IDS: &[&str] = &["blank", "invoice", "legal", "clinical_summary", "report"];

pub fn bundled_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates")
}

pub fn resolve(name_or_path: impl AsRef<Path>) -> Result<PathBuf, AgentError> {
    let input = name_or_path.as_ref();
    if input.is_dir() {
        return Ok(input.to_path_buf());
    }
    let name = input
        .to_str()
        .ok_or_else(|| AgentError::new(INVALID_ARGUMENT, "template path must be utf-8"))?;
    if let Ok(root) = std::env::var("K2F_TEMPLATES") {
        let candidate = PathBuf::from(&root).join(name);
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    let candidate = bundled_root().join(name);
    if candidate.is_dir() {
        return Ok(candidate);
    }
    #[cfg(target_arch = "wasm32")]
    if OFFICIAL_IDS.contains(&name) {
        return Ok(PathBuf::from(name));
    }
    Err(AgentError::new(
        INVALID_ARGUMENT,
        format!(
            "unknown template '{name}' (set K2F_TEMPLATES or use a directory path; official: {})",
            OFFICIAL_IDS.join(", ")
        ),
    ))
}

/// Load an official template or author directory into a package.
pub fn load_package(name_or_path: impl AsRef<Path>) -> Result<Package, AgentError> {
    let input = name_or_path.as_ref();
    if input.is_dir() {
        return load_dir(input).map_err(AgentError::from);
    }
    let name = input
        .to_str()
        .ok_or_else(|| AgentError::new(INVALID_ARGUMENT, "template path must be utf-8"))?;
    if let Ok(root) = std::env::var("K2F_TEMPLATES") {
        let candidate = PathBuf::from(&root).join(name);
        if candidate.is_dir() {
            return load_dir(&candidate).map_err(AgentError::from);
        }
    }
    let candidate = bundled_root().join(name);
    if candidate.is_dir() {
        return load_dir(&candidate).map_err(AgentError::from);
    }
    #[cfg(target_arch = "wasm32")]
    if OFFICIAL_IDS.contains(&name) {
        let files = embedded_template_files(name)?;
        return k2f_package::load_from_file_map(&files).map_err(AgentError::from);
    }
    Err(AgentError::new(
        INVALID_ARGUMENT,
        format!(
            "unknown template '{name}' (set K2F_TEMPLATES or use a directory path; official: {})",
            OFFICIAL_IDS.join(", ")
        ),
    ))
}

#[cfg(target_arch = "wasm32")]
mod embed {
    use include_dir::{include_dir, Dir};

    pub static TEMPLATES: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../templates");
}

#[cfg(target_arch = "wasm32")]
fn embedded_template_files(name: &str) -> Result<BTreeMap<String, Vec<u8>>, AgentError> {
    let dir = embed::TEMPLATES.get_dir(name).ok_or_else(|| {
        AgentError::new(
            INVALID_ARGUMENT,
            format!("embedded template '{name}' is missing"),
        )
    })?;
    let mut files = BTreeMap::new();
    collect_embedded_dir(dir, &mut files);
    let strip = format!("{name}/");
    Ok(files
        .into_iter()
        .map(|(path, bytes)| {
            let path = path
                .strip_prefix(&strip)
                .map(str::to_string)
                .unwrap_or(path);
            (path, bytes)
        })
        .collect())
}

#[cfg(target_arch = "wasm32")]
fn collect_embedded_dir(dir: &include_dir::Dir<'_>, out: &mut BTreeMap<String, Vec<u8>>) {
    for file in dir.files() {
        let path = file.path().to_string_lossy().replace('\\', "/");
        out.insert(path, file.contents().to_vec());
    }
    for sub in dir.dirs() {
        collect_embedded_dir(sub, out);
    }
}

pub fn copy_to(name_or_path: impl AsRef<Path>, dest: &Path) -> Result<(), AgentError> {
    let src = resolve(name_or_path)?;
    if dest.exists() {
        return Err(AgentError::new(
            INVALID_ARGUMENT,
            format!("destination already exists: {}", dest.display()),
        ));
    }
    copy_dir_all(&src, dest).map_err(|e| {
        AgentError::new(
            INVALID_ARGUMENT,
            format!("copy template {} → {}: {e}", src.display(), dest.display()),
        )
    })
}

fn copy_dir_all(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
