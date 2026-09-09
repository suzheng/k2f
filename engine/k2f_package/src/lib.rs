mod error;
mod fonts;
mod includes;
mod integrity;
mod load_dir;
mod manifest;
mod pack;
mod package;
mod parse;
pub mod paths;
mod schema;
mod schema_error;
mod sign;
mod tables;
mod unpack;
mod verify;
mod write_dir;
mod zip_store;

pub use error::{
    PackageError, VerifyStatus, CODE_APPEARANCE_CHANGED, CODE_CONTENT_CHANGED,
    CODE_ENGINE_MISMATCH, CODE_FONT_MISSING, CODE_IMAGE_SIZE, CODE_NODE_ID,
    CODE_PDF_IS_NOT_A_SOURCE, CODE_SCHEMA_INVALID, CODE_SIGNED, CODE_SIGNED_BUT_BROKEN,
    CODE_UNEXPECTED_PATH, CODE_UNKNOWN_PAINT_OP, CODE_UNLOCKED, CODE_UNSIGNED, CODE_VALID,
};
pub use fonts::apply_coverage_subset;
pub use integrity::{inspect_package, IntegrityReport, IntegrityStatus};
pub use load_dir::{load_dir, load_from_file_map};
pub use manifest::PackageManifest;
pub use pack::{pack_bytes, validate_package_schema};
pub use package::Package;
pub use schema::{bundled_schema_files, validate_theme_json};
pub use sign::{generate_secret_key, sign_package, utc_unix_seconds, SecretKey};
pub use unpack::unpack_bytes;
pub use verify::{appearance_hash_for_lock, verify_package};
pub use write_dir::{write_dir, WriteDirOpts};
pub use zip_store::write_zip;

use k2f_core::Manifest;
use std::collections::BTreeMap;

pub fn package_from_engine(
    engine: Manifest,
    theme_json: String,
    font_path: &str,
    font_bytes: Vec<u8>,
) -> Result<Package, PackageError> {
    let title = engine.title.clone();
    let mut fonts = BTreeMap::new();
    fonts.insert(font_path.to_string(), font_bytes);
    Package::from_engine_parts(
        title,
        None,
        Some(0),
        engine,
        theme_json,
        fonts,
        BTreeMap::new(),
    )
}
