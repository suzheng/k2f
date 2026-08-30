use crate::error::PackageError;
use crate::package::Package;
use crate::paths;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default)]
pub struct WriteDirOpts {
    /// Include `document.K2F.lock` when present (product round-trip).
    pub include_lock: bool,
    /// Include embedded `schema/*.json` (author sources omit; `pack` injects).
    pub include_schema: bool,
}

pub fn write_dir(
    package: &Package,
    dest: &Path,
    opts: WriteDirOpts,
) -> Result<(), PackageError> {
    if dest.exists() {
        let mut entries = fs::read_dir(dest)?;
        if entries.next().is_some() {
            return Err(PackageError::Other(format!(
                "destination is not empty: {}",
                dest.display()
            )));
        }
    } else {
        fs::create_dir_all(dest)?;
    }

    let files = package.to_file_map()?;
    for (rel, bytes) in files {
        if !opts.include_lock && rel == paths::LOCK {
            continue;
        }
        if !opts.include_schema && paths::is_schema_path(&rel) {
            continue;
        }
        let path = dest.join(&rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PackageError::Other(format!("mkdir: {e}")))?;
        }
        fs::write(&path, bytes).map_err(|e| PackageError::Other(format!("write {rel}: {e}")))?;
    }

    // Ensure standard asset dirs exist for author workflows.
    for sub in ["assets/images", "assets/data"] {
        let d = dest.join(sub);
        if !d.exists() {
            fs::create_dir_all(&d).map_err(|e| PackageError::Other(format!("mkdir: {e}")))?;
        }
    }
    Ok(())
}
