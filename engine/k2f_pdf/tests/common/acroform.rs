//! lopdf helpers for AcroForm structure. Parse only — not a second writer.

use lopdf::{Dictionary, Document, Object};

pub fn acroform<'a>(doc: &'a Document) -> Option<&'a Dictionary> {
    doc.get_dict_in_dict(doc.catalog().ok()?, b"AcroForm").ok()
}

pub fn field_dicts(pdf: &[u8]) -> Vec<Dictionary> {
    let doc = Document::load_mem(pdf).expect("pdf");
    let Some(acro) = acroform(&doc) else {
        return Vec::new();
    };
    let Ok(Object::Array(items)) = acro.get(b"Fields") else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|obj| match obj {
            Object::Reference(id) => doc.get_dictionary(*id).ok().cloned(),
            Object::Dictionary(d) => Some(d.clone()),
            _ => None,
        })
        .collect()
}

pub fn has_acroform(pdf: &[u8]) -> bool {
    let doc = Document::load_mem(pdf).expect("pdf");
    acroform(&doc).is_some()
}

pub fn text_of(obj: &Object) -> String {
    match obj {
        Object::String(bytes, _) => decode_pdf_string(bytes),
        Object::Name(n) => String::from_utf8_lossy(n).into_owned(),
        _ => String::new(),
    }
}

fn decode_pdf_string(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let units: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16(&units).unwrap_or_default();
    }
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn field_alt_names(pdf: &[u8]) -> Vec<String> {
    field_dicts(pdf)
        .iter()
        .filter_map(|d| d.get(b"TU").ok().map(text_of))
        .collect()
}

pub fn field_values(pdf: &[u8]) -> Vec<String> {
    field_dicts(pdf)
        .iter()
        .filter_map(|d| d.get(b"V").ok().map(text_of))
        .collect()
}

pub fn field_by_alt_name(pdf: &[u8], alt: &str) -> Option<Dictionary> {
    field_dicts(pdf).into_iter().find(|d| {
        d.get(b"TU")
            .ok()
            .map(text_of)
            .as_deref()
            .is_some_and(|tu| tu == alt)
    })
}

pub fn field_type(d: &Dictionary) -> Option<String> {
    d.get(b"FT").ok().map(text_of)
}

pub fn field_int(d: &Dictionary, key: &[u8]) -> Option<i64> {
    match d.get(key).ok()? {
        Object::Integer(v) => Some(*v),
        _ => None,
    }
}

pub fn glyph_note_count(pdf: &[u8]) -> usize {
    let parsed = Document::load_mem(pdf).expect("pdf");
    let mut n = 0;
    for id in parsed.get_pages().values() {
        let content = parsed.get_page_content(*id).unwrap_or_default();
        let (_, glyphs, _) = k2f_pdf::parse_notes(&content);
        n += glyphs.len();
    }
    n
}
