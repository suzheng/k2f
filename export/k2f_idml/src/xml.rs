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

#[allow(dead_code)]
pub fn sanitize_name(id: &str) -> String {
    let s: String = id
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '.' | '-' => c,
            _ => '_',
        })
        .collect();
    if s.starts_with(|c: char| c.is_ascii_digit()) || s.is_empty() {
        format!("n{s}")
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_special_chars_escaped() {
        assert_eq!(escape_xml("a&b<c>"), "a&amp;b&lt;c&gt;");
    }
}
