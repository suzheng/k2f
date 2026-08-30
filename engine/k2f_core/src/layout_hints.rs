use crate::Pt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

impl Default for Align {
    fn default() -> Self {
        Align::Stretch
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JustifyContent {
    Start,
    Center,
    End,
}

impl Default for JustifyContent {
    fn default() -> Self {
        JustifyContent::Start
    }
}

/// Default per-cell alignment for grid children.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CellAlign {
    #[serde(default)]
    pub x: Align,
    #[serde(default)]
    pub y: Align,
}

/// Optional deterministic fixed outer size for container layout hints.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FixedSizeHint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<Pt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<Pt>,
}
