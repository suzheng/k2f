use crate::ir::PageElement;
use k2f_core::{Pt, Rect};

/// Neighbor gap (millipt) that unlimited WidthOnly would eat.
/// 8pt pill stacks collide; a 12pt masthead gap still has room to grow.
const GROW_ROOM: i128 = 10_000;
/// NoBreak overset hides the whole line if we are even 1pt short — use the
/// full gap as pad. WidthOnly is still off so growth cannot cross the sibling.
const GAP_KEEP: i128 = 0;

/// WidthOnly has no max: a shrink-wrapped label beside a badge grows through
/// the sibling. NoBreak + a too-tight lock box oversets (hides) the whole
/// line. Cap growth by expanding the lock rect into the gap instead.
pub(crate) fn clamp_width_autosize(elements: &mut [PageElement]) {
    let rects: Vec<Rect> = elements.iter().map(|el| el.rect().clone()).collect();
    for i in 0..elements.len() {
        let PageElement::TextBox(tb) = &elements[i] else {
            continue;
        };
        if !tb.autosize_width {
            continue;
        }
        let (grow_left, grow_right) = grow_dirs(tb.autosize_refer);
        let me = &rects[i];
        let right_gap = if grow_right {
            nearest_right_gap(me, &rects, i)
        } else {
            None
        };
        let left_gap = if grow_left {
            nearest_left_gap(me, &rects, i)
        } else {
            None
        };
        if right_gap.is_none() && left_gap.is_none() {
            continue;
        }
        if let PageElement::TextBox(tb) = &mut elements[i] {
            tb.autosize_width = false;
            tb.autosize_no_wrap = false;
            // Positive Tracking is 1/1000 em extra per gap. On a shrink-wrapped
            // left label it is why the line is wider than the lock box and runs
            // through the badge. Isolated labels keep tracking + WidthOnly.
            if grow_right && !grow_left {
                for run in &mut tb.runs {
                    if run.tracking > 0 {
                        run.tracking = 0;
                    }
                }
                if let Some(gap) = right_gap {
                    tb.rect.width = Pt(tb.rect.width.0 + pad_for_gap(gap));
                }
            } else if grow_left && !grow_right {
                for run in &mut tb.runs {
                    if run.tracking > 0 {
                        run.tracking = 0;
                    }
                }
                if let Some(gap) = left_gap {
                    let pad = pad_for_gap(gap);
                    tb.rect.x = Pt(tb.rect.x.0 - pad);
                    tb.rect.width = Pt(tb.rect.width.0 + pad);
                }
            }
        }
    }
}

fn pad_for_gap(gap: i128) -> i128 {
    if gap <= GAP_KEEP {
        0
    } else {
        gap - GAP_KEEP
    }
}

fn grow_dirs(refer: &str) -> (bool, bool) {
    match refer {
        "CenterRightPoint" | "TopRightPoint" => (true, false),
        "CenterPoint" | "TopCenterPoint" => (true, true),
        _ => (false, true),
    }
}

fn covers(outer: &Rect, inner: &Rect) -> bool {
    outer.x.0 <= inner.x.0
        && outer.y.0 <= inner.y.0
        && outer.x.0 + outer.width.0 >= inner.x.0 + inner.width.0
        && outer.y.0 + outer.height.0 >= inner.y.0 + inner.height.0
}

fn y_overlap(a: &Rect, b: &Rect) -> bool {
    let a0 = a.y.0;
    let a1 = a.y.0 + a.height.0;
    let b0 = b.y.0;
    let b1 = b.y.0 + b.height.0;
    a0 < b1 && b0 < a1
}

fn blocks_grow_right(me: &Rect, other: &Rect) -> bool {
    if !y_overlap(me, other) || covers(other, me) || covers(me, other) {
        return false;
    }
    let my_right = me.x.0 + me.width.0;
    let other_left = other.x.0;
    // Gap < 0 is shadow-bleed / underlay spanning this frame, not a 10pt
    // sibling on the growth side (card rasters that clip the footer band).
    other_left >= my_right && other_left < my_right + GROW_ROOM
}

fn blocks_grow_left(me: &Rect, other: &Rect) -> bool {
    if !y_overlap(me, other) || covers(other, me) || covers(me, other) {
        return false;
    }
    let my_left = me.x.0;
    let other_right = other.x.0 + other.width.0;
    other_right <= my_left && other_right + GROW_ROOM > my_left
}

fn nearest_right_gap(me: &Rect, rects: &[Rect], my_i: usize) -> Option<i128> {
    let my_right = me.x.0 + me.width.0;
    let mut best: Option<i128> = None;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !blocks_grow_right(me, other) {
            continue;
        }
        let gap = other.x.0 - my_right;
        best = Some(best.map_or(gap, |g| g.min(gap)));
    }
    best
}

fn nearest_left_gap(me: &Rect, rects: &[Rect], my_i: usize) -> Option<i128> {
    let my_left = me.x.0;
    let mut best: Option<i128> = None;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !blocks_grow_left(me, other) {
            continue;
        }
        let gap = my_left - (other.x.0 + other.width.0);
        best = Some(best.map_or(gap, |g| g.min(gap)));
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{TextAlign, TextBox};
    use k2f_core::{Pt, Rect};

    fn tracked_run() -> crate::ir::TextRun {
        crate::ir::TextRun {
            text: "LABEL".into(),
            font_name: "Roboto".into(),
            size_pt: 8.5,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "FFFFFF".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            leading_pt: None,
            auto_page_number: false,
            tracking: 176,
        }
    }

    fn tb(id: &str, x: i128, w: i128, autosize: bool, refer: &'static str) -> PageElement {
        PageElement::TextBox(TextBox {
            node_id: id.into(),
            rect: Rect {
                x: Pt(x),
                y: Pt(0),
                width: Pt(w),
                height: Pt(16_000),
            },
            runs: vec![tracked_run()],
            align: TextAlign::Left,
            inset_top: 0.0,
            inset_left: 0.0,
            inset_bottom: 0.0,
            inset_right: 0.0,
            vert_center: false,
            autosize_width: autosize,
            autosize_refer: refer,
            autosize_no_wrap: autosize,
            autosize_height: false,
            no_break: true,
        })
    }

    fn shape(x: i128, y: i128, w: i128, h: i128) -> PageElement {
        PageElement::Shape(crate::ir::ShapeBox {
            node_id: "card".into(),
            rect: Rect {
                x: Pt(x),
                y: Pt(y),
                width: Pt(w),
                height: Pt(h),
            },
            fill_hex: None,
            fill_alpha: 255,
            corner_pt: 0.0,
            line_hex: None,
            line_w_pt: 0.0,
            line_dash: crate::ir::LineDash::Solid,
        })
    }

    fn is_width(el: &PageElement) -> bool {
        matches!(el, PageElement::TextBox(t) if t.autosize_width)
    }

    fn width(el: &PageElement) -> i128 {
        el.rect().width.0
    }

    #[test]
    fn adjacent_label_pads_into_gap_not_through_badge() {
        // 8pt gap — same class as horizontal pill stacks.
        let mut els = vec![
            tb("eyebrow", 0, 106_000, true, "CenterLeftPoint"),
            tb("badge", 114_000, 58_000, true, "CenterPoint"),
        ];
        clamp_width_autosize(&mut els);
        assert!(!is_width(&els[0]), "must not WidthOnly through the badge");
        assert!(
            !is_width(&els[1]),
            "badge CenterPoint must not grow into the label"
        );
        assert_eq!(
            width(&els[0]),
            114_000,
            "left label should expand through the 8pt gap, not past the badge"
        );
        assert_eq!(
            width(&els[1]),
            58_000,
            "centered pill chrome stays lock-sized"
        );
        let PageElement::TextBox(left) = &els[0] else {
            panic!("textbox")
        };
        assert_eq!(
            left.runs[0].tracking, 0,
            "tight stack must drop overflowing tracking"
        );
        let PageElement::TextBox(badge) = &els[1] else {
            panic!("textbox")
        };
        assert_eq!(badge.runs[0].tracking, 176, "centered pill keeps tracking");
    }

    #[test]
    fn isolated_tight_line_keeps_widthonly() {
        let mut els = vec![tb("title", 0, 200_000, true, "CenterLeftPoint")];
        clamp_width_autosize(&mut els);
        assert!(is_width(&els[0]));
        assert_eq!(width(&els[0]), 200_000);
    }

    #[test]
    fn parent_card_does_not_block() {
        let mut els = vec![
            shape(0, 0, 300_000, 200_000),
            tb("title", 20_000, 200_000, true, "CenterLeftPoint"),
        ];
        clamp_width_autosize(&mut els);
        assert!(is_width(&els[1]), "containing chrome is not a sibling");
    }

    #[test]
    fn twelve_pt_masthead_gap_keeps_widthonly() {
        let mut els = vec![
            tb("tag", 0, 200_000, true, "CenterLeftPoint"),
            tb("live", 212_000, 70_000, true, "CenterPoint"),
        ];
        clamp_width_autosize(&mut els);
        assert!(is_width(&els[0]), "12pt gap must still WidthOnly");
        assert_eq!(width(&els[0]), 200_000);
    }

    #[test]
    fn wide_gutter_keeps_widthonly() {
        // 24pt gap (hero column): host slack still has room.
        let mut els = vec![
            tb("lead", 0, 520_000, true, "CenterLeftPoint"),
            tb("hero", 544_000, 300_000, false, "CenterLeftPoint"),
        ];
        clamp_width_autosize(&mut els);
        assert!(is_width(&els[0]));
        assert_eq!(width(&els[0]), 520_000);
    }

    #[test]
    fn overlapping_card_shadow_is_not_a_side_neighbor() {
        // Effect raster bleeds into the footer band and spans across the label.
        // Negative gap is underlay, not a 10pt sibling on the growth side.
        let mut els = vec![
            shape(-50_000, 0, 300_000, 20_000),
            tb("footer", 0, 200_000, true, "CenterLeftPoint"),
        ];
        clamp_width_autosize(&mut els);
        assert!(
            is_width(&els[1]),
            "overlapping chrome must not turn off WidthOnly"
        );
        assert_eq!(width(&els[1]), 200_000);
    }
}
