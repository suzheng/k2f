use k2f_core::LockFile;
use k2f_layout::LayoutEngine;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_e2e_compile_flow() {
    // 1. Setup Resources
    // Hermetic + deterministic: always use the vendored font (no fallback).
    let font_path: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
    let font_data = fs::read(&font_path).expect("Failed to read bundled test font");

    let content_json = r#"{
        "title": "E2E",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "running_blocks": [
            {
                "position": "footer",
                "node": {
                    "id": "rb.footer",
                    "role": "body",
                    "content": { "type": "text", "value": "Page {{page_current}} of {{page_total}}" },
                    "modifiers": []
                }
            }
        ],
        "root": {
            "id": "root",
            "role": "body",
            "content": { "type": "text", "value": "Hello WASM" },
            "modifiers": []
        }
    }"#;

    let theme_json = r#"{
        "palette": {},
        "roles": {
            "body": {
                "font_family": "default",
                "font_size": 12000,
                "line_height_mult": 1200,
                "color": "black"
            }
        }
    }"#;

    // 2. Compile
    let result_json = LayoutEngine::compile_chunk(content_json, theme_json, &font_data);

    // 3. Verify Result
    assert!(
        result_json.is_ok(),
        "Compilation failed: {:?}",
        result_json.err()
    );
    let json_str = result_json.unwrap();

    // 4. Parse LockFile
    let lock: LockFile = serde_json::from_str(&json_str).expect("Failed to deserialize LockFile");

    assert_eq!(lock.engine_version, env!("CARGO_PKG_VERSION"));
    assert!(!lock.content_hash.is_empty());

    println!("Generated Lock File Hash: {}", lock.content_hash);

    // 5. Determinism Check
    let result2 = LayoutEngine::compile_chunk(content_json, theme_json, &font_data).unwrap();
    let lock2: LockFile = serde_json::from_str(&result2).unwrap();
    assert_eq!(lock.content_hash, lock2.content_hash);
    assert_eq!(json_str, result2);

    // 6. Content hash must change when running blocks change.
    let content_json_changed = r#"{
        "title": "E2E",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "running_blocks": [
            {
                "position": "footer",
                "node": {
                    "id": "rb.footer",
                    "role": "body",
                    "content": { "type": "text", "value": "FOOTER CHANGED" },
                    "modifiers": []
                }
            }
        ],
        "root": {
            "id": "root",
            "role": "body",
            "content": { "type": "text", "value": "Hello WASM" },
            "modifiers": []
        }
    }"#;
    let changed =
        LayoutEngine::compile_chunk(content_json_changed, theme_json, &font_data).unwrap();
    let lock_changed: LockFile = serde_json::from_str(&changed).unwrap();
    assert_ne!(lock.content_hash, lock_changed.content_hash);
}
