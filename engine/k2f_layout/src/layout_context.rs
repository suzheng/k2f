use crate::Theme;
use k2f_core::Pt;
use k2f_text::FontLibrary;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// We hold fonts and theme.
pub struct LayoutContext<'a> {
    pub fonts: &'a FontLibrary,
    pub theme: &'a Theme,
    /// Derived list marker labels keyed by semantic node id (used during arrangement).
    pub list_markers: HashMap<String, String>,
}

impl<'a> LayoutContext<'a> {
    pub fn new(fonts: &'a FontLibrary, theme: &'a Theme) -> Self {
        Self {
            fonts,
            theme,
            list_markers: HashMap::new(),
        }
    }
}

/// Represents a 2D size in fixed-point units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: Pt,
    pub height: Pt,
}

impl Size {
    pub const ZERO: Size = Size {
        width: Pt::ZERO,
        height: Pt::ZERO,
    };

    pub fn new(width: Pt, height: Pt) -> Self {
        Self { width, height }
    }
}

/// Represents a 2D point in fixed-point units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: Pt,
    pub y: Pt,
}

impl Point {
    pub const ZERO: Point = Point {
        x: Pt::ZERO,
        y: Pt::ZERO,
    };

    pub fn new(x: Pt, y: Pt) -> Self {
        Self { x, y }
    }
}

/// Constraints for layout measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizeConstraint {
    pub min: Size,
    pub max: Size,
}

impl SizeConstraint {
    pub fn new(min: Size, max: Size) -> Self {
        Self { min, max }
    }

    /// Returns a constraint that allows anything from 0 to infinity (represented by max i128 or close to it)
    pub fn infinite() -> Self {
        Self {
            min: Size::ZERO,
            max: Size {
                width: Pt(i128::MAX),
                height: Pt(i128::MAX),
            },
        }
    }

    /// Returns the closest size to `size` that satisfies the constraints.
    pub fn constrain(&self, size: Size) -> Size {
        let width = size.width.0.max(self.min.width.0).min(self.max.width.0);
        let height = size.height.0.max(self.min.height.0).min(self.max.height.0);
        Size {
            width: Pt(width),
            height: Pt(height),
        }
    }
}
