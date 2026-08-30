use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// Minimal theme vocabulary used for validating role/variant usage in semantic content.
///
/// This intentionally does **not** model the full theme surface area; it only captures the
/// allowed role keys and the allowed variant names per role.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThemeVocab {
    /// Role name -> role definition (only `variants` is consumed here).
    ///
    /// Any other properties on each role (typography, decorations, etc.) are ignored by serde.
    #[serde(default)]
    pub roles: HashMap<String, RoleVocabEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RoleVocabEntry {
    /// Variant name -> variant definition (shape ignored here).
    ///
    /// If absent, the role is treated as having no allowed variants.
    #[serde(default)]
    pub variants: HashMap<String, Value>,
}
