use crate::{canonical_json_string, sha256_hex, PageConfig};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub struct AppearanceHashInput<'a> {
    pub content_hash: &'a str,
    pub theme_json: &'a str,
    pub fonts: &'a BTreeMap<String, Vec<u8>>,
    pub page_config: &'a PageConfig,
    pub engine_version: &'a str,
    pub engine_commit_sha: &'a str,
}

/// Appearance binding: semantic hash + canonical theme + embedded font digests
/// + page_config + engine identity. This is what VALID means for sign-off.
pub fn hash_appearance_binding(input: &AppearanceHashInput<'_>) -> Result<String, String> {
    let theme: Value =
        serde_json::from_str(input.theme_json).map_err(|e| format!("Theme JSON error: {e}"))?;

    #[derive(Serialize)]
    struct FontDigest {
        path: String,
        sha256: String,
    }

    #[derive(Serialize)]
    struct Shape<'a> {
        content_hash: &'a str,
        theme: Value,
        fonts: Vec<FontDigest>,
        page_config: &'a PageConfig,
        engine_version: &'a str,
        engine_commit_sha: &'a str,
    }

    let fonts = input
        .fonts
        .iter()
        .map(|(path, bytes)| FontDigest {
            path: path.clone(),
            sha256: sha256_hex(bytes),
        })
        .collect();

    let shape = Shape {
        content_hash: input.content_hash,
        theme,
        fonts,
        page_config: input.page_config,
        engine_version: input.engine_version,
        engine_commit_sha: input.engine_commit_sha,
    };
    Ok(sha256_hex(
        canonical_json_string(&shape)
            .map_err(|e| format!("Appearance hash serialization error: {e}"))?
            .as_bytes(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hash_manifest_semantic, CanvasMode, Manifest, NodeContent, PageConfig, Pt, SemanticNode,
    };

    fn text_manifest(text: &str) -> Manifest {
        Manifest {
            title: "t".to_string(),
            canvas_mode: CanvasMode::Paged,
            page_config: PageConfig {
                width: Pt(595000),
                height: Pt(842000),
                margin: [Pt(72000); 4],
            },
            root: SemanticNode {
                id: "root".to_string(),
                role: "body".to_string(),
                variant: None,
                preserve_whitespace: None,
                list_id: None,
                depth: None,
                marker_type: None,
                content: NodeContent::Text(text.to_string()),
                modifiers: vec![],
                layout: None,
                ..Default::default()
            },
            running_blocks: vec![],
        }
    }

    fn fonts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
        let mut m = BTreeMap::new();
        m.insert(
            "assets/fonts/Roboto-Regular.ttf".to_string(),
            bytes.to_vec(),
        );
        m
    }

    fn appearance(
        content_hash: &str,
        theme: &str,
        fonts: &BTreeMap<String, Vec<u8>>,
        page: &PageConfig,
    ) -> String {
        hash_appearance_binding(&AppearanceHashInput {
            content_hash,
            theme_json: theme,
            fonts,
            page_config: page,
            engine_version: "0.1.0",
            engine_commit_sha: "UNKNOWN",
        })
        .unwrap()
    }

    #[test]
    fn semantic_hash_changes_when_text_changes() {
        let a = hash_manifest_semantic(&text_manifest("hello"));
        let b = hash_manifest_semantic(&text_manifest("hallo"));
        assert_ne!(a, b);
    }

    #[test]
    fn theme_change_does_not_change_semantic_hash() {
        let m = text_manifest("hello");
        assert_eq!(hash_manifest_semantic(&m), hash_manifest_semantic(&m));
    }

    #[test]
    fn appearance_hash_changes_with_theme_not_semantic() {
        let m = text_manifest("hello");
        let semantic = hash_manifest_semantic(&m);
        let font = fonts(b"font-a");
        let theme_a = r#"{"palette":{},"roles":{"body":{"font_family":"default","font_size":12000,"line_height_mult":1200,"color":"black"}}}"#;
        let theme_b = r#"{"palette":{},"roles":{"body":{"font_family":"default","font_size":12000,"line_height_mult":1200,"color":"red"}}}"#;
        let appear_a = appearance(&semantic, theme_a, &font, &m.page_config);
        let appear_b = appearance(&semantic, theme_b, &font, &m.page_config);
        assert_eq!(hash_manifest_semantic(&m), semantic);
        assert_ne!(appear_a, appear_b);
    }

    #[test]
    fn appearance_hash_changes_with_font_bytes() {
        let m = text_manifest("hello");
        let semantic = hash_manifest_semantic(&m);
        let theme = r#"{"palette":{}}"#;
        let a = appearance(&semantic, theme, &fonts(b"aaa"), &m.page_config);
        let b = appearance(&semantic, theme, &fonts(b"bbb"), &m.page_config);
        assert_ne!(a, b);
    }

    #[test]
    fn appearance_hash_changes_with_page_config() {
        let mut m = text_manifest("hello");
        let semantic = hash_manifest_semantic(&m);
        let theme = r#"{"palette":{}}"#;
        let font = fonts(b"font");
        let a = appearance(&semantic, theme, &font, &m.page_config);
        m.page_config.margin = [Pt(10000); 4];
        let b = appearance(&semantic, theme, &font, &m.page_config);
        assert_ne!(a, b);
        assert_eq!(hash_manifest_semantic(&m), semantic);
    }
}
