use serde::{Deserialize, Serialize};

use crate::{K2FError, NodeContent, Pt, SemanticNode};

/// Semantic role for fillable form fields (lock-size boxes; value does not reflow).
pub const ROLE_FORM_FIELD: &str = "form_field";

pub const CHECKBOX_UNCHECKED: &str = "";
pub const CHECKBOX_CHECKED: &str = "true";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FormFieldKind {
    Text,
    Multiline,
    Checkbox,
}

impl FormFieldKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Multiline => "multiline",
            Self::Checkbox => "checkbox",
        }
    }

    pub fn is_checkbox(self) -> bool {
        matches!(self, Self::Checkbox)
    }
}

/// Fillable field payload. Outer box size is declared (`width`/`height`/`lines`);
/// `value` is authority for what is written in the box and must not drive measure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormFieldSpec {
    pub kind: FormFieldKind,
    /// NFC-normalized. Checkbox allows only `""` or `"true"`.
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// millipt; omitted → parent max width at layout (unbounded fails there).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<Pt>,
    /// millipt; omitted → `lines` × role line-height + padding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<Pt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
}

impl FormFieldSpec {
    pub fn char_len(&self) -> usize {
        self.value.chars().count()
    }

    pub fn exceeds_max_length(&self) -> bool {
        self.max_length
            .is_some_and(|max| self.char_len() > max as usize)
    }
}

/// Validates invariants for form-field nodes. Does not mutate the node.
pub fn validate_form_field_invariants(node: &SemanticNode) -> Result<(), K2FError> {
    let is_role = node.role == ROLE_FORM_FIELD;
    let spec = match &node.content {
        NodeContent::FormField(spec) => Some(spec),
        _ => None,
    };

    if is_role && spec.is_none() {
        return Err(K2FError::FormFieldRoleRequiresFormFieldContent {
            node_id: node.id.clone(),
            got: node.content.type_name().to_string(),
        });
    }
    if spec.is_some() && !is_role {
        return Err(K2FError::FormFieldContentRequiresFormFieldRole {
            node_id: node.id.clone(),
            role: node.role.clone(),
        });
    }

    let Some(spec) = spec else {
        return Ok(());
    };

    if node.layout.is_some() {
        return Err(K2FError::FormFieldLayoutNotAllowed {
            node_id: node.id.clone(),
        });
    }
    if !node.modifiers.is_empty() {
        return Err(K2FError::FormFieldModifiersNotAllowed {
            node_id: node.id.clone(),
        });
    }

    if spec.kind.is_checkbox()
        && spec.value != CHECKBOX_UNCHECKED
        && spec.value != CHECKBOX_CHECKED
    {
        return Err(K2FError::FormFieldCheckboxValue {
            node_id: node.id.clone(),
            got: spec.value.clone(),
        });
    }

    if spec.lines == Some(0) {
        return Err(K2FError::FormFieldLines {
            node_id: node.id.clone(),
            got: 0,
        });
    }

    if spec.width.is_some_and(|w| w.0 <= 0) || spec.height.is_some_and(|h| h.0 <= 0) {
        return Err(K2FError::FormFieldSize {
            node_id: node.id.clone(),
        });
    }

    if spec.exceeds_max_length() {
        return Err(K2FError::FormFieldMaxLength {
            node_id: node.id.clone(),
            max: spec.max_length.unwrap(),
            got: spec.char_len(),
        });
    }

    Ok(())
}
