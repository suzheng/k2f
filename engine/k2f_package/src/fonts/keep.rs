//! Which cmap codepoints to keep when coverage-subsetting a face.

pub fn is_cjk_ideograph(cp: u32) -> bool {
    matches!(
        cp,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0x2CEB0..=0x2EBEF
            | 0x30000..=0x3134F
    )
}

pub fn keep_codepoint(cp: u32) -> bool {
    !is_cjk_ideograph(cp) || crate::fonts::han::contains(cp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_han_always_kept() {
        assert!(keep_codepoint(b'A' as u32));
        assert!(keep_codepoint('あ' as u32));
        assert!(keep_codepoint('한' as u32));
        assert!(keep_codepoint('，' as u32));
    }

    #[test]
    fn han_union_not_only_gb2312() {
        assert!(keep_codepoint('你' as u32));
        assert!(keep_codepoint('體' as u32));
        assert!(keep_codepoint('峠' as u32));
        assert!(!keep_codepoint(0x9FA6));
    }
}
