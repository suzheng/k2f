use crate::error::{PackageError, VerifyStatus};
use crate::package::Package;
use k2f_core::{
    engine_commit_sha, engine_version, expand_manifest_tables_with_assets, hash_appearance_binding,
    hash_manifest_semantic, normalize_manifest_nfc, AppearanceHashInput, AssetsMap, LockFile,
    Manifest,
};

fn prepared_manifest(package: &Package) -> Result<Manifest, PackageError> {
    let mut manifest = package.engine_manifest();
    if !package.assets.is_empty() {
        let assets: AssetsMap = package.assets.clone().into_iter().collect();
        manifest =
            expand_manifest_tables_with_assets(&manifest, &assets).map_err(PackageError::Other)?;
    }
    normalize_manifest_nfc(&mut manifest);
    Ok(manifest)
}

/// Appearance binding using the lock's recorded engine identity, not the reader's.
pub fn appearance_hash_for_lock(
    package: &Package,
    lock: &LockFile,
) -> Result<String, PackageError> {
    let manifest = prepared_manifest(package)?;
    hash_appearance_binding(&AppearanceHashInput {
        content_hash: &lock.content_hash,
        theme_json: &package.theme_json,
        fonts: &package.fonts,
        page_config: &manifest.page_config,
        engine_version: &lock.engine_version,
        engine_commit_sha: &lock.engine_commit_sha,
    })
    .map_err(PackageError::Other)
}

pub fn verify_package(package: &Package) -> Result<VerifyStatus, PackageError> {
    if !crate::paths::has_font_face(&package.fonts) {
        return Ok(VerifyStatus::FontMissing);
    }
    let Some(lock_json) = &package.lock_json else {
        return Ok(VerifyStatus::Unlocked);
    };

    let lock: LockFile =
        serde_json::from_str(lock_json).map_err(|e| PackageError::Other(format!("lock: {e}")))?;

    let manifest = prepared_manifest(package)?;
    let content_hash = hash_manifest_semantic(&manifest);
    if content_hash != lock.content_hash {
        return Ok(VerifyStatus::ContentChanged);
    }

    let appearance_hash = hash_appearance_binding(&AppearanceHashInput {
        content_hash: &content_hash,
        theme_json: &package.theme_json,
        fonts: &package.fonts,
        page_config: &manifest.page_config,
        engine_version: &lock.engine_version,
        engine_commit_sha: &lock.engine_commit_sha,
    })
    .map_err(PackageError::Other)?;

    if appearance_hash != lock.appearance_hash {
        return Ok(VerifyStatus::AppearanceChanged);
    }

    if lock.engine_version != engine_version() || lock.engine_commit_sha != engine_commit_sha() {
        return Ok(VerifyStatus::EngineMismatch);
    }

    Ok(VerifyStatus::Valid)
}
