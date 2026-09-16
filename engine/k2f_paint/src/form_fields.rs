use k2f_core::{for_each_form_field_in_trees, FormFieldKind};

use crate::document::OpenedDocument;

/// A fillable field joined with its lock box. Geometry comes from `boxes_for`;
/// value comes from the semantic tree. Not a lock-file field.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FormFieldLoc {
    pub id: String,
    pub page: usize,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub kind: FormFieldKind,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
}

impl OpenedDocument {
    /// Fields in document order. Uses the first lock box per id (`break_inside: avoid`
    /// should keep that to one). Nodes with no geometry are omitted.
    pub fn form_fields(&self) -> Vec<FormFieldLoc> {
        let mut out = Vec::new();
        for_each_form_field_in_trees(&self.query_root, &self.query_running, &mut |node, spec| {
            let Some(b) = self.boxes_for(&node.id).into_iter().next() else {
                return;
            };
            out.push(FormFieldLoc {
                id: node.id.clone(),
                page: b.page,
                x: b.x,
                y: b.y,
                width: b.width,
                height: b.height,
                kind: spec.kind,
                value: spec.value.clone(),
                placeholder: spec.placeholder.clone(),
                required: spec.required,
                max_length: spec.max_length,
            });
        });
        out
    }
}
