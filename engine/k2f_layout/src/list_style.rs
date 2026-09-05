use crate::style::TextAlign;
use k2f_core::Pt;
use serde::Deserialize;

/// Theme tokens controlling list marker geometry and indentation.
///
/// All fields are optional so themes can partially override base role values via variants.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct ListStyle {
    /// Fixed-width marker box to keep wrap width independent of the marker label.
    #[serde(default)]
    pub marker_box_width_pt: Option<Pt>,

    /// Gap between marker box and the start of body text.
    #[serde(default)]
    pub marker_gap_pt: Option<Pt>,

    /// Additional indent per nesting depth level.
    #[serde(default)]
    pub depth_indent_pt: Option<Pt>,

    /// Alignment of the marker label within the fixed marker box.
    #[serde(default)]
    pub marker_align: Option<TextAlign>,

    /// Glyph used for bullet markers.
    #[serde(default)]
    pub bullet_glyph: Option<String>,

    /// Suffix appended to numbers (e.g. "." yields "1.").
    #[serde(default)]
    pub number_suffix: Option<String>,
}

impl ListStyle {
    /// Starter-theme defaults when `list_item` omits `list_style`.
    pub fn list_item_defaults() -> Self {
        Self {
            marker_box_width_pt: Some(Pt(18_000)),
            marker_gap_pt: Some(Pt(4_000)),
            depth_indent_pt: Some(Pt(18_000)),
            marker_align: None,
            bullet_glyph: Some("-".to_string()),
            number_suffix: Some(".".to_string()),
        }
    }

    /// Deterministically merge a base definition with an overlay definition.
    ///
    /// For each field, `overlay` wins when it is present; otherwise the value from `base` is kept.
    pub fn merged(base: Option<&ListStyle>, overlay: Option<&ListStyle>) -> Option<ListStyle> {
        if base.is_none() && overlay.is_none() {
            return None;
        }

        let mut out = base.cloned().unwrap_or_default();
        if let Some(ov) = overlay {
            out.apply_overrides(ov);
        }
        Some(out)
    }

    fn apply_overrides(&mut self, overlay: &ListStyle) {
        if overlay.marker_box_width_pt.is_some() {
            self.marker_box_width_pt = overlay.marker_box_width_pt;
        }
        if overlay.marker_gap_pt.is_some() {
            self.marker_gap_pt = overlay.marker_gap_pt;
        }
        if overlay.depth_indent_pt.is_some() {
            self.depth_indent_pt = overlay.depth_indent_pt;
        }
        if overlay.marker_align.is_some() {
            self.marker_align = overlay.marker_align;
        }
        if overlay.bullet_glyph.is_some() {
            self.bullet_glyph = overlay.bullet_glyph.clone();
        }
        if overlay.number_suffix.is_some() {
            self.number_suffix = overlay.number_suffix.clone();
        }
    }
}
