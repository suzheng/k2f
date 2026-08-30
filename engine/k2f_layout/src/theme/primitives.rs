use crate::visual_primitives::{Blur, Border, Fill, LinearGradient, Shadow};
use serde::Deserialize;
use std::collections::HashMap;

/// Named visual atoms. Roles/variants reference these by string.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThemePrimitives {
    #[serde(default)]
    pub surfaces: HashMap<String, Fill>,
    #[serde(default)]
    pub gradients: HashMap<String, LinearGradient>,
    #[serde(default)]
    pub shadows: HashMap<String, Shadow>,
    #[serde(default)]
    pub blurs: HashMap<String, Blur>,
    #[serde(default)]
    pub corners: HashMap<String, i64>,
    #[serde(default)]
    pub borders: HashMap<String, Border>,
}
