use serde::{Deserialize, Serialize};

/// Defines how colors must be blended when executing a render plan.
///
/// This is part of the deterministic contract: without an explicit policy, different
/// platforms can produce visibly different results for transparency.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompositingColorSpace {
    /// Standard gamma-encoded sRGB compositing (common web look).
    Srgb,
}

/// Defines the alpha representation policy expected by the render executor.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlphaMode {
    /// Colors are treated as premultiplied by alpha during compositing.
    Premultiplied,
}

/// Blend mode for an operation group.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    /// Standard source-over alpha compositing.
    SourceOver,
}

/// Canonical compositing policy for executing a render plan.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompositingSpec {
    pub color_space: CompositingColorSpace,
    pub alpha_mode: AlphaMode,
    pub blend_mode: BlendMode,
}

impl Default for CompositingSpec {
    fn default() -> Self {
        Self {
            color_space: CompositingColorSpace::Srgb,
            alpha_mode: AlphaMode::Premultiplied,
            blend_mode: BlendMode::SourceOver,
        }
    }
}
