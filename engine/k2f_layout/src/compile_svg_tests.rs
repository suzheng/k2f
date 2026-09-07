use crate::LayoutEngine;
use k2f_core::AssetsMap;
use std::path::PathBuf;

fn load_test_font() -> Vec<u8> {
    let font_path: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
    std::fs::read(&font_path).expect("bundled test font")
}

fn theme_json() -> &'static str {
    r##"{
        "palette": { "ink": "#111111" },
        "roles": {
            "document": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "ink" },
            "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "ink" },
            "image": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "ink" }
        }
    }"##
}

fn content_json() -> &'static str {
    r#"{
        "title": "svg",
        "canvas_mode": "paged",
        "page_config": { "width": 200000, "height": 200000, "margin": [10000, 10000, 10000, 10000] },
        "root": {
            "id": "root",
            "role": "document",
            "content": { "type": "container", "value": { "children": [
                {
                    "id": "fig",
                    "role": "image",
                    "content": { "type": "image", "value": { "src": "assets/images/fig.svg", "width": 40000, "height": 40000 } }
                }
            ] } }
        }
    }"#
}

#[test]
fn compile_rejects_svg_text_element() {
    let mut assets = AssetsMap::new();
    assets.insert(
        "assets/images/fig.svg".into(),
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><text x="1" y="2">A</text></svg>"#.to_vec(),
    );
    let err = LayoutEngine::compile_chunk_with_assets(
        content_json(),
        theme_json(),
        &load_test_font(),
        &assets,
    )
    .unwrap_err();
    assert!(err.contains("IMAGE_SIZE"), "{err}");
    assert!(err.contains("<text>"), "{err}");
    assert!(err.contains("<path>"), "{err}");
}

#[test]
fn compile_allows_svg_with_text_only_in_comment() {
    let mut assets = AssetsMap::new();
    assets.insert(
        "assets/images/fig.svg".into(),
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><!-- <text> converted to path --><rect width="4" height="4" fill="#00f"/></svg>"##.to_vec(),
    );
    LayoutEngine::compile_chunk_with_assets(
        content_json(),
        theme_json(),
        &load_test_font(),
        &assets,
    )
    .unwrap();
}

#[test]
fn compile_rejects_svg_textpath() {
    let mut assets = AssetsMap::new();
    assets.insert(
        "assets/images/fig.svg".into(),
        br##"<svg xmlns="http://www.w3.org/2000/svg"><textPath href="#p">A</textPath></svg>"##
            .to_vec(),
    );
    let err = LayoutEngine::compile_chunk_with_assets(
        content_json(),
        theme_json(),
        &load_test_font(),
        &assets,
    )
    .unwrap_err();
    assert!(err.contains("IMAGE_SIZE"), "{err}");
    assert!(err.contains("<path>"), "{err}");
}
