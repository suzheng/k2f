use crate::ir::{PageElement, TextAlign};
use k2f_core::{Pt, Rect};

/// Neighbor gap (millipt) that unlimited WidthOnly would eat.
/// 8pt pill stacks collide; a 12pt masthead gap still has room to grow.
const GROW_ROOM: i128 = 10_000;
/// NoBreak overset hides the whole line if we are even 1pt short — use the
/// full gap as pad. WidthOnly is still off so growth cannot cross the sibling.
const GAP_KEEP: i128 = 0;
/// Pad below this: host metrics cannot grow, so NoBreak would overset the
/// rest of the story (flush column body beside the next card). Drop NoBreak
/// and HeightOnly-grow instead of hiding lines.
const FLUSH_PAD: i128 = 2_000;
/// Fallback extra millipt when page width is unknown (`clamp_width_autosize`
/// tests): grow a little so host metrics are not 1pt overset.
const HOST_LINE_PAD: i128 = 24_000;
/// Skip the margin-grow path if the open side is thinner than this — wrapping
/// is then the only way leftover glyphs stay visible.
const MIN_OPEN: i128 = 4_000;

/// NoBreak without WidthOnly oversets (hides) the whole story when host
/// metrics are even 1pt wider than the lock box — 2-line titles, tracked
/// labels. Give those frames WidthOnly first; `clamp_width_autosize` still
/// caps growth against siblings.
pub(crate) fn apply(elements: &mut [PageElement], page_w: i128) {
    ensure_nobreak_width(elements);
    clamp_width_autosize_on(elements, Some(page_w));
}

fn ensure_nobreak_width(elements: &mut [PageElement]) {
    let rects: Vec<Rect> = elements.iter().map(|el| el.rect().clone()).collect();
    for i in 0..elements.len() {
        let PageElement::TextBox(tb) = &elements[i] else {
            continue;
        };
        if !tb.no_break || tb.autosize_width {
            continue;
        }
        // Wide left-aligned frames (authors, ABSTRACT, 2-line display titles
        // that do not fill the box) must not WidthOnly-shrink: InDesign
        // re-anchors about ItemTransform and the line looks centered.
        // Tight lock ink already set autosize_width in textbox_from_draw
        // (48pt slack). Forcing it here would shrink a loose frame.
        if matches!(tb.align, TextAlign::Left) {
            continue;
        }
        // Center/Right single-line without HeightOnly: same shrink is toward
        // the reference point and is correct. Skip only when there is no
        // extra line that HeightOnly+NoBreak would otherwise overset.
        let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
        if !tb.autosize_height && nbreaks == 0 {
            continue;
        }
        // UseNoLineBreaks on 2+ lock lines sizes the frame as one unwrapped
        // paragraph. Beside a column sibling that explodes through the gutter.
        if nbreaks >= 1 && column_sibling(&rects[i], &rects, i) {
            continue;
        }
        if let PageElement::TextBox(tb) = &mut elements[i] {
            tb.autosize_width = true;
            // Respect `\n` when sizing: longest lock line, not the unwrapped
            // paragraph. Single-line NoBreak still uses UseNoLineBreaks.
            tb.autosize_no_wrap = nbreaks == 0;
            tb.autosize_height = false;
        }
    }
}

/// WidthOnly has no max: a shrink-wrapped label beside a badge grows through
/// the sibling. NoBreak + a too-tight lock box oversets (hides) the whole
/// line. Cap growth by expanding the lock rect into the gap instead.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn clamp_width_autosize(elements: &mut [PageElement]) {
    clamp_width_autosize_on(elements, None);
}

fn clamp_width_autosize_on(elements: &mut [PageElement], page_w: Option<i128>) {
    let rects: Vec<Rect> = elements.iter().map(|el| el.rect().clone()).collect();
    for i in 0..elements.len() {
        let PageElement::TextBox(tb) = &elements[i] else {
            continue;
        };
        if !tb.autosize_width {
            continue;
        }
        let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
        let multi = tb.runs.iter().any(|r| r.text.contains('\n'));
        let me = &rects[i];
        // GROW_ROOM (10pt) misses a 12–16pt two-column gutter. WidthOnly then
        // expands about ItemTransform and the body eats the sidenote (and vice
        // versa). Any-distance side sibling: stay in the lock column.
        if (multi || nbreaks >= 1) && column_sibling(me, &rects, i) {
            if let PageElement::TextBox(tb) = &mut elements[i] {
                tb.autosize_width = false;
                tb.autosize_no_wrap = false;
                tb.autosize_height = true;
                if tb.no_break {
                    tb.no_break = false;
                    unglue_nbsp(&mut tb.runs);
                    if nbreaks >= 2 && !tb.semantic_newlines {
                        collapse_lock_newlines(&mut tb.runs);
                    }
                }
            }
            continue;
        }
        let (grow_left, grow_right) = grow_dirs(tb.autosize_refer);
        // Measure both sides even when we only grow one way: host WidthOnly
        // can still expand about ItemTransform and collide with a flush
        // neighbor (KEYWORDS: vs the list).
        let right_gap = nearest_right_gap(me, &rects, i);
        let left_gap = nearest_left_gap(me, &rects, i);
        let grow_right_gap = if grow_right { right_gap } else { None };
        let grow_left_gap = if grow_left { left_gap } else { None };
        let other_flush = match (grow_left, grow_right) {
            // Any close neighbor on the non-grow side: WidthOnly can still
            // expand about the frame center and collide (5pt KEYWORDS gap).
            (false, true) => left_gap.is_some(),
            (true, false) => right_gap.is_some(),
            _ => false,
        };
        if grow_right_gap.is_none() && grow_left_gap.is_none() && !other_flush {
            continue;
        }
        if let PageElement::TextBox(tb) = &mut elements[i] {
            tb.autosize_width = false;
            tb.autosize_no_wrap = false;
            // Positive Tracking is 1/1000 em extra per gap. On a shrink-wrapped
            // left label it is why the line is wider than the lock box and runs
            // through the badge. Isolated labels keep tracking + WidthOnly.
            let mut pad = 0i128;
            if grow_right && !grow_left {
                for run in &mut tb.runs {
                    if run.tracking > 0 {
                        run.tracking = 0;
                    }
                }
                if let Some(gap) = grow_right_gap {
                    pad = pad_for_gap(gap);
                    tb.rect.width = Pt(tb.rect.width.0 + pad);
                }
            } else if grow_left && !grow_right {
                for run in &mut tb.runs {
                    if run.tracking > 0 {
                        run.tracking = 0;
                    }
                }
                if let Some(gap) = grow_left_gap {
                    pad = pad_for_gap(gap);
                    tb.rect.x = Pt(tb.rect.x.0 - pad);
                    tb.rect.width = Pt(tb.rect.width.0 + pad);
                }
            } else {
                pad = match (grow_left_gap, grow_right_gap) {
                    (Some(l), Some(r)) => pad_for_gap(l).min(pad_for_gap(r)),
                    (Some(l), None) => pad_for_gap(l),
                    (None, Some(r)) => pad_for_gap(r),
                    (None, None) => 0,
                };
            }
            // Flush sibling: expanding 0pt leaves NoBreak lines one host-em
            // over-wide, and InDesign oversets the rest of the story. Multi-line
            // lock-pinned body is the same even with an 8pt pad — that slack
            // fits a short label, not a full-width paragraph. Allow wrap
            // (unglue NBSP) and HeightOnly so leftover words stay visible.
            let multi = tb.runs.iter().any(|r| r.text.contains('\n'));
            if tb.no_break && (pad < FLUSH_PAD || multi || other_flush) {
                let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
                // Single lock line (folio `Page 1 of 2`, article title beside a
                // badge): wrapping overprints the next row. Grow into the open
                // page margin on the unconstrained side and keep NoBreak. Do
                // not WidthOnly — ItemTransform still expands about the center
                // and would eat the pinned sibling. Multi-line lock body still
                // wraps (KEYWORDS, clause columns).
                let single = !multi && nbreaks == 0;
                if single {
                    let extra = if grow_right && !grow_left {
                        open_right(&tb.rect, &rects, i, page_w)
                    } else if grow_left && !grow_right {
                        open_left(&tb.rect, &rects, i, page_w)
                    } else {
                        0
                    };
                    if extra >= MIN_OPEN {
                        if grow_right && !grow_left {
                            tb.rect.width = Pt(tb.rect.width.0 + extra);
                        } else if grow_left && !grow_right {
                            tb.rect.x = Pt(tb.rect.x.0 - extra);
                            tb.rect.width = Pt(tb.rect.width.0 + extra);
                        }
                        continue;
                    }
                }
                tb.no_break = false;
                unglue_nbsp(&mut tb.runs);
                // 2-line display breaks (title / DEPTHS) stay. 3+ lock wraps
                // are paragraph reflow — keep them as `\n` and leftover words
                // wrap *plus* the forced break, overprinting the next card.
                // Author newlines (seal stacks) must not be flattened.
                if nbreaks >= 2 && !tb.semantic_newlines {
                    collapse_lock_newlines(&mut tb.runs);
                }
                tb.autosize_height = true;
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

fn unglue_nbsp(runs: &mut [crate::ir::TextRun]) {
    for run in runs {
        if run.auto_page_number {
            continue;
        }
        if run.text.contains('\u{00A0}') {
            run.text = run.text.replace('\u{00A0}', " ");
        }
    }
}

fn collapse_lock_newlines(runs: &mut [crate::ir::TextRun]) {
    for run in runs {
        if run.auto_page_number {
            continue;
        }
        if run.text.contains('\n') {
            run.text = run.text.replace('\n', " ");
        }
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

/// Strictly left or right, any distance. Containing chrome is not a sibling.
fn column_sibling(me: &Rect, rects: &[Rect], my_i: usize) -> bool {
    let my_left = me.x.0;
    let my_right = me.x.0 + me.width.0;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !y_overlap(me, other) || covers(other, me) || covers(me, other) {
            continue;
        }
        let o_left = other.x.0;
        let o_right = other.x.0 + other.width.0;
        if o_left >= my_right || o_right <= my_left {
            return true;
        }
    }
    false
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

/// Room to the right at any distance (not just GROW_ROOM). Page-fill boxes
/// that contain `me` are skipped via `covers`.
fn open_right(me: &Rect, rects: &[Rect], my_i: usize, page_w: Option<i128>) -> i128 {
    let my_right = me.x.0 + me.width.0;
    let mut limit = page_w.unwrap_or(my_right + HOST_LINE_PAD);
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !y_overlap(me, other) || covers(other, me) || covers(me, other) {
            continue;
        }
        if other.x.0 >= my_right {
            limit = limit.min(other.x.0);
        }
    }
    (limit - my_right).max(0)
}

fn open_left(me: &Rect, rects: &[Rect], my_i: usize, page_w: Option<i128>) -> i128 {
    let my_left = me.x.0;
    let mut limit = 0i128;
    let _ = page_w;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !y_overlap(me, other) || covers(other, me) || covers(me, other) {
            continue;
        }
        let other_right = other.x.0 + other.width.0;
        if other_right <= my_left {
            limit = limit.max(other_right);
        }
    }
    (my_left - limit).max(0)
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
            first_line_indent_pt: 0.0,
            vert_center: false,
            autosize_width: autosize,
            autosize_refer: refer,
            autosize_no_wrap: autosize,
            autosize_height: false,
            no_break: true,
            semantic_newlines: false,
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
        assert!(
            left.no_break,
            "8pt gap still NoBreak (pad is enough for a short label)"
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
    fn flush_neighbor_drops_nobreak_instead_of_overset() {
        // 0pt gap — column body flush with the next card (poster metrics).
        let mut els = vec![
            tb("body", 0, 180_000, true, "CenterLeftPoint"),
            shape(180_000, 0, 80_000, 90_000),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text = "endures\u{00A0}is\u{00A0}an\u{00A0}immense".into();
        }
        clamp_width_autosize(&mut els);
        assert!(!is_width(&els[0]), "must not WidthOnly through the card");
        assert_eq!(width(&els[0]), 180_000, "zero gap adds no pad");
        let PageElement::TextBox(body) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            !body.no_break,
            "flush sibling must allow wrap instead of overset"
        );
        assert!(
            body.autosize_height,
            "extra wrap must HeightOnly rather than clip"
        );
        assert!(
            !body.runs[0].text.contains('\u{00A0}'),
            "NBSP glue must undo so host can wrap"
        );
    }

    #[test]
    fn multiline_body_beside_card_drops_nobreak() {
        // 8pt gap is enough for a short pill, not a lock-pinned paragraph.
        let mut els = vec![
            tb("body", 0, 174_000, true, "CenterLeftPoint"),
            shape(182_000, 0, 80_000, 90_000),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text =
                "In the oceanic abyss\nextinguishes completely\nthreshold what endures"
                    .replace(' ', "\u{00A0}");
        }
        clamp_width_autosize(&mut els);
        assert!(!is_width(&els[0]));
        assert_eq!(
            width(&els[0]),
            174_000,
            "multi-line column body stays lock-width; padding into the gutter is not required"
        );
        let PageElement::TextBox(body) = &els[0] else {
            panic!("textbox")
        };
        assert!(!body.no_break, "pinned body must wrap, not overset");
        assert!(body.autosize_height);
        assert!(
            !body.runs[0].text.contains('\u{00A0}'),
            "NBSP glue must undo so host can wrap"
        );
        assert!(
            !body.runs[0].text.contains('\n'),
            "3+ lock wraps collapse so host reflows as one paragraph"
        );
    }

    #[test]
    fn two_line_title_keeps_display_break() {
        let mut els = vec![
            tb("title", 0, 174_000, true, "CenterLeftPoint"),
            shape(182_000, 0, 80_000, 40_000),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text = "RESONANCE BEYOND THE PHOTIC\nDEPTHS".into();
        }
        clamp_width_autosize(&mut els);
        let PageElement::TextBox(title) = &els[0] else {
            panic!("textbox")
        };
        assert!(!title.no_break);
        assert!(
            title.runs[0].text.contains('\n'),
            "2-line display break must not collapse"
        );
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

    #[test]
    fn wide_two_line_left_does_not_widthonly_shrink() {
        // Social-snap / paper display title: 2 semantic lines in a wide
        // left-aligned frame. WidthOnly shrinks to ink and InDesign re-anchors
        // about ItemTransform, so the title looks centered.
        let mut els = vec![tb("title", 0, 488_000, false, "TopLeftPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.no_break = true;
            t.autosize_height = true;
            t.semantic_newlines = true;
            t.runs[0].text = "THE NEXT GENERATION\nAI DESIGN CANVAS".into();
        }
        apply(&mut els, 600_000);
        let PageElement::TextBox(title) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            !title.autosize_width,
            "loose left 2-line must not WidthOnly-shrink and look centered"
        );
        assert!(title.no_break, "isolated title keeps NoBreak");
        assert!(
            title.autosize_height,
            "HeightOnly stays so extra host wrap is not clipped"
        );
    }

    #[test]
    fn center_two_line_nobreak_gets_widthonly() {
        // Center shrink toward CenterPoint is correct; host metrics still
        // need WidthOnly so HeightOnly+NoBreak does not overset.
        let mut els = vec![tb("title", 0, 499_000, false, "CenterPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.no_break = true;
            t.autosize_height = true;
            t.align = TextAlign::Center;
            t.runs[0].text = "The Architecture of Thought:\nHuman-Centric Reasoning".into();
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(title) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            title.autosize_width,
            "isolated Center NoBreak must WidthOnly so host metrics do not hide the title"
        );
        assert!(
            title.no_break,
            "isolated title keeps NoBreak with WidthOnly"
        );
        assert!(!title.autosize_height, "WidthOnly replaces HeightOnly");
        assert!(
            !title.autosize_no_wrap,
            "2-line title must size to the longest line, not the unwrapped paragraph"
        );
    }

    #[test]
    fn wide_single_line_does_not_widthonly_shrink() {
        let mut els = vec![tb("authors", 0, 499_000, false, "TopLeftPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.no_break = true;
            t.runs[0].text = "Dr. Julian Vance".into();
        }
        apply(&mut els, 595_000);
        assert!(
            !is_width(&els[0]),
            "wide authors line must not WidthOnly-shrink and re-anchor"
        );
    }

    #[test]
    fn flush_left_neighbor_stops_widthonly_from_eating_label() {
        // KEYWORDS: 5pt gap (lock) then the list. List WidthOnly from
        // CenterLeftPoint must not grow through the label.
        let mut els = vec![
            tb("label", 0, 42_806, false, "CenterLeftPoint"),
            tb("list", 47_806, 367_760, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[1] {
            t.runs[0].text = "systems\narchitecture\nlayout".into();
        }
        apply(&mut els, 595_000);
        assert!(
            !is_width(&els[1]),
            "close left sibling must disable WidthOnly on the list"
        );
        let PageElement::TextBox(list) = &els[1] else {
            panic!("textbox")
        };
        assert!(
            !list.no_break,
            "list beside a label must wrap instead of collide"
        );
        assert!(
            list.autosize_height,
            "wrapped keyword list must HeightOnly so the second line is not clipped"
        );
    }

    #[test]
    fn flush_right_folio_grows_into_page_margin_not_wrap() {
        // Tight "Page 1 of 2" ink box flush with the left footer. WidthOnly
        // would expand about ItemTransform into that footer; wrapping makes
        // two lines. Grow into the open right margin instead.
        let mut els = vec![
            shape(0, 0, 595_000, 842_000),
            tb("footer.left", 48_000, 462_666, false, "CenterLeftPoint"),
            tb("footer.right", 510_666, 36_334, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[2] {
            t.runs[0].text = "Page\u{00A0}1\u{00A0}of\u{00A0}2".into();
            t.runs[0].tracking = 0;
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(folio) = &els[2] else {
            panic!("textbox")
        };
        assert!(folio.no_break, "folio must stay one glued line");
        assert!(
            !folio.autosize_height,
            "folio must not HeightOnly-wrap onto two lines"
        );
        assert!(
            !folio.autosize_width,
            "must not WidthOnly into the left footer"
        );
        assert!(
            folio.rect.width.0 > 36_334,
            "must grow into the page margin, got {}",
            folio.rect.width.0
        );
        assert_eq!(folio.rect.x.0, 510_666, "left edge (glyph start) stays");
        assert!(
            folio.rect.x.0 + folio.rect.width.0 <= 595_000,
            "must not grow past the page"
        );
        assert!(
            folio.runs[0].text.contains('\u{00A0}'),
            "NBSP glue must stay so the folio cannot wrap"
        );
    }

    #[test]
    fn single_line_heading_beside_badge_grows_into_open_margin() {
        // Ink-tight article title (~194pt) flush with a left badge. The rest of
        // A4 is empty. Old SHORT_LINE cap (100pt) dropped NoBreak and wrapped
        // the heading onto the clause grid. Grow into the open right margin.
        let mut els = vec![
            tb("badge", 45_000, 60_000, false, "CenterLeftPoint"),
            tb("title", 105_000, 193_774, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[1] {
            t.runs[0].text = "ENGAGEMENT\u{00A0}&\u{00A0}SCOPE\u{00A0}OF\u{00A0}SERVICES".into();
            t.runs[0].tracking = 0;
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(title) = &els[1] else {
            panic!("textbox")
        };
        assert!(
            title.no_break,
            "single lock line must not wrap beside a badge"
        );
        assert!(
            !title.autosize_height,
            "heading must not HeightOnly onto the next row"
        );
        assert!(
            !title.autosize_width,
            "must not WidthOnly through the badge"
        );
        assert_eq!(title.rect.x.0, 105_000, "left edge (glyph start) stays");
        assert!(
            title.rect.width.0 > 193_774,
            "must grow into the page margin, got {}",
            title.rect.width.0
        );
        assert!(
            title.rect.x.0 + title.rect.width.0 <= 595_000,
            "must not grow past the page"
        );
        assert!(
            title.runs[0].text.contains('\u{00A0}'),
            "NBSP glue must stay so the heading cannot wrap"
        );
    }

    #[test]
    fn multiline_column_gutter_does_not_widthonly_through_sibling() {
        // 14.5pt gutter — wider than GROW_ROOM (10pt). WidthOnly +
        // UseNoLineBreaks sized the sidenote as one unwrapped line and ate
        // the body column. Stay in the lock width; HeightOnly for extra wrap.
        let mut els = vec![
            tb("sidenote", 46_500, 116_000, true, "CenterLeftPoint"),
            tb("body", 177_000, 376_000, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text =
                "Establishes canonical definitions,\ntechnical specifications, and\narchitectural boundaries"
                    .into();
        }
        if let PageElement::TextBox(t) = &mut els[1] {
            t.runs[0].text =
                "Licensed Blueprint Architecture signifies\nthe modular software kernels"
                    .into();
        }
        apply(&mut els, 595_000);
        assert!(!is_width(&els[0]), "sidenote must not WidthOnly through body");
        assert!(!is_width(&els[1]), "body must not WidthOnly through sidenote");
        assert_eq!(width(&els[0]), 116_000, "sidenote lock width stays");
        assert_eq!(width(&els[1]), 376_000, "body lock width stays");
        let PageElement::TextBox(sn) = &els[0] else {
            panic!("textbox")
        };
        let PageElement::TextBox(body) = &els[1] else {
            panic!("textbox")
        };
        assert!(sn.autosize_height);
        assert!(body.autosize_height);
        assert!(!sn.no_break);
        assert!(!body.no_break);
        assert_eq!(sn.rect.x.0, 46_500);
        assert_eq!(body.rect.x.0, 177_000);
    }

    #[test]
    fn semantic_newlines_beside_sibling_are_not_collapsed() {
        // Seal stack: author `\n` matching lock lines. A left icon is a column
        // sibling. Flattening `\n` makes one long line in a short box.
        let mut els = vec![
            tb("icon", 50_000, 30_000, false, "CenterLeftPoint"),
            tb("seal", 92_500, 187_500, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[1] {
            t.runs[0].text = "OFFICIAL CORPORATE SEAL\nAETHERIA SYSTEMS CORP.\nDELAWARE REGISTRATION"
                .into();
            t.semantic_newlines = true;
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(seal) = &els[1] else {
            panic!("textbox")
        };
        assert!(!is_width(&els[1]));
        assert!(seal.autosize_height);
        assert!(
            seal.runs[0].text.contains('\n'),
            "author newlines must survive, got {:?}",
            seal.runs[0].text
        );
        assert_eq!(seal.rect.width.0, 187_500);
    }
}
