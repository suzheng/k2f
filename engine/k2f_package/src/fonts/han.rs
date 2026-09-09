//! BMP Han coverage bitsets (U+4E00..=U+9FFF).

const BMP_START: u32 = 0x4E00;
const BMP_END: u32 = 0x9FFF;
const GB2312: &[u8] = include_bytes!("gb2312_hanzi.bin");
const BIG5_L1: &[u8] = include_bytes!("big5_l1_hanzi.bin");
const JISX0208: &[u8] = include_bytes!("jisx0208_hanzi.bin");

fn bitmap_contains(bits: &[u8], cp: u32) -> bool {
    if cp < BMP_START || cp > BMP_END {
        return false;
    }
    let i = (cp - BMP_START) as usize;
    bits[i >> 3] & (1 << (i & 7)) != 0
}

/// GB2312 ∪ Big5 level 1 ∪ JIS X 0208 kanji.
pub fn contains(cp: u32) -> bool {
    bitmap_contains(GB2312, cp) || bitmap_contains(BIG5_L1, cp) || bitmap_contains(JISX0208, cp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_keeps_sc_tc_jp_daily_han() {
        assert!(contains('你' as u32));
        assert!(contains('體' as u32));
        assert!(contains('峠' as u32));
        assert!(!contains(b'A' as u32));
        assert!(!contains(0x9FA6));
    }
}
