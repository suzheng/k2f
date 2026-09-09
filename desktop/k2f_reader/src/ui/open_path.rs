//! Turn Finder / argv / `file://` strings into a filesystem path.
//! Unit-tested without AppKit.

use std::path::PathBuf;

/// Parse a path or `file:` URL from the OS open-document event.
pub fn path_from_open_string(s: &str) -> Option<PathBuf> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix("file:") {
        let rest = rest.strip_prefix("//").unwrap_or(rest);
        let path = if rest.starts_with('/') {
            rest
        } else if let Some(slash) = rest.find('/') {
            &rest[slash..]
        } else {
            return None;
        };
        return Some(PathBuf::from(percent_decode(path)));
    }
    Some(PathBuf::from(s))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
