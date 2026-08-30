/// Escape K2F plain text so re-import does not treat punctuation as Markdown.
pub fn escape_md(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, c) in text.chars().enumerate() {
        let inline = matches!(c, '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>');
        let block = i == 0 && matches!(c, '#' | '>' | '-' | '+');
        if inline || block {
            out.push('\\');
        }
        out.push(c);
    }
    escape_ordered_prefix(out)
}

fn escape_ordered_prefix(s: String) -> String {
    let digits = s.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 && s.as_bytes().get(digits) == Some(&b'.') {
        let mut out = String::with_capacity(s.len() + 1);
        out.push_str(&s[..digits]);
        out.push('\\');
        out.push_str(&s[digits..]);
        out
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_emphasis_and_list_prefixes() {
        assert_eq!(escape_md("a * b"), r"a \* b");
        assert_eq!(escape_md("_x_"), r"\_x\_");
        assert_eq!(escape_md("- item"), r"\- item");
        assert_eq!(escape_md("1. step"), r"1\. step");
    }
}
