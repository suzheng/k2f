use k2f_sdk::SYSTEM_PROMPT;
use wasm_bindgen::prelude::*;

fn js_err(e: k2f_sdk::AgentError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

#[wasm_bindgen]
pub fn system_prompt() -> String {
    SYSTEM_PROMPT.to_string()
}

#[wasm_bindgen]
pub fn official_templates() -> String {
    serde_json::to_string(k2f_sdk::OFFICIAL_IDS).unwrap_or_else(|_| "[]".into())
}

#[wasm_bindgen]
pub fn resolve_template(template: &str) -> Result<String, JsValue> {
    Ok(k2f_sdk::resolve(template)
        .map_err(js_err)?
        .display()
        .to_string())
}

#[wasm_bindgen]
pub fn copy_template(template: &str, dest: &str) -> Result<(), JsValue> {
    k2f_sdk::copy_to(template, std::path::Path::new(dest)).map_err(js_err)
}

#[wasm_bindgen]
pub fn generate_signing_key() -> Result<String, JsValue> {
    let key = k2f_sdk::generate_key().map_err(js_err)?;
    serde_json::to_string(&serde_json::json!({
        "secret_hex": key.secret_hex,
        "public_hex": key.public_hex,
        "fingerprint": key.fingerprint,
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn sign_k2f(
    package_bytes: &[u8],
    secret_hex: &str,
    signed_by: Option<String>,
    signed_at: Option<i64>,
) -> Result<Vec<u8>, JsValue> {
    k2f_sdk::sign(package_bytes, secret_hex, signed_by.as_deref(), signed_at).map_err(js_err)
}

#[wasm_bindgen]
pub fn markdown_to_k2f(md: &str, title: &str, template: &str) -> Result<Vec<u8>, JsValue> {
    let opts = k2f_sdk::MarkdownOptions::new(title, template).map_err(js_err)?;
    Ok(k2f_sdk::markdown_to_k2f(md, opts).map_err(js_err)?.bytes)
}

#[wasm_bindgen]
pub fn k2f_to_markdown(bytes: &[u8]) -> Result<String, JsValue> {
    k2f_sdk::k2f_to_markdown(bytes).map_err(js_err)
}
