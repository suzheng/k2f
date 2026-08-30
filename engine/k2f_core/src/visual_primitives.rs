use serde::{Deserialize, Serialize};

/// A color reference (e.g. "#RRGGBB", "#RRGGBBAA", or a named token key).
///
/// Resolution rules are owned by the consumer (theme/paint). The model preserves the
/// author-provided string to keep JSON stable and easy to edit.
pub type ColorRef = String;

/// A gradient stop position in 0..=1000, where 0 is start and 1000 is end.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GradientStop {
    pub pos: i64,
    pub color: ColorRef,
}

/// A linear gradient definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinearGradient {
    Linear {
        angle_degrees: i64,
        stops: Vec<GradientStop>,
    },
}

/// A fill that can be applied to backgrounds/surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Fill {
    Solid { color: ColorRef },
    LinearGradient { value: LinearGradient },
}

/// Either an inline fill definition or a reference to a named primitive.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum FillRef {
    Ref(String),
    Inline(Fill),
}

/// Which edges of a box receive a border stroke.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BorderEdge {
    Top,
    Right,
    Bottom,
    Left,
}

/// Stroke pattern for a named border primitive.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

fn default_border_edges() -> Vec<BorderEdge> {
    vec![
        BorderEdge::Top,
        BorderEdge::Right,
        BorderEdge::Bottom,
        BorderEdge::Left,
    ]
}

fn is_all_border_edges(edges: &[BorderEdge]) -> bool {
    edges.len() == 4
        && edges.contains(&BorderEdge::Top)
        && edges.contains(&BorderEdge::Right)
        && edges.contains(&BorderEdge::Bottom)
        && edges.contains(&BorderEdge::Left)
}

fn is_solid_border_style(style: &BorderStyle) -> bool {
    matches!(style, BorderStyle::Solid)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Border {
    pub width_pt: i64,
    pub color: ColorRef,
    /// Edges that receive the stroke. Omitted / default = all four sides.
    #[serde(
        default = "default_border_edges",
        skip_serializing_if = "is_all_border_edges"
    )]
    pub edges: Vec<BorderEdge>,
    /// Stroke pattern. Omitted / default = solid.
    #[serde(default, skip_serializing_if = "is_solid_border_style")]
    pub style: BorderStyle,
}

impl Border {
    pub fn draws_edge(&self, edge: BorderEdge) -> bool {
        self.edges.contains(&edge)
    }

    pub fn is_full_rect_stroke(&self) -> bool {
        is_all_border_edges(&self.edges) && matches!(self.style, BorderStyle::Solid)
    }
}

/// Padding / insets in fixed-point Pt (1/1000 pt units).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum EdgeInsetsPt {
    Uniform(i64),
    PerEdge {
        top: i64,
        right: i64,
        bottom: i64,
        left: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowLayer {
    pub offset_x_pt: i64,
    pub offset_y_pt: i64,
    pub blur_radius_pt: i64,
    pub spread_radius_pt: i64,
    pub color: ColorRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Shadow {
    pub layers: Vec<ShadowLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ShadowRef {
    Ref(String),
    Inline(Shadow),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Blur {
    pub radius_pt: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum BlurRef {
    Ref(String),
    Inline(Blur),
}

/// A minimal decoration model for painting a box.
///
/// This is intentionally representation-only. Consumers decide how to interpret references
/// and how to render shadows/blur deterministically.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BoxDecoration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<FillRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<Border>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corner_radius_pt: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_pt: Option<EdgeInsetsPt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<ShadowRef>,
    /// Backdrop blur applied to pixels behind this box (glass-style).
    ///
    /// The canonical execution path represents this as an explicit render-plan op
    /// (e.g. `PaintOp::BackdropBlur`) so the executor applies a deterministic algorithm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blur: Option<BlurRef>,
}
