use crate::visual_primitives::EdgeInsetsPt;
use serde::Deserialize;

/// Theme-side box decoration: named primitive refs plus layout padding.
///
/// Compile resolves names into an inlined lock `BoxDecoration`.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct ThemeDecoration {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub border: Option<String>,
    #[serde(default)]
    pub corner_radius: Option<String>,
    #[serde(default)]
    pub padding_pt: Option<EdgeInsetsPt>,
    #[serde(default)]
    pub shadow: Option<String>,
    #[serde(default)]
    pub blur: Option<String>,
}

impl ThemeDecoration {
    pub fn merge_from(&mut self, overlay: &ThemeDecoration) {
        if overlay.background.is_some() {
            self.background = overlay.background.clone();
        }
        if overlay.border.is_some() {
            self.border = overlay.border.clone();
        }
        if overlay.corner_radius.is_some() {
            self.corner_radius = overlay.corner_radius.clone();
        }
        if overlay.padding_pt.is_some() {
            self.padding_pt = overlay.padding_pt.clone();
        }
        if overlay.shadow.is_some() {
            self.shadow = overlay.shadow.clone();
        }
        if overlay.blur.is_some() {
            self.blur = overlay.blur.clone();
        }
    }

    pub fn has_paint_refs(&self) -> bool {
        self.background.is_some()
            || self.border.is_some()
            || self.corner_radius.is_some()
            || self.shadow.is_some()
            || self.blur.is_some()
    }
}

pub fn unknown_primitive(kind: &str, name: &str) -> String {
    format!("UNKNOWN_PRIMITIVE: {kind} '{name}'")
}
