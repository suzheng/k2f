use crate::{BoxDecoration, CompositingSpec, Pt, TextGlyphRun};
use serde::{Deserialize, Serialize};

/// A deterministic sequence of paint operations per page.
///
/// This complements geometry (positions/sizes/glyphs) by making draw order explicit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RenderPlan {
    /// Canonical compositing rules to be used when executing this plan.
    #[serde(default)]
    pub compositing: CompositingSpec,
    #[serde(default)]
    pub pages: Vec<PageRenderPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageRenderPlan {
    pub index: usize,
    #[serde(default)]
    pub ops: Vec<PaintOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaintOp {
    /// Apply a deterministic backdrop blur to the pixels *behind* a rect.
    ///
    /// Execution model:
    /// - The executor samples the current framebuffer within `rect` (including any content already
    ///   painted behind it), applies a deterministic blur, then writes the blurred pixels back
    ///   into `rect` (optionally clipped by `corner_radius_pt`).
    /// - Subsequent ops (e.g., DrawBox) paint on top of the blurred region.
    BackdropBlur {
        node_id: String,
        rect: Rect,
        radius_pt: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        corner_radius_pt: Option<i64>,
    },

    /// Paint the background/border/etc for a node's box.
    DrawBox {
        node_id: String,
        rect: Rect,
        decoration: BoxDecoration,
    },

    /// Paint shaped glyphs for a text node. Glyph positions are provided by geometry.
    DrawText {
        node_id: String,
        rect: Rect,
        #[serde(default)]
        runs: Vec<TextGlyphRun>,
    },

    /// Paint an image node.
    DrawImage {
        node_id: String,
        rect: Rect,
        src: String,
    },

    /// Paint a table reference placeholder/representation.
    DrawTableReference {
        node_id: String,
        rect: Rect,
        source: String,
        view_mode: String,
    },

    /// An op this engine cannot execute. Must fail as BROKEN; never skip.
    #[serde(other)]
    Unknown,
}

impl PaintOp {
    pub fn is_unknown(&self) -> bool {
        matches!(self, PaintOp::Unknown)
    }
}

pub fn render_plan_has_unknown_ops(plan: &RenderPlan) -> bool {
    plan.pages
        .iter()
        .any(|p| p.ops.iter().any(PaintOp::is_unknown))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_type_deserializes_as_unknown_not_skip() {
        let op: PaintOp = serde_json::from_str(r#"{"type":"draw_unicorn"}"#).unwrap();
        assert!(op.is_unknown());
        let plan: RenderPlan =
            serde_json::from_str(r#"{"pages":[{"index":0,"ops":[{"type":"draw_unicorn"}]}]}"#)
                .unwrap();
        assert!(render_plan_has_unknown_ops(&plan));
    }
}
