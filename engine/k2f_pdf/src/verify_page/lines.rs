use k2f_paint::OpenedDocument;

const SOURCE_NOTE: &str =
    "Official source is the .K2F package. This PDF is a copy, not the signed original.";

pub fn verification_lines(doc: &OpenedDocument) -> Vec<String> {
    let mut lines = vec![
        "K2F integrity verification".into(),
        format!("status: {}", doc.banner().as_str()),
        format!("status_code: {}", doc.status_code()),
    ];
    if doc.hash_code() != doc.status_code() {
        lines.push(format!("hash_code: {}", doc.hash_code()));
    }
    if let Some(h) = doc.content_hash() {
        lines.push(format!("content_hash: {h}"));
    }
    if let Some(h) = doc.appearance_hash() {
        lines.push(format!("appearance_hash: {h}"));
    }
    if let Some(fp) = doc.fingerprint() {
        lines.push(format!("fingerprint: {fp}"));
    }
    if let Some(by) = doc.signed_by() {
        lines.push(format!("signed_by: {by}"));
    }
    if let Some(at) = doc.signed_at() {
        lines.push(format!("signed_at: {at}"));
    }
    if let Some(by) = doc.generated_by() {
        lines.push(format!("generated_by: {by}"));
    }
    lines.push(String::new());
    lines.push(SOURCE_NOTE.into());
    lines
}
