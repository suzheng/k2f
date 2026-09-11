//! OS UI faces for chrome text only. Document paint never uses these.

use fontdue::Font;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Lazily loads a few system faces until a character is covered.
pub struct SystemFaces {
    loaded: Vec<Font>,
    next: usize,
    candidates: Vec<(PathBuf, u32)>,
}

impl SystemFaces {
    pub fn new() -> Self {
        Self {
            loaded: Vec::new(),
            next: 0,
            candidates: candidates(),
        }
    }

    pub fn font_for(&mut self, ch: char) -> Option<&Font> {
        if let Some(i) = self.loaded.iter().position(|f| f.has_glyph(ch)) {
            return Some(&self.loaded[i]);
        }
        while self.next < self.candidates.len() {
            let (path, index) = self.candidates[self.next].clone();
            self.next += 1;
            if let Some(font) = load_face(&path, index) {
                let covered = font.has_glyph(ch);
                self.loaded.push(font);
                if covered {
                    return self.loaded.last();
                }
            }
        }
        None
    }
}

fn load_face(path: &Path, collection_index: u32) -> Option<Font> {
    let bytes = std::fs::read(path).ok()?;
    Font::from_bytes(
        bytes,
        fontdue::FontSettings {
            collection_index,
            ..Default::default()
        },
    )
    .ok()
}

fn candidates() -> Vec<(PathBuf, u32)> {
    #[cfg(target_os = "macos")]
    {
        const FACES: &[(&str, u32)] = &[
            ("/System/Library/Fonts/PingFang.ttc", 2), // PingFang SC Regular
            ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
            ("/System/Library/Fonts/STHeiti Medium.ttc", 0),
            ("/System/Library/Fonts/Supplemental/Songti.ttc", 0),
            ("/System/Library/Fonts/AppleSDGothicNeo.ttc", 0),
            ("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", 0),
        ];
        return FACES
            .iter()
            .map(|(p, i)| (PathBuf::from(p), *i))
            .collect();
    }
    #[cfg(target_os = "windows")]
    {
        let fonts = windows_fonts_dir();
        const NAMES: &[(&str, u32)] = &[
            ("msyh.ttc", 0),    // Microsoft YaHei
            ("msyh.ttf", 0),
            ("msjh.ttc", 0),    // Microsoft JhengHei
            ("malgun.ttf", 0),  // Malgun Gothic
            ("YuGothR.ttc", 0), // Yu Gothic
            ("msgothic.ttc", 0),
            ("seguiemj.ttf", 0),
            ("arialuni.ttf", 0),
        ];
        return NAMES
            .iter()
            .map(|(name, i)| (fonts.join(name), *i))
            .collect();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        const FACES: &[(&str, u32)] = &[
            ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
            ("/usr/share/fonts/opentype/noto/NotoSansCJKsc-Regular.otf", 0),
            ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
            ("/usr/share/fonts/noto-cjk/NotoSansCJKsc-Regular.otf", 0),
            (
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                0,
            ),
            ("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", 0),
            ("/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf", 0),
        ];
        return FACES
            .iter()
            .map(|(p, i)| (PathBuf::from(p), *i))
            .collect();
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        all(unix, not(target_os = "macos"))
    )))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "windows")]
fn windows_fonts_dir() -> PathBuf {
    std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("Fonts")
}

/// Shared lazy pool used by chrome text.
pub fn pool() -> &'static Mutex<SystemFaces> {
    static POOL: std::sync::OnceLock<Mutex<SystemFaces>> = std::sync::OnceLock::new();
    POOL.get_or_init(|| Mutex::new(SystemFaces::new()))
}
