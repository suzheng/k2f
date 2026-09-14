use super::graphic;
use super::skeleton;
use super::spread;
use crate::coord::SpreadSpace;
use crate::IdmlError;
use k2f_core::LockFile;
use k2f_paint::OpenedDocument;
use std::collections::{BTreeMap, BTreeSet};

pub fn build_package(
    doc: &OpenedDocument,
    lock: &LockFile,
) -> Result<BTreeMap<String, Vec<u8>>, IdmlError> {
    let n = lock.geometry.pages.len();
    if n == 0 {
        return Err(IdmlError::Write("package has no pages".into()));
    }
    let page0 = &lock.geometry.pages[0];
    let space = SpreadSpace::new(page0.width, page0.height);
    let mut fills = Vec::with_capacity(n);
    let mut swatches = BTreeSet::new();
    for (i, page) in lock.geometry.pages.iter().enumerate() {
        let ops = lock
            .render_plan
            .pages
            .get(i)
            .map(|p| p.ops.as_slice())
            .unwrap_or(&[]);
        let hex = spread::page_fill_hex(page, ops)?;
        if let Some(ref h) = hex {
            swatches.insert(h.clone());
        }
        fills.push(hex);
    }
    let mut files = BTreeMap::new();
    files.insert(
        "designmap.xml".into(),
        skeleton::designmap_xml(n).into_bytes(),
    );
    files.insert(
        "META-INF/container.xml".into(),
        skeleton::CONTAINER_XML.as_bytes().to_vec(),
    );
    files.insert(
        "META-INF/metadata.xml".into(),
        skeleton::metadata_xml(doc.title()).into_bytes(),
    );
    files.insert(
        "Resources/Fonts.xml".into(),
        graphic::fonts_xml(doc.fonts()).into_bytes(),
    );
    files.insert(
        "Resources/Graphic.xml".into(),
        graphic::graphic_xml(&swatches).into_bytes(),
    );
    files.insert(
        "Resources/Preferences.xml".into(),
        skeleton::preferences_xml(&space).into_bytes(),
    );
    files.insert(
        "Resources/Styles.xml".into(),
        skeleton::STYLES_XML.as_bytes().to_vec(),
    );
    files.insert(
        "MasterSpreads/MasterSpread_kMaster.xml".into(),
        skeleton::master_xml(&space).into_bytes(),
    );
    files.insert(
        "XML/BackingStory.xml".into(),
        skeleton::BACKING_XML.as_bytes().to_vec(),
    );
    files.insert(
        "XML/Tags.xml".into(),
        skeleton::TAGS_XML.as_bytes().to_vec(),
    );
    for (i, fill) in fills.iter().enumerate() {
        files.insert(
            format!("Spreads/Spread_k{i}.xml"),
            spread::spread_xml(i, &space, fill.as_deref()).into_bytes(),
        );
    }
    Ok(files)
}
