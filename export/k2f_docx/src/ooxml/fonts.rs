use k2f_core::is_font_face_path;
use k2f_paint::{face_style_is_bold, face_style_is_italic, face_subfamily};
use std::collections::BTreeMap;
use std::path::Path;
use ttf_parser::Face;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EmbedSlot {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl EmbedSlot {
    fn attr(self) -> &'static str {
        match self {
            Self::Regular => "w:embedRegular",
            Self::Bold => "w:embedBold",
            Self::Italic => "w:embedItalic",
            Self::BoldItalic => "w:embedBoldItalic",
        }
    }
}

#[derive(Clone, Debug)]
struct EmbeddedFace {
    family: String,
    slot: EmbedSlot,
    rid: String,
    part_path: String,
    guid: [u8; 16],
    bytes: Vec<u8>,
}

/// Maps lock `font_family` keys to Word font embedding relationships.
#[derive(Clone, Debug, Default)]
pub(crate) struct FontEmbedPlan {
    faces: Vec<EmbeddedFace>,
    key_to_face: BTreeMap<String, usize>,
}

pub(crate) fn collect_package_fonts(fonts: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for (key, bytes) in fonts {
        if key.eq_ignore_ascii_case("default") {
            continue;
        }
        let norm = key.replace('\\', "/");
        if norm
            .split('/')
            .any(|part| part.eq_ignore_ascii_case("licenses"))
        {
            continue;
        }
        if !is_font_face_path(key) {
            continue;
        }
        out.insert(key.clone(), bytes.clone());
    }
    out
}

pub(crate) fn build_font_embed_plan(fonts: &BTreeMap<String, Vec<u8>>) -> FontEmbedPlan {
    let faces = collect_package_fonts(fonts);
    if faces.is_empty() {
        return FontEmbedPlan::default();
    }

    let mut plan = FontEmbedPlan::default();
    let mut sorted: Vec<_> = faces.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    let mut n = 1u32;
    for (path_key, bytes) in sorted {
        let stem = Path::new(path_key)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("K2F Font");
        // Prefer TTF names, then the package stem. A missing unicode FAMILY
        // (Bradley Hand Bold has only Mac FAMILY + Windows FULL_NAME) used
        // to become "K2F Font" while runs still said the stem — hosts then
        // substituted a 27pt serif.
        let family = family_from_bytes(bytes).unwrap_or_else(|| stem.to_string());
        let slot = slot_for_bytes(bytes);
        let guid = font_guid(bytes);
        let rid = format!("rIdEF{n}");
        let part_path = format!("word/fonts/font{n}.odttf");
        n += 1;
        let idx = plan.faces.len();
        plan.faces.push(EmbeddedFace {
            family,
            slot,
            rid,
            part_path,
            guid,
            bytes: obfuscate_font(bytes, &guid),
        });
        plan.key_to_face.insert(path_key.clone(), idx);
        plan.key_to_face.entry(stem.to_string()).or_insert(idx);
    }
    plan
}

impl FontEmbedPlan {
    pub(crate) fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    pub(crate) fn parts(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.faces
            .iter()
            .map(|f| (f.part_path.as_str(), f.bytes.as_slice()))
    }

    /// `(embed attribute, relationship id)` for a lock font key, if embedded.
    pub(crate) fn run_embed(&self, font_family_key: &str) -> Option<(&'static str, &str)> {
        if font_family_key.is_empty() {
            return None;
        }
        let face = self.faces.get(*self.key_to_face.get(font_family_key)?)?;
        Some((face.slot.attr(), face.rid.as_str()))
    }

    /// Bold-only package fonts (Bradley Hand Bold) keep `bold=false` on the
    /// lock run because the weight is the TTF, not a style flag. Hosts still
    /// look up the Bold style of the family, so the run needs `w:b`.
    pub(crate) fn run_needs_b(&self, font_family_key: &str) -> bool {
        let Some(&idx) = self.key_to_face.get(font_family_key) else {
            return false;
        };
        matches!(
            self.faces.get(idx).map(|f| f.slot),
            Some(EmbedSlot::Bold | EmbedSlot::BoldItalic)
        )
    }

    pub(crate) fn font_rels_xml(&self) -> String {
        let mut s = String::new();
        for face in &self.faces {
            let target = face
                .part_path
                .strip_prefix("word/")
                .unwrap_or(&face.part_path);
            s.push_str(&format!(
                r#"  <Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/font" Target="{}"/>
"#,
                face.rid,
                target
            ));
        }
        s
    }

    pub(crate) fn font_table_xml(&self) -> String {
        let mut by_family: BTreeMap<String, Vec<&EmbeddedFace>> = BTreeMap::new();
        for face in &self.faces {
            by_family
                .entry(face.family.clone())
                .or_default()
                .push(face);
        }
        let mut body = String::new();
        for (family, slots) in by_family {
            body.push_str(&format!(
                r#"  <w:font w:name="{}">
    <w:charset w:val="86"/>
    <w:family w:val="roman"/>
    <w:pitch w:val="variable"/>
"#,
                escape_xml_attr(&family)
            ));
            for face in &slots {
                body.push_str(&format!(
                    r#"    <{attr} r:id="{rid}" w:fontKey="{{{key}}}"/>
"#,
                    attr = face.slot.attr(),
                    rid = face.rid,
                    key = format_guid(&face.guid),
                ));
            }
            // Bold-only families have no Regular file. Also advertise that
            // face as Regular so a non-bold run can still bind the embed.
            if !slots.iter().any(|f| f.slot == EmbedSlot::Regular) {
                if let Some(face) = slots.first() {
                    body.push_str(&format!(
                        r#"    <w:embedRegular r:id="{rid}" w:fontKey="{{{key}}}"/>
"#,
                        rid = face.rid,
                        key = format_guid(&face.guid),
                    ));
                }
            }
            body.push_str("  </w:font>\n");
        }
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
{body}</w:fonts>
"#
        )
    }
}

fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn family_from_bytes(data: &[u8]) -> Option<String> {
    let face = Face::parse(data, 0).ok()?;
    name_english(&face, ttf_parser::name_id::FAMILY, true)
        .or_else(|| name_english(&face, ttf_parser::name_id::TYPOGRAPHIC_FAMILY, true))
        .or_else(|| {
            name_english(&face, ttf_parser::name_id::FULL_NAME, true).and_then(family_from_full_name)
        })
        .or_else(|| name_english(&face, ttf_parser::name_id::FULL_NAME, true))
}

fn family_from_full_name(full: String) -> Option<String> {
    const SUFFIXES: &[&str] = &[
        " Bold Italic",
        " Bold Oblique",
        " Bold",
        " Italic",
        " Oblique",
        " Regular",
        " Medium",
        " Light",
        " Black",
        " Semibold",
        " SemiBold",
    ];
    for suffix in SUFFIXES {
        if let Some(rest) = full.strip_suffix(suffix) {
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

fn name_english(face: &Face<'_>, id: u16, unicode_only: bool) -> Option<String> {
    let mut fallback = None;
    for name in face.names() {
        if name.name_id != id {
            continue;
        }
        if unicode_only && !name.is_unicode() {
            continue;
        }
        let Some(s) = name.to_string() else {
            continue;
        };
        if s.trim().is_empty() {
            continue;
        }
        if name.language() == ttf_parser::Language::English_UnitedStates {
            return Some(s);
        }
        if fallback.is_none() {
            fallback = Some(s);
        }
    }
    fallback
}

fn slot_for_bytes(data: &[u8]) -> EmbedSlot {
    let face = Face::parse(data, 0).ok();
    let style = face.map(|f| face_subfamily(&f)).unwrap_or_default();
    let b = face_style_is_bold(&style);
    let i = face_style_is_italic(&style);
    match (b, i) {
        (true, true) => EmbedSlot::BoldItalic,
        (true, false) => EmbedSlot::Bold,
        (false, true) => EmbedSlot::Italic,
        (false, false) => EmbedSlot::Regular,
    }
}

fn font_guid(data: &[u8]) -> [u8; 16] {
    let mut guid = [0u8; 16];
    for (i, b) in data.iter().enumerate().take(4096) {
        guid[i % 16] ^= *b;
    }
    guid[6] = (guid[6] & 0x0f) | 0x40;
    guid[8] = (guid[8] & 0x3f) | 0x80;
    guid
}

fn format_guid(guid: &[u8; 16]) -> String {
    format!(
        "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid[0],
        guid[1],
        guid[2],
        guid[3],
        guid[4],
        guid[5],
        guid[6],
        guid[7],
        guid[8],
        guid[9],
        guid[10],
        guid[11],
        guid[12],
        guid[13],
        guid[14],
        guid[15],
    )
}

fn obfuscate_font(data: &[u8], guid: &[u8; 16]) -> Vec<u8> {
    let mut out = data.to_vec();
    for i in 0..32.min(out.len()) {
        out[i] ^= guid[i % 16];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn roboto() -> Vec<u8> {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
        std::fs::read(path).expect("Roboto-Regular.ttf")
    }

    #[test]
    fn plan_emits_odttf_and_font_table() {
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), roboto());
        let plan = build_font_embed_plan(&fonts);
        assert_eq!(plan.faces.len(), 1);
        let table = plan.font_table_xml();
        assert!(table.contains("w:name=\"Roboto\""));
        assert!(table.contains("w:embedRegular"));
    }

    #[test]
    fn nameless_face_uses_package_stem_not_k2f_font() {
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Bradley Hand Bold.ttf".into(), vec![0, 1, 2, 3]);
        let plan = build_font_embed_plan(&fonts);
        let table = plan.font_table_xml();
        assert!(
            table.contains("w:name=\"Bradley Hand Bold\""),
            "expected stem name, got {table}"
        );
        assert!(!table.contains("K2F Font"), "{table}");
        assert!(table.contains("w:embedRegular"), "{table}");
        assert_eq!(
            plan.run_embed("Bradley Hand Bold"),
            Some(("w:embedRegular", "rIdEF1"))
        );
    }

    #[test]
    fn full_name_bold_strips_style_suffix() {
        assert_eq!(
            super::family_from_full_name("Bradley Hand Bold".into()).as_deref(),
            Some("Bradley Hand")
        );
    }
}
