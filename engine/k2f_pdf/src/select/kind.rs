#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontKind {
    TrueType,
    CffOpenType,
}

pub fn font_kind(bytes: &[u8]) -> FontKind {
    if bytes.starts_with(b"OTTO") {
        FontKind::CffOpenType
    } else {
        FontKind::TrueType
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otto_is_cff() {
        assert_eq!(font_kind(b"OTTO...."), FontKind::CffOpenType);
        assert_eq!(font_kind(&[0, 1, 0, 0]), FontKind::TrueType);
    }
}
