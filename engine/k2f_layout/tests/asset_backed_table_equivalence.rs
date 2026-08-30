use k2f_core::LockFile;
use k2f_layout::LayoutEngine;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn load_test_font() -> Vec<u8> {
    let font_path: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
    fs::read(&font_path).expect("Failed to read bundled test font")
}

#[test]
fn test_asset_backed_table_matches_inline_output() {
    let font_data = load_test_font();

    let theme_json = r##"{
        "palette": { "black": "#000000", "shape": "#4F46E5", "shape_border": "#1F2937" },
        "primitives": {
            "surfaces": { "shape": { "type": "solid", "color": "shape" } },
            "borders": { "shape": { "width_pt": 2000, "color": "shape_border" } },
            "corners": { "medium": 12000 }
        },
        "roles": {
            "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
            "document": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
            "shape": {
                "font_family": "default",
                "font_size": 12000,
                "line_height_mult": 1200,
                "color": "black",
                "box_decoration": {
                    "background": "shape",
                    "border": "shape",
                    "corner_radius": "medium",
                    "padding_pt": 0
                }
            }
        }
    }"##;

    let inline_content_json = r#"{
        "title": "Inline",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "root": {
            "id": "root",
            "role": "document",
            "content": { "type": "container", "value": { "children": [
                { "id": "t1", "role": "shape", "content": { "type": "table", "value": {
                    "column_widths": [{ "pt": 100000 }, { "pt": 100000 }],
                    "header_rows": 0,
                    "gap": 0,
                    "data": { "type": "inline", "rows": [
                        [
                            { "id": "c00", "role": "shape", "content": { "type": "image", "value": { "src": "asset://img00", "width": 40000, "height": 10000 } }, "modifiers": [], "layout": null },
                            { "id": "c01", "role": "shape", "content": { "type": "image", "value": { "src": "asset://img01", "width": 60000, "height": 12000 } }, "modifiers": [], "layout": null }
                        ],
                        [
                            { "id": "c10", "role": "shape", "content": { "type": "image", "value": { "src": "asset://img10", "width": 50000, "height": 20000 } }, "modifiers": [], "layout": null },
                            { "id": "c11", "role": "shape", "content": { "type": "image", "value": { "src": "asset://img11", "width": 30000, "height": 8000 } }, "modifiers": [], "layout": null }
                        ]
                    ] }
                } }, "modifiers": [], "layout": null }
            ] } },
            "modifiers": [],
            "layout": null
        }
    }"#;

    let asset_content_json = r#"{
        "title": "Asset",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "root": {
            "id": "root",
            "role": "document",
            "content": { "type": "container", "value": { "children": [
                { "id": "t1", "role": "shape", "content": { "type": "table", "value": {
                    "column_widths": [{ "pt": 100000 }, { "pt": 100000 }],
                    "header_rows": 0,
                    "gap": 0,
                    "data": { "type": "asset", "source": "assets/data/table.json" }
                } }, "modifiers": [], "layout": null }
            ] } },
            "modifiers": [],
            "layout": null
        }
    }"#;

    // Without assets, asset-backed compilation must error deterministically.
    assert!(LayoutEngine::compile_chunk(asset_content_json, theme_json, &font_data).is_err());

    // With assets, both should compile to identical lock output (incl. content hash binding).
    let mut assets: HashMap<String, Vec<u8>> = HashMap::new();
    assets.insert(
        "assets/data/table.json".to_string(),
        r#"{
            "rows": [
                [
                    { "id": "c00", "role": "shape", "content": { "type": "image", "value": { "src": "asset://img00", "width": 40000, "height": 10000 } }, "modifiers": [], "layout": null },
                    { "id": "c01", "role": "shape", "content": { "type": "image", "value": { "src": "asset://img01", "width": 60000, "height": 12000 } }, "modifiers": [], "layout": null }
                ],
                [
                    { "id": "c10", "role": "shape", "content": { "type": "image", "value": { "src": "asset://img10", "width": 50000, "height": 20000 } }, "modifiers": [], "layout": null },
                    { "id": "c11", "role": "shape", "content": { "type": "image", "value": { "src": "asset://img11", "width": 30000, "height": 8000 } }, "modifiers": [], "layout": null }
                ]
            ]
        }"#
        .as_bytes()
        .to_vec(),
    );

    let lock_inline: LockFile = serde_json::from_str(
        &LayoutEngine::compile_chunk(inline_content_json, theme_json, &font_data).unwrap(),
    )
    .unwrap();
    let lock_asset: LockFile = serde_json::from_str(
        &LayoutEngine::compile_chunk_with_assets(
            asset_content_json,
            theme_json,
            &font_data,
            &assets,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(lock_inline, lock_asset);
}
