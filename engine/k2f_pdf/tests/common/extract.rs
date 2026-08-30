//! Decode Identity-H `Tj` + ToUnicode `beginbfchar`. Does not use `lopdf::extract_text`.

#![allow(dead_code)]

use lopdf::{Dictionary, Document, Object, ObjectId};
use std::collections::HashMap;

pub fn extract_pdf_text(pdf: &[u8]) -> String {
    let doc = Document::load_mem(pdf).expect("pdf");
    let mut out = String::new();
    for id in doc.get_pages().values() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&page_text(&doc, *id));
    }
    out
}

pub fn page_has_font(pdf: &[u8], page_index: usize) -> bool {
    let doc = Document::load_mem(pdf).expect("pdf");
    let id = *doc.get_pages().values().nth(page_index).expect("page");
    doc.get_page_fonts(id)
        .map(|f| !f.is_empty())
        .unwrap_or(false)
}

pub fn page_content_has(pdf: &[u8], page_index: usize, needle: &str) -> bool {
    let doc = Document::load_mem(pdf).expect("pdf");
    let id = *doc.get_pages().values().nth(page_index).expect("page");
    let raw = doc.get_page_content(id).expect("content");
    String::from_utf8_lossy(&raw).contains(needle)
}

fn page_text(doc: &Document, page_id: ObjectId) -> String {
    let cmap = page_cmap(doc, page_id);
    let raw = doc.get_page_content(page_id).unwrap_or_default();
    let ops = lopdf::content::Content::decode(&raw).unwrap_or(lopdf::content::Content {
        operations: Vec::new(),
    });
    let mut out = String::new();
    for op in ops.operations {
        match op.operator.as_str() {
            "Tj" | "'" => {
                for operand in &op.operands {
                    push_string(&mut out, operand, &cmap);
                }
            }
            "TJ" => {
                if let Some(Object::Array(arr)) = op.operands.first() {
                    for item in arr {
                        push_string(&mut out, item, &cmap);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn page_cmap(doc: &Document, page_id: ObjectId) -> HashMap<u16, String> {
    let mut merged = HashMap::new();
    let Ok(fonts) = doc.get_page_fonts(page_id) else {
        return merged;
    };
    for font in fonts.values() {
        if let Some(map) = to_unicode_map(doc, font) {
            merged.extend(map);
        }
    }
    merged
}

fn to_unicode_map(doc: &Document, font: &Dictionary) -> Option<HashMap<u16, String>> {
    let obj = font.get(b"ToUnicode").ok()?;
    let stream = match obj {
        Object::Reference(id) => doc.get_object(*id).ok()?.as_stream().ok()?,
        Object::Stream(s) => s,
        _ => return None,
    };
    Some(parse_bfchar(&stream.get_plain_content().ok()?))
}

fn parse_bfchar(cmap: &[u8]) -> HashMap<u16, String> {
    let text = String::from_utf8_lossy(cmap);
    let mut out = HashMap::new();
    let mut in_bf = false;
    for line in text.lines() {
        let line = line.trim();
        if line.contains("beginbfchar") {
            in_bf = true;
            continue;
        }
        if line.contains("endbfchar") {
            in_bf = false;
            continue;
        }
        if !in_bf {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(src) = parts.next().and_then(hex_angles) else {
            continue;
        };
        let Some(dst) = parts.next().and_then(hex_angles) else {
            continue;
        };
        if src.len() < 2 {
            continue;
        }
        let gid = u16::from_be_bytes([src[src.len() - 2], src[src.len() - 1]]);
        out.insert(gid, utf16be_string(&dst));
    }
    out
}

fn hex_angles(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    let inner = s.strip_prefix('<')?.strip_suffix('>')?;
    if inner.len() % 2 != 0 {
        return None;
    }
    (0..inner.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&inner[i..i + 2], 16).ok())
        .collect()
}

fn utf16be_string(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&units).unwrap_or_default()
}

fn push_string(out: &mut String, obj: &Object, cmap: &HashMap<u16, String>) {
    let Object::String(bytes, _) = obj else {
        return;
    };
    for chunk in bytes.chunks_exact(2) {
        let gid = u16::from_be_bytes([chunk[0], chunk[1]]);
        if let Some(s) = cmap.get(&gid) {
            out.push_str(s);
        }
    }
}
