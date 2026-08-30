/// Character index → UTF-8 byte offset (clamped to char boundaries / string end).
pub fn char_to_byte(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

/// UTF-8 byte offset → character index (clamps to nearest prior char boundary).
pub fn byte_to_char(text: &str, byte: usize) -> usize {
    let mut b = byte.min(text.len());
    while b > 0 && !text.is_char_boundary(b) {
        b -= 1;
    }
    text[..b].chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_char_byte_roundtrip() {
        let t = "你好世界";
        assert_eq!(char_to_byte(t, 0), 0);
        assert_eq!(char_to_byte(t, 1), 3);
        assert_eq!(char_to_byte(t, 2), 6);
        assert_eq!(char_to_byte(t, 4), 12);
        assert_eq!(byte_to_char(t, 0), 0);
        assert_eq!(byte_to_char(t, 3), 1);
        assert_eq!(byte_to_char(t, 6), 2);
        assert_eq!(byte_to_char(t, 12), 4);
    }
}
