use k2f_core::is_font_face_path;
use std::collections::BTreeMap;
use ttf_parser::{name_id, Face, Language};

pub(crate) struct FontCtx {
    default_family: String,
    bytes: BTreeMap<String, Vec<u8>>,
}

impl FontCtx {
    pub(crate) fn new(fonts: &BTreeMap<String, Vec<u8>>) -> Self {
        let bytes: BTreeMap<String, Vec<u8>> = fonts
            .iter()
            .filter(|(k, _)| is_font_face_path(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Self {
            default_family: embedded_family(&bytes).unwrap_or_else(|| "Roboto".into()),
            bytes,
        }
    }

    pub(crate) fn typeface(&self, family: &str) -> String {
        if let Some(data) = self.bytes_for(family) {
            if let Some(name) = names_from_bytes(data).map(|n| n.family) {
                return name;
            }
        }
        if family == "default" {
            return self.default_family.clone();
        }
        family.to_string()
    }

    pub(crate) fn face_style(&self, family: &str) -> String {
        if let Some(data) = self.bytes_for(family) {
            if let Some(n) = names_from_bytes(data) {
                return n.style;
            }
        }
        "Regular".into()
    }

    pub(crate) fn default_family(&self) -> &str {
        &self.default_family
    }

    pub(crate) fn default_style(&self) -> String {
        self.bytes
            .get("default")
            .and_then(|b| names_from_bytes(b))
            .or_else(|| self.bytes.values().find_map(|b| names_from_bytes(b)))
            .map(|n| n.style)
            .unwrap_or_else(|| "Regular".into())
    }

    pub(crate) fn bytes_for(&self, family: &str) -> Option<&[u8]> {
        if let Some(b) = self.bytes.get(family) {
            return Some(b.as_slice());
        }
        for (path, b) in &self.bytes {
            let p = std::path::Path::new(path);
            if p.file_stem().is_some_and(|s| s == family) {
                return Some(b.as_slice());
            }
            if p.file_name().is_some_and(|s| s == family) {
                return Some(b.as_slice());
            }
        }
        for b in self.bytes.values() {
            if family_from_bytes(b).as_deref() == Some(family) {
                return Some(b.as_slice());
            }
        }
        if family == "default" {
            if let Some(b) = self.bytes.get("default") {
                return Some(b.as_slice());
            }
            if let Some((_, b)) = self.bytes.iter().find(|(k, _)| k.contains("Roboto")) {
                return Some(b.as_slice());
            }
            return self.bytes.values().next().map(|b| b.as_slice());
        }
        None
    }
}

fn embedded_family(bytes: &BTreeMap<String, Vec<u8>>) -> Option<String> {
    if let Some(data) = bytes.get("default") {
        if let Some(n) = names_from_bytes(data) {
            return Some(n.family);
        }
    }
    bytes.values().find_map(|b| names_from_bytes(b).map(|n| n.family))
}

struct FaceNames {
    family: String,
    style: String,
}

fn family_from_bytes(data: &[u8]) -> Option<String> {
    names_from_bytes(data).map(|n| n.family)
}

fn names_from_bytes(data: &[u8]) -> Option<FaceNames> {
    let face = Face::parse(data, 0).ok()?;
    let family = name_english(&face, name_id::TYPOGRAPHIC_FAMILY)
        .or_else(|| name_english(&face, name_id::FAMILY))?;
    let style = name_english(&face, name_id::TYPOGRAPHIC_SUBFAMILY)
        .or_else(|| name_english(&face, name_id::SUBFAMILY))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Regular".into());
    Some(FaceNames { family, style })
}

fn name_english(face: &Face<'_>, id: u16) -> Option<String> {
    let mut fallback = None;
    for name in face.names() {
        if name.name_id != id || !name.is_unicode() {
            continue;
        }
        let Some(s) = name.to_string() else {
            continue;
        };
        if name.language() == Language::English_UnitedStates {
            return Some(s);
        }
        if fallback.is_none() {
            fallback = Some(s);
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn roboto_bytes() -> Vec<u8> {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        std::fs::read(p).expect("Roboto test fixture")
    }

    #[test]
    fn roboto_face_is_regular() {
        let n = names_from_bytes(&roboto_bytes()).expect("names");
        assert_eq!(n.family, "Roboto");
        assert_eq!(n.style, "Regular");
    }
}
