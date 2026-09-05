use std::collections::BTreeMap;
use ttf_parser::{name_id, Face, Language};

pub(crate) struct FontCtx {
    default_family: String,
    families: BTreeMap<String, String>,
    bytes: BTreeMap<String, Vec<u8>>,
}

impl FontCtx {
    pub(crate) fn new(fonts: &BTreeMap<String, Vec<u8>>) -> Self {
        let mut families = BTreeMap::new();
        for (key, data) in fonts {
            if let Some(name) = family_from_bytes(data) {
                families.insert(key.clone(), name);
            }
        }
        let default_family = fonts
            .get("default")
            .and_then(|b| family_from_bytes(b))
            .or_else(|| {
                fonts
                    .iter()
                    .find(|(k, _)| k.contains("Roboto") || *k == "default")
                    .and_then(|(_, b)| family_from_bytes(b))
            })
            .or_else(|| families.values().next().cloned())
            .unwrap_or_else(|| "Roboto".into());
        if let Some(data) = fonts.get("default") {
            if let Some(name) = family_from_bytes(data) {
                families.insert("default".into(), name);
            }
        }
        families
            .entry("default".into())
            .or_insert_with(|| default_family.clone());
        Self {
            default_family,
            families,
            bytes: fonts.clone(),
        }
    }

    pub(crate) fn typeface(&self, family: &str) -> String {
        if let Some(name) = self.families.get(family) {
            return name.clone();
        }
        if family == "default" {
            return self.default_family.clone();
        }
        if self.bytes.contains_key(family) {
            return family_from_bytes(&self.bytes[family])
                .unwrap_or_else(|| self.default_family.clone());
        }
        family.to_string()
    }

    pub(crate) fn bytes_for(&self, family: &str) -> Option<&[u8]> {
        if let Some(b) = self.bytes.get(family) {
            return Some(b.as_slice());
        }
        if family == "default" {
            if let Some(b) = self.bytes.get("default") {
                return Some(b.as_slice());
            }
            if let Some((_, b)) = self.bytes.iter().find(|(k, _)| k.contains("Roboto")) {
                return Some(b.as_slice());
            }
            return self
                .bytes
                .values()
                .min_by_key(|b| b.len())
                .map(|b| b.as_slice());
        }
        None
    }
}

fn family_from_bytes(data: &[u8]) -> Option<String> {
    let face = Face::parse(data, 0).ok()?;
    let mut fallback = None;
    for name in face.names() {
        if name.name_id != name_id::FAMILY || !name.is_unicode() {
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
