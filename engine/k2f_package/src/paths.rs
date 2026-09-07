pub const MANIFEST: &str = "manifest.json";
pub const ROOT: &str = "content/root.json";
pub const THEME: &str = "styles/theme.json";
pub const TOKENS: &str = "styles/tokens.json";
pub const CHANGELOG: &str = "changelog.json";
pub const LOCK: &str = "document.K2F.lock";
pub const SIGNATURES: &str = "signatures/v1.json";
pub const FORMAT_SCHEMA_FILES: &[&str] = &[
    "schema/manifest.schema.json",
    "schema/nodes.schema.json",
    "schema/styles.schema.json",
    "schema/visual_primitives.schema.json",
    "schema/signatures.schema.json",
];
pub const FONTS_DIR: &str = "assets/fonts/";
pub const IMAGES_DIR: &str = "assets/images/";
pub const DATA_DIR: &str = "assets/data/";

pub const EMPTY_CHANGELOG: &str = r#"{"entries":[]}"#;

pub fn is_font_path(path: &str) -> bool {
    path.starts_with(FONTS_DIR) && path.len() > FONTS_DIR.len()
}

/// TrueType/OpenType face (`.ttf` / `.otf`), including the in-memory `"default"` key.
pub fn is_font_face_path(path: &str) -> bool {
    k2f_core::is_font_face_path(path)
}

/// At least one face (not a licenses-only `assets/fonts/` tree).
pub fn has_font_face(fonts: &std::collections::BTreeMap<String, Vec<u8>>) -> bool {
    fonts.keys().any(|p| is_font_face_path(p))
}

pub fn is_schema_path(path: &str) -> bool {
    FORMAT_SCHEMA_FILES.contains(&path)
}

pub fn is_extra_asset_path(path: &str) -> bool {
    (path.starts_with(IMAGES_DIR) || path.starts_with(DATA_DIR)) && !path.ends_with('/')
}

/// Valid include target: `content/<segment>.json` or nested `content/a/b.json`.
/// Excludes `content/root.json` (the package entry).
pub fn is_content_json_path(path: &str) -> bool {
    if path == ROOT || !path.starts_with("content/") || !path.ends_with(".json") {
        return false;
    }
    let rest = &path["content/".len()..];
    if rest.is_empty() || rest.contains("..") || rest.contains("//") {
        return false;
    }
    let segments: Vec<&str> = rest.split('/').collect();
    for (i, seg) in segments.iter().enumerate() {
        if seg.is_empty() {
            return false;
        }
        let name = if i + 1 == segments.len() {
            seg.strip_suffix(".json").unwrap_or(seg)
        } else {
            seg
        };
        if name.is_empty()
            || !name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric())
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    Manifest,
    Root,
    Theme,
    Tokens,
    Changelog,
    Lock,
    Signatures,
    Font,
    ExtraAsset,
    Schema,
    ContentJson,
}

pub fn classify_path(path: &str) -> Option<PathKind> {
    match path {
        MANIFEST => Some(PathKind::Manifest),
        ROOT => Some(PathKind::Root),
        THEME => Some(PathKind::Theme),
        TOKENS => Some(PathKind::Tokens),
        CHANGELOG => Some(PathKind::Changelog),
        LOCK => Some(PathKind::Lock),
        SIGNATURES => Some(PathKind::Signatures),
        _ if is_font_path(path) => Some(PathKind::Font),
        _ if is_extra_asset_path(path) => Some(PathKind::ExtraAsset),
        _ if is_schema_path(path) => Some(PathKind::Schema),
        _ if is_content_json_path(path) => Some(PathKind::ContentJson),
        _ => None,
    }
}
