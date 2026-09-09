use k2f_sdk::Editor;
use wasm_bindgen::prelude::*;

fn js_err(e: k2f_sdk::AgentError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Surgical editor. Save recompiles and drops signatures. Does not paint; the viewer paints the lock.
#[wasm_bindgen]
pub struct K2fEditor {
    inner: Editor,
}

#[wasm_bindgen]
impl K2fEditor {
    #[wasm_bindgen(constructor)]
    pub fn open(bytes: &[u8]) -> Result<K2fEditor, JsValue> {
        Ok(Self {
            inner: Editor::open(bytes).map_err(js_err)?,
        })
    }

    #[wasm_bindgen(js_name = openTemplate)]
    pub fn open_template(template: &str) -> Result<K2fEditor, JsValue> {
        Ok(Self {
            inner: Editor::open_template(template).map_err(js_err)?,
        })
    }

    pub fn outline(&self) -> String {
        self.inner.outline_json().unwrap_or_else(|_| "[]".into())
    }

    pub fn diff(&self) -> String {
        self.inner.diff_json().unwrap_or_else(|_| "[]".into())
    }

    pub fn get_node(&self, id: &str) -> Result<String, JsValue> {
        self.inner.get_node_json(id).map_err(js_err)
    }

    pub fn selection(&self, id: &str) -> Result<String, JsValue> {
        self.inner.selection_json(id).map_err(js_err)
    }

    pub fn clipboard(&self, id: &str) -> Result<String, JsValue> {
        self.inner.clipboard_json(id).map_err(js_err)
    }

    pub fn search(&self, query: &str) -> String {
        serde_json::to_string(&self.inner.search(query)).unwrap_or_else(|_| "[]".into())
    }

    pub fn replace_text(&mut self, id: &str, text: &str) -> Result<(), JsValue> {
        self.inner.replace_text(id, text).map_err(js_err)
    }

    pub fn set_role(
        &mut self,
        id: &str,
        role: &str,
        variant: Option<String>,
    ) -> Result<(), JsValue> {
        self.inner
            .set_role(id, role, variant.as_deref())
            .map_err(js_err)
    }

    pub fn insert_node(
        &mut self,
        parent_id: &str,
        index: u32,
        node_json: &str,
    ) -> Result<(), JsValue> {
        self.inner
            .insert_node(parent_id, index as usize, node_json)
            .map_err(js_err)
    }

    pub fn delete_node(&mut self, id: &str) -> Result<(), JsValue> {
        self.inner.delete_node(id).map_err(js_err)
    }

    pub fn set_generated_by(&mut self, id: &str) {
        self.inner.set_generated_by(id);
    }

    pub fn set_running_header(&mut self, text: &str) -> Result<(), JsValue> {
        self.inner.set_running_header(text).map_err(js_err)
    }

    pub fn set_running_footer(&mut self, text: &str) -> Result<(), JsValue> {
        self.inner.set_running_footer(text).map_err(js_err)
    }

    pub fn suggest(&mut self, id: &str, text: &str) -> Result<(), JsValue> {
        self.inner.suggest(id, text).map_err(js_err)
    }

    pub fn accept_suggestion(&mut self, id: &str) -> Result<(), JsValue> {
        self.inner.accept_suggestion(id).map_err(js_err)
    }

    pub fn reject_suggestion(&mut self, id: &str) -> Result<(), JsValue> {
        self.inner.reject_suggestion(id).map_err(js_err)
    }

    pub fn suggestions(&self) -> String {
        self.inner
            .suggestions_json()
            .unwrap_or_else(|_| "{}".into())
    }

    pub fn save(&mut self) -> Result<Vec<u8>, JsValue> {
        self.inner.save_bytes().map_err(js_err)
    }

    pub fn export_pptx(&self) -> Result<Vec<u8>, JsValue> {
        self.inner.export_pptx_bytes().map_err(js_err)
    }

    pub fn export_docx(&self) -> Result<Vec<u8>, JsValue> {
        self.inner.export_docx_bytes().map_err(js_err)
    }

    pub fn save_with(&mut self, expected_content_hash: Option<String>) -> Result<Vec<u8>, JsValue> {
        self.inner
            .save_with(expected_content_hash.as_deref())
            .map_err(js_err)
    }
}
