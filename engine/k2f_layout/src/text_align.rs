use crate::style::TextAlign;
use k2f_core::Pt;

/// Mutable budget of millipt to inject into U+0020 advances while placing a justified line.
pub(crate) struct JustifyBudget {
    pub extra_per: Pt,
    pub rem: i128,
}

impl JustifyBudget {
    pub(crate) fn take_space_extra(&mut self) -> Pt {
        if self.extra_per.0 == 0 && self.rem == 0 {
            return Pt::ZERO;
        }
        let mut e = self.extra_per;
        if self.rem > 0 {
            e = Pt(e.0 + 1);
            self.rem -= 1;
        }
        e
    }
}

/// Compute a deterministic x-start offset for a line of text inside a text box.
///
/// `Justify` uses the same start offset as `Start`; stretching is applied separately
/// by expanding U+0020 advances on non-final lines.
pub(crate) fn line_start_offset(align: TextAlign, node_width: Pt, line_width: Pt) -> Pt {
    if line_width.0 >= node_width.0 {
        return Pt::ZERO;
    }
    let free = node_width.0 - line_width.0;
    match align {
        TextAlign::Start | TextAlign::Justify => Pt::ZERO,
        TextAlign::Center => Pt(free / 2),
        TextAlign::End => Pt(free),
    }
}

/// Count U+0020 characters in a line (justification gap slots).
pub(crate) fn count_justify_spaces(
    line_text_parts: impl Iterator<Item = impl AsRef<str>>,
) -> usize {
    line_text_parts
        .map(|s| s.as_ref().chars().filter(|c| *c == ' ').count())
        .sum()
}

/// Extra advance per U+0020 on a justified line, plus leftover millipt for the first gaps.
///
/// Returns `(extra_per_space, remainder)` where the first `remainder` spaces also get +1 millipt.
pub(crate) fn justify_space_extras(
    align: TextAlign,
    node_width: Pt,
    line_width: Pt,
    space_count: usize,
    is_last_line: bool,
) -> (Pt, i128) {
    if !matches!(align, TextAlign::Justify) || is_last_line || space_count == 0 {
        return (Pt::ZERO, 0);
    }
    if line_width.0 >= node_width.0 {
        return (Pt::ZERO, 0);
    }
    let free = node_width.0 - line_width.0;
    (Pt(free / space_count as i128), free % space_count as i128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_start_offset_is_deterministic_and_non_negative() {
        let node_width = Pt(100);
        let line_width = Pt(40);

        assert_eq!(
            line_start_offset(TextAlign::Start, node_width, line_width),
            Pt(0)
        );
        assert_eq!(
            line_start_offset(TextAlign::Center, node_width, line_width),
            Pt(30)
        );
        assert_eq!(
            line_start_offset(TextAlign::End, node_width, line_width),
            Pt(60)
        );
        assert_eq!(
            line_start_offset(TextAlign::Justify, node_width, line_width),
            Pt(0)
        );

        // When the line is wider than the node, offset is clamped to 0.
        assert_eq!(line_start_offset(TextAlign::End, Pt(50), Pt(120)), Pt(0));
    }

    #[test]
    fn justify_extras_only_on_non_last_lines_with_spaces() {
        assert_eq!(
            justify_space_extras(TextAlign::Justify, Pt(100), Pt(40), 3, false),
            (Pt(20), 0)
        );
        assert_eq!(
            justify_space_extras(TextAlign::Justify, Pt(100), Pt(40), 3, true),
            (Pt(0), 0)
        );
        assert_eq!(
            justify_space_extras(TextAlign::Start, Pt(100), Pt(40), 3, false),
            (Pt(0), 0)
        );
        assert_eq!(
            justify_space_extras(TextAlign::Justify, Pt(100), Pt(40), 0, false),
            (Pt(0), 0)
        );
        // 61 free / 2 spaces => 30 each + remainder 1
        assert_eq!(
            justify_space_extras(TextAlign::Justify, Pt(100), Pt(39), 2, false),
            (Pt(30), 1)
        );
    }
}
