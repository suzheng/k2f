pub fn encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

pub fn decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("hex length must be even".into());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        out.push((nibble(bytes[i])? << 4) | nibble(bytes[i + 1])?);
    }
    Ok(out)
}

fn nibble(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err("hex must be lowercase 0-9a-f".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        assert_eq!(encode(&[0x00, 0xab, 0xff]), "00abff");
        assert_eq!(decode("00abff").unwrap(), vec![0x00, 0xab, 0xff]);
    }

    #[test]
    fn rejects_uppercase() {
        assert!(decode("AA").is_err());
    }
}
