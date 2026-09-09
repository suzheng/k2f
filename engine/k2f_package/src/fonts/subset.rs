use crate::error::PackageError;
use crate::fonts::keep::keep_codepoint;
use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::subset::{subset, CmapTarget, SubsetProfile};
use std::collections::BTreeSet;
use ttf_parser::Face;

const SHAPING_TABLES: &str = "gpos,gsub,gdef,cvt,fpgm,prep";

pub fn coverage_subset_face(path: &str, bytes: &[u8]) -> Result<Vec<u8>, PackageError> {
    let face = Face::parse(bytes, 0).map_err(|e| {
        PackageError::Other(format!("FONT_INVALID: {path}: {e}"))
    })?;
    let Some(gids) = keep_glyph_ids(&face) else {
        return Ok(bytes.to_vec());
    };
    subset_with_gids(path, bytes, &gids)
}

fn keep_glyph_ids(face: &Face<'_>) -> Option<Vec<u16>> {
    let cmap = face.tables().cmap?;
    let mut gids = BTreeSet::new();
    gids.insert(0);
    let mut drop_han = false;
    for subtable in cmap.subtables.into_iter() {
        if !subtable.is_unicode() {
            continue;
        }
        subtable.codepoints(|cp| {
            let Some(gid) = subtable.glyph_index(cp) else {
                return;
            };
            if keep_codepoint(cp) {
                gids.insert(gid.0);
            } else {
                drop_han = true;
            }
        });
    }
    if !drop_han {
        return None;
    }
    Some(gids.into_iter().collect())
}

fn subset_with_gids(path: &str, bytes: &[u8], gids: &[u16]) -> Result<Vec<u8>, PackageError> {
    let font = ReadScope::new(bytes)
        .read::<FontData<'_>>()
        .map_err(|e| PackageError::Other(format!("font subset {path}: {e}")))?;
    let provider = font
        .table_provider(0)
        .map_err(|e| PackageError::Other(format!("font subset {path}: {e}")))?;
    let profile = SubsetProfile::parse_custom(SHAPING_TABLES.into())
        .map_err(|e| PackageError::Other(format!("font subset {path}: {e}")))?;
    subset(&provider, gids, &profile, CmapTarget::Unicode)
        .map_err(|e| PackageError::Other(format!("font subset {path}: {e}")))
}
