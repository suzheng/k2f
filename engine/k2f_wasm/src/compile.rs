use k2f_layout::LayoutEngine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile_chunk(
    content_json: &str,
    theme_json: &str,
    font_blob: &[u8],
) -> Result<String, JsValue> {
    LayoutEngine::compile_chunk(content_json, theme_json, font_blob)
        .map_err(|e| JsValue::from_str(&e))
}

/// Compile with an explicit assets map.
///
/// `assets_json` must be a JSON object mapping asset keys to UTF-8 string contents, e.g.:
/// `{ "assets/data/table.json": "{ \"rows\": [...] }" }`
#[wasm_bindgen]
pub fn compile_chunk_with_assets(
    content_json: &str,
    theme_json: &str,
    font_blob: &[u8],
    assets_json: &str,
) -> Result<String, JsValue> {
    let assets_str_map: std::collections::HashMap<String, String> =
        serde_json::from_str(assets_json).map_err(|e| JsValue::from_str(&format!("{e}")))?;

    let mut assets: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
    for (k, v) in assets_str_map {
        assets.insert(k, v.into_bytes());
    }

    LayoutEngine::compile_chunk_with_assets(content_json, theme_json, font_blob, &assets)
        .map_err(|e| JsValue::from_str(&e))
}
