use k2f_core::Pt;

/// Size class of a sub-formula. v1 has no separate `Text` style: `Display` is the
/// outer style and scripts step down through `Script` and `ScriptScript`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathStyle {
    Display,
    /// Inline (text-style) math: same scale as Display, but big-ops keep side limits.
    Text,
    Script,
    ScriptScript,
}

impl MathStyle {
    /// Font size at this style, as an integer fraction of the base size.
    pub fn scale(self, base: Pt) -> Pt {
        match self {
            MathStyle::Display | MathStyle::Text => base,
            MathStyle::Script => Pt(base.0 * 7 / 10),
            MathStyle::ScriptScript => Pt(base.0 * 5 / 10),
        }
    }

    /// Style used for superscripts, subscripts and fraction parts.
    pub fn script(self) -> MathStyle {
        match self {
            MathStyle::Display | MathStyle::Text => MathStyle::Script,
            MathStyle::Script | MathStyle::ScriptScript => MathStyle::ScriptScript,
        }
    }

    pub fn is_display(self) -> bool {
        matches!(self, MathStyle::Display)
    }
}
