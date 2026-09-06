pub fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Word `ST_HexColor` / DrawingML `srgbClr@val` must be exactly 6 hex digits.
pub fn word_hex_color(color: &str) -> String {
    let t = color.trim().trim_start_matches('#');
    if t.len() == 6 && t.bytes().all(|b| b.is_ascii_hexdigit()) {
        match t.to_ascii_uppercase().as_str() {
            "000000" => "000001".into(),
            "FFFFFF" => "FFFFFE".into(),
            u => u.to_string(),
        }
    } else {
        "000001".into()
    }
}
