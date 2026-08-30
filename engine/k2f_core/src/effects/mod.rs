//! Deterministic pixel-domain effects used by renderers.
//!
//! The core engine produces geometry in fixed-point units and a render plan that references
//! named effects (e.g., elevation shadows). Renderers can use these helpers to execute
//! effects with explicit, deterministic math (integer arithmetic + defined edge handling).

pub mod backdrop_blur;
pub mod box_blur;
