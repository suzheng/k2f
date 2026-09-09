mod han;
mod keep;
mod subset;

use crate::error::PackageError;
use crate::paths::is_font_face_path;
use std::collections::BTreeMap;

/// Replace large CJK faces with a Han coverage subset (non-Han glyphs kept).
/// Han keep-set is GB2312 ∪ Big5 level 1 ∪ JIS X 0208. Faces that would not
/// drop any Han are left byte-identical.
pub fn apply_coverage_subset(fonts: &mut BTreeMap<String, Vec<u8>>) -> Result<(), PackageError> {
    for (path, bytes) in fonts.iter_mut() {
        if !is_font_face_path(path) {
            continue;
        }
        *bytes = subset::coverage_subset_face(path, bytes)?;
    }
    Ok(())
}
