use crate::ir::{collect_pictures, PictureBox};
use crate::xml::escape_xml;
use std::collections::{BTreeMap, BTreeSet};

pub fn picture_rids(pics: &[&PictureBox]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut i = 1u32;
    for p in pics {
        map.entry(p.media_name.clone()).or_insert_with(|| {
            let rid = format!("rIdM{i}");
            i = i.saturating_add(1);
            rid
        });
    }
    map
}

pub fn collect_page_pictures(pages: &[crate::ir::PageIR]) -> Vec<&PictureBox> {
    let mut out = Vec::new();
    for p in pages {
        out.extend(collect_pictures(&p.elements));
    }
    out
}

pub fn media_exts<'a>(pics: impl Iterator<Item = &'a PictureBox>) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for p in pics {
        if let Some((_, ext)) = p.media_name.rsplit_once('.') {
            set.insert(ext.to_ascii_lowercase());
        }
    }
    set
}

pub fn insert_media(files: &mut BTreeMap<String, Vec<u8>>, pics: &[&PictureBox]) {
    for p in pics {
        files
            .entry(format!("word/media/{}", p.media_name))
            .or_insert_with(|| p.bytes.clone());
    }
}

pub fn image_rel_xml(rids: &BTreeMap<String, String>) -> String {
    let mut rows: Vec<_> = rids.iter().collect();
    rows.sort_by_key(|(_, rid)| *rid);
    let mut s = String::new();
    for (name, rid) in rows {
        s.push_str(&format!(
            r#"  <Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/{}"/>
"#,
            escape_xml(name)
        ));
    }
    s
}

pub fn hyperlink_rel_xml(rids: &BTreeMap<String, String>) -> String {
    let mut rows: Vec<_> = rids.iter().collect();
    rows.sort_by_key(|(_, rid)| *rid);
    let mut s = String::new();
    for (url, rid) in rows {
        s.push_str(&format!(
            r#"  <Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{}" TargetMode="External"/>
"#,
            escape_xml(url)
        ));
    }
    s
}

pub fn part_rels_xml(
    hyperlink_rids: &BTreeMap<String, String>,
    picture_rids: &BTreeMap<String, String>,
) -> Option<String> {
    let inner = format!(
        "{}{}",
        image_rel_xml(picture_rids),
        hyperlink_rel_xml(hyperlink_rids)
    );
    if inner.is_empty() {
        None
    } else {
        Some(rels_part(&inner))
    }
}

pub fn rels_part(inner: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
{inner}</Relationships>
"#
    )
}

pub fn media_default_xml(exts: &BTreeSet<String>) -> String {
    let mut s = String::new();
    for ext in exts {
        if let Some(ct) = media_content_type(ext) {
            s.push_str(&format!(
                r#"  <Default Extension="{ext}" ContentType="{ct}"/>
"#
            ));
        }
    }
    s
}

fn media_content_type(ext: &str) -> Option<&'static str> {
    match ext {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}
