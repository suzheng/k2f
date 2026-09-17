use crate::ir::{PageElement, TextAlign, TextBox};
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
/// Lock box center within this many millipt of page center — shrink-wrapped
/// invitation/card lines centered on the sheet.
const PAGE_CENTER_TOL: i128 = 2_000;
/// Verified-badge / avatar-icon width. A 6pt gap to a 14pt icon is not a
/// column sibling: WidthOnly only grows the host-metrics delta. Padding that
/// gap and dropping WidthOnly leaves NoBreak 1pt short, and InDesign hides
/// the whole line. Pills/cards (50pt+) still clamp — see adjacent_label.
const SMALL_SIBLING: i128 = 40_000;
/// Hairline rules / column gutters are not icons. Treating a 0.6pt divider
/// as SMALL_SIBLING enabled WidthOnly and recentered left card columns.
const MIN_ICON_W: i128 = 4_000;
/// Left 2-line display titles wider than this with loose ink keep HeightOnly so
/// WidthOnly shrink does not re-anchor the block toward center.
const WIDE_LOOSE_TITLE_W: i128 = 400_000;
/// Single-line NoBreak without WidthOnly: only auto-enable WidthOnly below
/// this frame width. Wide loose frames (author lines) must not re-anchor.
const NARROW_NOBREAK_W: i128 = 300_000;
/// Side-neighbor Y overlap below this is hairline (shadow-expanded rasters
/// sitting 0.05pt under a subtitle). Treating that as a column sibling
/// blocks open_right, and NoBreak oversets (hides) the line in InDesign.
const Y_TOUCH: i128 = 2_000;

/// NoBreak without WidthOnly oversets (hides) the whole story when host
/// metrics are even 1pt wider than the lock box — 2-line titles, tracked
/// labels. Give those frames WidthOnly first; `clamp_width_autosize` still
/// caps growth against siblings.
pub(crate) fn apply(elements: &mut [PageElement], page_w: i128) {
    enable_widthonly_beside_icons(elements);
    ensure_nobreak_width(elements, page_w);
    clamp_width_autosize_on(elements, Some(page_w));
    repair_nobreak_overset(elements);
    drop_tracking_on_nobreak_multiline(elements);
}

/// Host Roboto + positive Tracking on glued lock lines oversets (hides) the
/// whole story when slack is tight. Drop tracking on multi-line NoBreak frames
/// so InDesign keeps every lock line visible.
fn drop_tracking_on_nobreak_multiline(elements: &mut [PageElement]) {
    for el in elements {
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        if !tb.no_break {
            continue;
        }
        let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
        let multi = tb.runs.iter().any(|r| r.text.contains('\n'));
        if !multi && nbreaks == 0 {
            continue;
        }
        for run in &mut tb.runs {
            if run.tracking > 0 {
                run.tracking = 0;
            }
        }
    }
}

/// Tight NoBreak display lines beside a verified badge / avatar icon need
/// WidthOnly so host Roboto does not overset and hide the whole story.
fn enable_widthonly_beside_icons(elements: &mut [PageElement]) {
    let rects: Vec<Rect> = elements.iter().map(|el| el.rect().clone()).collect();
    for i in 0..elements.len() {
        let PageElement::TextBox(tb) = &mut elements[i] else {
            continue;
        };
        if !tb.no_break || tb.autosize_width {
            continue;
        }
        let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
        let multi = tb.runs.iter().any(|r| r.text.contains('\n'));
        if multi || nbreaks > 0 {
            continue;
        }
        // Same-x column stack (card/grid copy) must keep the lock width.
        // WidthOnly re-anchors about ItemTransform and left ink looks centered.
        if stacked_lock_column_sibling(&rects[i], &rects, i) {
            continue;
        }
        if small_side_blocker(&rects[i], &rects, i, &tb.autosize_refer).is_some_and(is_icon_w) {
            tb.autosize_width = true;
            tb.autosize_no_wrap = true;
        }
    }
}

fn ensure_nobreak_width(elements: &mut [PageElement], page_w: i128) {
    let rects: Vec<Rect> = elements.iter().map(|el| el.rect().clone()).collect();
    for i in 0..elements.len() {
        let PageElement::TextBox(tb) = &elements[i] else {
            continue;
        };
        if !tb.no_break {
            continue;
        }
        // Wide left-aligned frames (authors, ABSTRACT, 2-line display titles
        // that do not fill the box) must not WidthOnly-shrink: InDesign
        // re-anchors about ItemTransform and the line looks centered.
        // Tight lock ink already set autosize_width in textbox_from_draw
        // (48pt slack). Forcing it here would shrink a loose frame. Isolated
        // tight leads still WidthOnly-grow about the center and walk off the
        // left margin — expand the lock rect into the open right instead.
        // Column siblings stay WidthOnly so clamp can wrap in the lock column.
        if matches!(tb.align, TextAlign::Left) {
            if stacked_graphic_caption(&rects[i], &rects, i)
                && !stacked_lock_column_sibling(&rects[i], &rects, i)
            {
                if let PageElement::TextBox(tb) = &mut elements[i] {
                    tb.autosize_width = true;
                    tb.autosize_no_wrap = true;
                    tb.autosize_refer = "CenterPoint";
                }
                continue;
            }
            // Left column stack: WidthOnly shrinks about ItemTransform and
            // flush-left card/grid copy looks centered (color-block).
            if tb.autosize_width && stacked_lock_column_sibling(&rects[i], &rects, i) {
                if let PageElement::TextBox(tb) = &mut elements[i] {
                    tb.autosize_width = false;
                    tb.autosize_no_wrap = false;
                }
            }
            let PageElement::TextBox(tb) = &elements[i] else {
                continue;
            };
            if tb.autosize_width && !column_sibling(&rects[i], &rects, i) {
                let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
                let multi = tb.runs.iter().any(|r| r.text.contains('\n'));
                if tb.no_break
                    && !multi
                    && nbreaks == 0
                    && small_side_blocker(&rects[i], &rects, i, &tb.autosize_refer)
                        .is_some_and(is_icon_w)
                {
                    continue;
                }
                if stacked_lock_column_sibling(&rects[i], &rects, i)
                    || stacked_right_edge_sibling(&rects[i], &rects, i)
                {
                    continue;
                }
                if is_page_centered(&rects[i], page_w) {
                    if let PageElement::TextBox(tb) = &mut elements[i] {
                        apply_page_center_autosize(tb);
                    }
                    continue;
                }
                let extra = open_right(&rects[i], &rects, i, Some(page_w));
                if let PageElement::TextBox(tb) = &mut elements[i] {
                    tb.autosize_width = false;
                    tb.autosize_no_wrap = false;
                    if extra >= MIN_OPEN {
                        tb.rect.width = Pt(tb.rect.width.0 + extra);
                    }
                    // textbox_from_draw sets autosize_height=false whenever
                    // autosize_width was true. Restoring HeightOnly here keeps
                    // multi-line NoBreak body from oversetting in InDesign.
                    if (multi || nbreaks >= 1) && !tb.vert_center {
                        tb.autosize_height = true;
                    }
                }
            }
            if let PageElement::TextBox(tb) = &mut elements[i] {
                let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
                let multi = tb.runs.iter().any(|r| r.text.contains('\n'));
                // Use the live rect: open_right above may have widened the frame;
                // a stale snapshot still looks "narrow" and wrongly re-enables
                // WidthOnly (letter recipient / closing lines center in InDesign).
                let live_rect = tb.rect.clone();
                repair_left_nobreak(tb, &live_rect, &rects, i, page_w, multi, nbreaks);
            }
            continue;
        }
        // Full-width paper titles with author `\n`: WidthOnly + CenterPoint
        // concatenates semantic paragraphs onto one clipped line in InDesign.
        // HeightOnly + drop NoBreak lets each author line stack using Leading.
        if tb.semantic_newlines
            && tb.lock_line_count >= 2
            && matches!(tb.align, TextAlign::Center)
            && rects[i].width.0 > WIDE_LOOSE_TITLE_W
            && !column_sibling(&rects[i], &rects, i)
        {
            if let PageElement::TextBox(tb) = &mut elements[i] {
                tb.autosize_width = false;
                tb.autosize_no_wrap = false;
                tb.autosize_height = true;
                drop_nobreak(tb);
            }
            continue;
        }
        if tb.autosize_width {
            continue;
        }
        // Center/Right single-line without HeightOnly: WidthOnly-grow so host
        // metrics do not overset and hide the story.
        let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
        if !tb.autosize_height && nbreaks == 0 && !tb.vert_center {
            if let PageElement::TextBox(tb) = &mut elements[i] {
                tb.autosize_width = true;
                tb.autosize_no_wrap = true;
                tb.autosize_height = false;
            }
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

/// Left NoBreak frames that never got WidthOnly: host metrics overset the lock
/// box and InDesign hides the story (letter recipient lines, body paragraphs).
fn repair_left_nobreak(
    tb: &mut crate::ir::TextBox,
    me: &Rect,
    rects: &[Rect],
    my_i: usize,
    page_w: i128,
    multi: bool,
    nbreaks: usize,
) {
    if !tb.no_break {
        return;
    }
    if tb.autosize_height && !tb.autosize_width && tb.lock_line_count >= 2 {
        if (multi || nbreaks >= 1) && !column_sibling(me, rects, my_i) && !tb.semantic_newlines {
            if should_widthonly_two_line_nobreak(tb, nbreaks) {
                tb.autosize_width = true;
                tb.autosize_no_wrap = false;
                tb.autosize_height = false;
            } else {
                drop_nobreak(tb);
                for run in &mut tb.runs {
                    if run.tracking > 0 {
                        run.tracking = 0;
                    }
                }
            }
        }
        return;
    }
    if tb.autosize_width || tb.autosize_height || multi || nbreaks > 0 {
        return;
    }
    if me.width.0 >= NARROW_NOBREAK_W {
        // Wide hug one-liners (532pt slide subtitles) skip WidthOnly so the
        // frame does not re-anchor, but still need host-metric room when the
        // lock ink already fills the box. Loose wide headers must stay put.
        if tb.full_width_lock_line
            && tb.lock_line_count <= 1
            && !stacked_lock_column_sibling(me, rects, my_i)
            && !stacked_right_edge_sibling(me, rects, my_i)
        {
            let extra = open_right(me, rects, my_i, Some(page_w));
            if extra >= MIN_OPEN {
                tb.rect.width = Pt(tb.rect.width.0 + extra);
            }
        }
        return;
    }
    if stacked_lock_column_sibling(me, rects, my_i) || stacked_right_edge_sibling(me, rects, my_i)
    {
        if is_page_centered(me, page_w) && !stacked_lock_column_sibling(me, rects, my_i) {
            apply_page_center_autosize(tb);
            return;
        }
        let extra = open_right(me, rects, my_i, Some(page_w));
        if extra >= MIN_OPEN {
            tb.rect.width = Pt(tb.rect.width.0 + extra);
        } else {
            drop_nobreak(tb);
        }
        return;
    }
    tb.autosize_width = true;
    tb.autosize_no_wrap = true;
}

/// Final pass: HeightOnly + NoBreak without WidthOnly still oversets in InDesign
/// when lock-pinned lines are wider than host metrics expect.
fn repair_nobreak_overset(elements: &mut [PageElement]) {
    let rects: Vec<Rect> = elements.iter().map(|el| el.rect().clone()).collect();
    for i in 0..elements.len() {
        let PageElement::TextBox(tb) = &mut elements[i] else {
            continue;
        };
        if !tb.no_break || tb.autosize_width {
            continue;
        }
        let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
        let multi = tb.runs.iter().any(|r| r.text.contains('\n'));
        // Author affiliation / title stacks: semantic `\n` paragraphs with
        // HeightOnly. NoBreak on every paragraph oversets and hides the story.
        if tb.semantic_newlines
            && tb.autosize_height
            && tb.lock_line_count >= 2
            && !column_sibling(&rects[i], &rects, i)
        {
            drop_nobreak(tb);
            continue;
        }
        if tb.autosize_height
            && tb.lock_line_count >= 2
            && (multi || nbreaks >= 1)
            && !column_sibling(&rects[i], &rects, i)
            && !tb.semantic_newlines
            && !should_widthonly_two_line_nobreak(tb, nbreaks)
        {
            drop_nobreak(tb);
            for run in &mut tb.runs {
                if run.tracking > 0 {
                    run.tracking = 0;
                }
            }
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
            // Extra wrap + HeightOnly overprints a stacked sibling a few pt
            // below (grid card body → CTA). Keep lock lines glued and grow
            // into the open right margin instead of dropping NoBreak.
            let stacked = nearest_below_gap(me, &rects, i).is_some_and(|g| g < GROW_ROOM);
            if stacked {
                let extra = open_right(me, &rects, i, page_w);
                if let PageElement::TextBox(tb) = &mut elements[i] {
                    // Grid column copy on the page midline stays left-aligned;
                    // page_center_autosize is for isolated centered pills only.
                    tb.autosize_width = false;
                    tb.autosize_no_wrap = false;
                    tb.autosize_height = false;
                    if extra >= MIN_OPEN {
                        tb.rect.width = Pt(tb.rect.width.0 + extra);
                    }
                }
                continue;
            }
            if let PageElement::TextBox(tb) = &mut elements[i] {
                tb.autosize_width = false;
                tb.autosize_no_wrap = false;
                tb.autosize_height = true;
                if tb.no_break {
                    drop_nobreak(tb);
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
        // Single lock line beside a small icon: keep WidthOnly. The 6pt gap
        // to a verified badge used to disable it, pad 6pt, and keep NoBreak
        // — host Roboto then overset and hid "KRONOS // NIGHTFALL".
        if tb.no_break && !multi && nbreaks == 0 {
            if small_side_blocker(me, &rects, i, &tb.autosize_refer)
                .is_some_and(is_icon_w)
            {
                continue;
            }
            if stacked_lock_column_sibling(me, &rects, i)
                || stacked_right_edge_sibling(me, &rects, i)
            {
                continue;
            }
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
                        if page_w.is_some_and(|pw| is_page_centered(&tb.rect, pw))
                            && !column_sibling(&tb.rect, &rects, i)
                        {
                            apply_page_center_autosize(tb);
                        } else if grow_right && !grow_left {
                            tb.rect.width = Pt(tb.rect.width.0 + extra);
                        } else if grow_left && !grow_right {
                            tb.rect.x = Pt(tb.rect.x.0 - extra);
                            tb.rect.width = Pt(tb.rect.width.0 + extra);
                        }
                        continue;
                    }
                }
                if should_widthonly_two_line_nobreak(tb, nbreaks) {
                    tb.autosize_width = true;
                    tb.autosize_no_wrap = false;
                    tb.autosize_height = false;
                } else {
                    // 2-line display breaks (title / DEPTHS) stay. Column wrap
                    // (full-width or 3+ lock lines) collapses so leftover words
                    // do not wrap *plus* the forced lock break.
                    drop_nobreak(tb);
                    tb.autosize_height = true;
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

fn is_page_centered(me: &Rect, page_w: i128) -> bool {
    // Full-width column bodies share the page midpoint but are not centered pills.
    if me.width.0 * 10 >= page_w * 8 {
        return false;
    }
    let cx = me.x.0 + me.width.0 / 2;
    (cx - page_w / 2).abs() <= PAGE_CENTER_TOL
}

/// Shrink-wrapped lines centered on the page: ink fills the lock box so
/// alignment inference says Left, but the box itself is midline. Asymmetric
/// open_right expansion walks ItemTransform off center in InDesign.
fn apply_page_center_autosize(tb: &mut TextBox) {
    tb.align = TextAlign::Center;
    tb.autosize_refer = "CenterPoint";
    tb.autosize_width = true;
    tb.autosize_no_wrap = true;
    tb.autosize_height = false;
}

/// Two-line NoBreak display copy: keep WidthOnly so host metrics size to the
/// longest lock line instead of HeightOnly re-wrapping inside a line.
fn should_widthonly_two_line_nobreak(tb: &TextBox, nbreaks: usize) -> bool {
    if tb.lock_line_count != 2 || nbreaks != tb.lock_line_count.saturating_sub(1) {
        return false;
    }
    // Wrapped bibliography / column body: first lock line fills the frame.
    // WidthOnly re-anchors and misaligns the entry (academic-serif-classic ref3).
    if tb.full_width_lock_line {
        return false;
    }
    if !tb.semantic_newlines {
        return true;
    }
    // Author `\n` in a wide frame must not WidthOnly-shrink (re-anchors centered).
    tb.rect.width.0 <= WIDE_LOOSE_TITLE_W
}

/// Drop NoBreak and flatten lock-pinned wrap breaks so the host reflows at
/// the actual frame width. Semantic `\n` (titles, seal stacks) stay.
fn drop_nobreak(tb: &mut TextBox) {
    let nbreaks: usize = tb.runs.iter().map(|r| r.text.matches('\n').count()).sum();
    tb.no_break = false;
    unglue_nbsp(&mut tb.runs);
    if should_collapse_lock_newlines(tb, nbreaks) {
        collapse_lock_newlines(&mut tb.runs);
    }
}

/// Flatten lock-pinned wrap breaks when the host is allowed to reflow.
/// Keep a designed 2-line display break (title / DEPTHS). Collapse column
/// wrap — including left-aligned body whose `\n` count matches lock lines —
/// so a narrower IDML frame does not wrap *and* keep the old break points.
fn should_collapse_lock_newlines(tb: &TextBox, nbreaks: usize) -> bool {
    if tb.semantic_newlines || nbreaks == 0 {
        return false;
    }
    if nbreaks == 1 && !tb.full_width_lock_line {
        return false;
    }
    true
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
    for run in runs.iter_mut() {
        if run.auto_page_number {
            continue;
        }
        if run.text.contains('\n') {
            run.text = run.text.replace('\n', " ");
        }
    }
    // `insert_lock_breaks` puts `\n` before the next line's first glyph, so
    // "lock \ntypefaces" is common. Replacing `\n` with a space would leave
    // a double word-gap in the reflowed paragraph.
    let mut prev_space = false;
    for run in runs.iter_mut() {
        if run.auto_page_number {
            continue;
        }
        let mut out = String::with_capacity(run.text.len());
        for ch in run.text.chars() {
            let is_space = ch == ' ' || ch == '\u{00A0}';
            if is_space {
                if prev_space {
                    continue;
                }
                out.push(' ');
                prev_space = true;
            } else {
                prev_space = false;
                out.push(ch);
            }
        }
        run.text = out;
    }
}

fn small_side_blocker(me: &Rect, rects: &[Rect], my_i: usize, refer: &str) -> Option<i128> {
    let (grow_left, grow_right) = grow_dirs(refer);
    match (grow_left, grow_right) {
        (false, true) => nearest_right_blocker_w(me, rects, my_i),
        (true, false) => nearest_left_blocker_w(me, rects, my_i),
        (true, true) => nearest_right_blocker_w(me, rects, my_i)
            .or_else(|| nearest_left_blocker_w(me, rects, my_i)),
        (false, false) => None,
    }
}

fn grow_dirs(refer: &str) -> (bool, bool) {
    match refer {
        "CenterRightPoint" | "TopRightPoint" => (true, false),
        "CenterPoint" | "TopCenterPoint" => (true, true),
        _ => (false, true),
    }
}

fn is_icon_w(w: i128) -> bool {
    w > MIN_ICON_W && w <= SMALL_SIBLING
}

fn covers(outer: &Rect, inner: &Rect) -> bool {
    outer.x.0 <= inner.x.0
        && outer.y.0 <= inner.y.0
        && outer.x.0 + outer.width.0 >= inner.x.0 + inner.width.0
        && outer.y.0 + outer.height.0 >= inner.y.0 + inner.height.0
}

fn y_overlap(a: &Rect, b: &Rect) -> bool {
    y_overlap_at(a, b, 1)
}

fn y_overlap_at(a: &Rect, b: &Rect, min: i128) -> bool {
    let a0 = a.y.0;
    let a1 = a.y.0 + a.height.0;
    let b0 = b.y.0;
    let b1 = b.y.0 + b.height.0;
    a1.min(b1) - a0.max(b0) >= min
}

fn x_overlap(a: &Rect, b: &Rect) -> bool {
    let a0 = a.x.0;
    let a1 = a.x.0 + a.width.0;
    let b0 = b.x.0;
    let b1 = b.x.0 + b.width.0;
    a0 < b1 && b0 < a1
}

fn vertical_stack_gap(a: &Rect, b: &Rect) -> Option<i128> {
    if a.y.0 + a.height.0 <= b.y.0 {
        Some(b.y.0 - (a.y.0 + a.height.0))
    } else if b.y.0 + b.height.0 <= a.y.0 {
        Some(a.y.0 - (b.y.0 + b.height.0))
    } else {
        None
    }
}

/// Stacked frames that share the same right edge (x+width) but may differ in
/// left edge / width — common in right-aligned signature blocks. Growing one
/// tight NoBreak line into the page margin shifts ItemTransform right and walks
/// past the lines above.
fn stacked_right_edge_sibling(me: &Rect, rects: &[Rect], my_i: usize) -> bool {
    let my_right = me.x.0 + me.width.0;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || other.x.0 + other.width.0 != my_right {
            continue;
        }
        if !x_overlap(me, other) {
            continue;
        }
        if vertical_stack_gap(me, other).is_some_and(|g| g < GROW_ROOM) {
            return true;
        }
    }
    false
}

/// Stacked frames that share the same lock column (x/width). Growing one tight
/// NoBreak line into the page margin would walk its right edge past siblings
/// (signature name/title/org in one end block).
/// Short label stacked under a taller graphic with the same horizontal center
/// (QR caption, image footnotes). WidthOnly + CenterPoint keeps host metrics
/// centered; expanding the lock rect leaves left-aligned ink off-center.
fn stacked_graphic_caption(me: &Rect, rects: &[Rect], my_i: usize) -> bool {
    if me.height.0 > 12_000 {
        return false;
    }
    let my_cx = me.x.0 + me.width.0 / 2;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || other.height.0 < me.height.0 * 3 {
            continue;
        }
        let other_cx = other.x.0 + other.width.0 / 2;
        if (other_cx - my_cx).abs() > 2_000 {
            continue;
        }
        if other.y.0 + other.height.0 <= me.y.0
            && vertical_stack_gap(other, me).is_some_and(|g| g < GROW_ROOM)
        {
            return true;
        }
    }
    false
}

fn stacked_lock_column_sibling(me: &Rect, rects: &[Rect], my_i: usize) -> bool {
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || me.x.0 != other.x.0 || me.width.0 != other.width.0 {
            continue;
        }
        if !x_overlap(me, other) {
            continue;
        }
        if vertical_stack_gap(me, other).is_some_and(|g| g < GROW_ROOM) {
            return true;
        }
    }
    false
}

/// Gap to the nearest stacked sibling (not containing chrome). `None` if the
/// rest of the column below `me` is empty.
fn nearest_below_gap(me: &Rect, rects: &[Rect], my_i: usize) -> Option<i128> {
    let my_bottom = me.y.0 + me.height.0;
    let mut best: Option<i128> = None;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !x_overlap(me, other) || covers(other, me) || covers(me, other) {
            continue;
        }
        let o_top = other.y.0;
        if o_top >= my_bottom {
            let gap = o_top - my_bottom;
            best = Some(best.map_or(gap, |g| g.min(gap)));
        }
    }
    best
}

/// Strictly left or right, any distance. Containing chrome is not a sibling.
fn column_sibling(me: &Rect, rects: &[Rect], my_i: usize) -> bool {
    let my_left = me.x.0;
    let my_right = me.x.0 + me.width.0;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !y_overlap_at(me, other, Y_TOUCH) || covers(other, me) || covers(me, other)
        {
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

fn nearest_right_blocker_w(me: &Rect, rects: &[Rect], my_i: usize) -> Option<i128> {
    let my_right = me.x.0 + me.width.0;
    let mut best: Option<(i128, i128)> = None;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !blocks_grow_right(me, other) {
            continue;
        }
        let gap = other.x.0 - my_right;
        let w = other.width.0;
        best = Some(match best {
            None => (gap, w),
            Some((g, _)) if gap < g => (gap, w),
            Some((g, ow)) if gap == g => (g, ow.min(w)),
            Some(prev) => prev,
        });
    }
    best.map(|(_, w)| w)
}

fn nearest_left_blocker_w(me: &Rect, rects: &[Rect], my_i: usize) -> Option<i128> {
    let my_left = me.x.0;
    let mut best: Option<(i128, i128)> = None;
    for (j, other) in rects.iter().enumerate() {
        if j == my_i || !blocks_grow_left(me, other) {
            continue;
        }
        let gap = my_left - (other.x.0 + other.width.0);
        let w = other.width.0;
        best = Some(match best {
            None => (gap, w),
            Some((g, _)) if gap < g => (gap, w),
            Some((g, ow)) if gap == g => (g, ow.min(w)),
            Some(prev) => prev,
        });
    }
    best.map(|(_, w)| w)
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
        if j == my_i || !y_overlap_at(me, other, Y_TOUCH) || covers(other, me) || covers(me, other)
        {
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
        if j == my_i || !y_overlap_at(me, other, Y_TOUCH) || covers(other, me) || covers(me, other)
        {
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
            face_style: "Regular".into(),
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
            left_indent_pt: 0.0,
            vert_center: false,
            autosize_width: autosize,
            autosize_refer: refer,
            autosize_no_wrap: autosize,
            autosize_height: false,
            no_break: true,
            semantic_newlines: false,
            lock_line_count: 1,
            first_baseline_leading_offset: false,
            full_width_lock_line: false,
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
    fn tight_line_beside_icon_keeps_widthonly() {
        // Glow-feed profile name: 149pt ink-tight line, 6pt gap, 14pt badge.
        // Padding the gap and dropping WidthOnly oversets (hides) the name.
        let mut els = vec![
            tb("profile.name", 102_000, 149_342, true, "CenterLeftPoint"),
            shape(257_342, 0, 14_000, 14_000),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text = "KRONOS\u{00A0}//\u{00A0}NIGHTFALL".into();
            t.runs[0].tracking = 0;
            t.runs[0].size_pt = 14.5;
            t.runs[0].bold = true;
        }
        apply(&mut els, 600_000);
        assert!(
            is_width(&els[0]),
            "icon sibling must not disable WidthOnly on a tight display name"
        );
        assert_eq!(
            width(&els[0]),
            149_342,
            "lock rect stays; InDesign WidthOnly grows only the host delta"
        );
        let PageElement::TextBox(name) = &els[0] else {
            panic!("textbox")
        };
        assert!(name.no_break, "name must stay one glued line");
        assert!(
            !name.autosize_height,
            "must not HeightOnly-wrap onto the handle row"
        );
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
    fn left_column_body_with_matching_lock_lines_collapses() {
        // Production lock_line_count matches wrap lines. The old exception
        // kept those `\n` so InDesign wrapped at the lock points *and* at the
        // (different) frame width.
        let mut els = vec![
            tb("body", 0, 174_000, true, "CenterLeftPoint"),
            shape(182_000, 0, 80_000, 90_000),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.lock_line_count = 3;
            t.full_width_lock_line = true;
            t.runs[0].text =
                "In the oceanic abyss\nextinguishes completely\nthreshold what endures"
                    .replace(' ', "\u{00A0}");
        }
        clamp_width_autosize(&mut els);
        let PageElement::TextBox(body) = &els[0] else {
            panic!("textbox")
        };
        assert!(!body.no_break, "column body must wrap, not overset");
        assert!(
            !body.runs[0].text.contains('\n'),
            "matching lock-line wraps must still collapse, got {:?}",
            body.runs[0].text
        );
        assert!(
            !body.runs[0].text.contains("  "),
            "collapsed lock breaks must not leave double spaces, got {:?}",
            body.runs[0].text
        );
    }

    #[test]
    fn collapsed_lock_breaks_squeeze_space_before_newline() {
        // Pin inserts `\n` at the next line's first glyph, so the lock space
        // stays: "lock \ntypefaces". Collapse must not become "lock  typefaces".
        let mut els = vec![
            tb("body", 0, 174_000, true, "CenterLeftPoint"),
            shape(182_000, 0, 80_000, 90_000),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text =
                "Deterministic micro-increments lock \ntypefaces and containers into \nharmonious"
                    .into();
        }
        clamp_width_autosize(&mut els);
        let PageElement::TextBox(body) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            !body.runs[0].text.contains('\n'),
            "3+ lock wraps collapse"
        );
        assert!(
            !body.runs[0].text.contains("  "),
            "must squeeze space+newline, got {:?}",
            body.runs[0].text
        );
        assert!(
            body.runs[0].text.contains("lock typefaces"),
            "got {:?}",
            body.runs[0].text
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
    fn center_semantic_two_line_title_keeps_heightonly() {
        // academic-serif-classic paper.title: two semantic paragraphs, center.
        let mut els = vec![tb("paper.title", 0, 499_000, false, "TopCenterPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.no_break = true;
            t.autosize_height = true;
            t.semantic_newlines = true;
            t.lock_line_count = 2;
            t.align = TextAlign::Center;
            t.runs[0].text =
                "Deterministic Semantic Layout Compilation\nfor Academic Publishing Systems"
                    .into();
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(title) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            !title.autosize_width,
            "WidthOnly concatenates the two title lines into one overflow"
        );
        assert!(title.autosize_height);
        assert!(
            !title.no_break,
            "NoBreak+HeightOnly oversets and hides the title"
        );
        assert!(
            title.runs[0].text.contains('\n'),
            "author newline must stay"
        );
    }

    #[test]
    fn affiliation_stack_drops_nobreak() {
        let mut els = vec![tb("paper.affiliations", 0, 499_000, false, "TopLeftPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.no_break = true;
            t.autosize_height = true;
            t.semantic_newlines = true;
            t.lock_line_count = 4;
            t.runs[0].text =
                "1 Department of Computer Science\n2 Laboratory for Information\n3 Institute for Automated Reasoning\n*Correspondence: a@b.c"
                    .into();
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(aff) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            !aff.no_break,
            "4-line affiliation stack must not NoBreak+HeightOnly overset"
        );
        assert!(aff.autosize_height);
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
    fn tight_letter_recipient_line_grows_not_widthonly() {
        // blush-floral-letterhead: ink-tight recipient name at the left margin.
        // open_right widens the frame; WidthOnly on that wide box re-anchors
        // the line to the page center in InDesign.
        let mut els = vec![tb(
            "letter.recipient.name",
            54_000,
            102_090,
            true,
            "CenterLeftPoint",
        )];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text = "Dr. Evelyn Montgomery".into();
            t.runs[0].tracking = 0;
            t.runs[0].bold = true;
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(name) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            !name.autosize_width,
            "margin-grown recipient line must not WidthOnly-shrink"
        );
        assert!(
            name.rect.width.0 > 102_090,
            "frame grows into the open right margin for host metrics"
        );
        assert!(name.no_break, "single-line salutation keeps NoBreak");
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
    fn wide_hug_subtitle_grows_past_hairline_card_raster() {
        // aurora-data slide.02.desc: 532pt ink-tight NoBreak one-liner. A
        // shadow-expanded card raster sits 0.05pt under it, which used to
        // count as a column sibling and freeze the lock width. Host metrics
        // then overset and InDesign hides the subtitle.
        let mut desc = tb("slide.02.desc", 56_000, 532_052, false, "CenterLeftPoint");
        if let PageElement::TextBox(t) = &mut desc {
            t.rect.y = Pt(89_600);
            t.rect.height = Pt(14_950);
            t.runs[0].text = "Real-time\u{00A0}diffusion\u{00A0}sampling\u{00A0}across\u{00A0}38,000\u{00A0}ligand-target\u{00A0}pairs\u{00A0}with\u{00A0}physical\u{00A0}thermodynamic\u{00A0}constraints.".into();
            t.runs[0].tracking = 0;
            t.runs[0].size_pt = 11.5;
            t.full_width_lock_line = true;
            t.lock_line_count = 1;
        }
        let mut els = vec![
            desc,
            shape(56_000, 104_500, 268_000, 310_000),
            shape(342_000, 104_500, 268_000, 310_000),
            shape(628_000, 104_500, 268_000, 310_000),
        ];
        apply(&mut els, 960_000);
        let PageElement::TextBox(desc) = &els[0] else {
            panic!("textbox")
        };
        assert!(desc.no_break, "subtitle must stay one glued line");
        assert!(
            !desc.autosize_width,
            "wide hug line must not WidthOnly-reanchor"
        );
        assert_eq!(desc.rect.x.0, 56_000, "left edge stays");
        assert!(
            desc.rect.width.0 > 532_052,
            "must grow into the open header band, got {}",
            desc.rect.width.0
        );
        assert!(
            desc.rect.x.0 + desc.rect.width.0 <= 960_000,
            "must not grow past the page"
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

    #[test]
    fn isolated_left_tight_lead_grows_right_not_widthonly() {
        // Tight single-line lead (editorial hero). WidthOnly grows about
        // ItemTransform and the line walks off the left page margin.
        let mut els = vec![tb(
            "hero.lead",
            26_000,
            454_000,
            true,
            "CenterLeftPoint",
        )];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text =
                "A serialized field guide exploring mathematical proportion, deliberate whitespace"
                    .into();
            t.runs[0].tracking = 0;
        }
        apply(&mut els, 600_000);
        let PageElement::TextBox(lead) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            !lead.autosize_width,
            "isolated left lead must not WidthOnly-reanchor"
        );
        assert!(lead.no_break, "lead must stay one glued line");
        assert_eq!(lead.rect.x.0, 26_000, "left margin stays");
        assert!(
            lead.rect.width.0 > 454_000,
            "must grow into the open page, got {}",
            lead.rect.width.0
        );
        assert!(
            lead.rect.x.0 + lead.rect.width.0 <= 600_000,
            "must not grow past the page"
        );
        assert!(!lead.autosize_height);
    }

    #[test]
    fn stacked_right_edge_block_keeps_lock_width() {
        // Right-aligned end block: each line has a different width but the
        // same right edge. open_right would shift ItemTransform right.
        let right = 384_000i128;
        let mut els = vec![
            tb("closing", right - 238_686, 238_686, true, "CenterLeftPoint"),
            tb("signature", right - 243_061, 243_061, true, "CenterLeftPoint"),
            tb("meta", right - 194_842, 194_842, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.rect.y = Pt(488_800);
            t.rect.height = Pt(17_250);
            t.runs[0].text = "With our warmest love & heartfelt appreciation,".into();
        }
        if let PageElement::TextBox(t) = &mut els[1] {
            t.rect.y = Pt(511_050);
            t.rect.height = Pt(24_700);
            t.runs[0].text = "Julian & Vivienne Sterling".into();
            t.runs[0].bold = true;
        }
        if let PageElement::TextBox(t) = &mut els[2] {
            t.rect.y = Pt(540_750);
            t.rect.height = Pt(13_300);
            t.runs[0].text = "The Sterling Celebration · Autumn 2026".into();
        }
        apply(&mut els, 420_000);
        for (i, (id, w)) in [
            ("closing", 238_686),
            ("signature", 243_061),
            ("meta", 194_842),
        ]
        .iter()
        .enumerate()
        {
            let PageElement::TextBox(tb) = &els[i] else {
                panic!("textbox")
            };
            assert_eq!(tb.node_id, *id);
            assert_eq!(tb.rect.width.0, *w, "{id} lock width");
            assert_eq!(tb.rect.x.0 + tb.rect.width.0, right, "{id} right edge");
            assert!(tb.autosize_width, "{id} keeps WidthOnly for host slack");
        }
    }

    #[test]
    fn stacked_signature_meta_keeps_lock_width() {
        // End-block stack: name/title/org share one lock column. The longest
        // line must not grow into the page margin or its right edge walks past
        // the lines above.
        let w = 232_259i128;
        let x = 310_741i128;
        let mut els = vec![
            tb("sig.name", x, w, true, "CenterLeftPoint"),
            tb("sig.title", x, w, true, "CenterLeftPoint"),
            tb("sig.org", x, w, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.rect.y = Pt(657_150);
            t.rect.height = Pt(17_550);
            t.runs[0].text = "Arthur C. Pendelton".into();
        }
        if let PageElement::TextBox(t) = &mut els[1] {
            t.rect.y = Pt(678_700);
            t.rect.height = Pt(14_000);
            t.runs[0].text = "Executive Creative Director & Founder".into();
            t.runs[0].tracking = 30;
        }
        if let PageElement::TextBox(t) = &mut els[2] {
            t.rect.y = Pt(696_700);
            t.rect.height = Pt(13_300);
            t.runs[0].text =
                "The Heritage Botanical Studio · September 15, 2026".into();
            t.runs[0].tracking = 31;
        }
        apply(&mut els, 595_000);
        for (i, id) in ["sig.name", "sig.title", "sig.org"].iter().enumerate() {
            let PageElement::TextBox(tb) = &els[i] else {
                panic!("textbox")
            };
            assert_eq!(tb.node_id, *id);
            assert_eq!(tb.rect.x.0, x, "{id} left edge");
            assert_eq!(tb.rect.width.0, w, "{id} lock width");
            assert!(tb.autosize_width, "{id} keeps WidthOnly for host slack");
        }
    }

    #[test]
    fn multiline_nobreak_drops_tracking() {
        let mut els = vec![tb("letter.p2", 42_000, 510_000, false, "CenterLeftPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text = "line one\nline two".into();
            t.runs[0].tracking = 13;
            t.no_break = true;
            t.autosize_height = true;
            t.lock_line_count = 2;
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(body) = &els[0] else {
            panic!("textbox")
        };
        assert_eq!(body.runs[0].tracking, 0);
        assert!(
            body.no_break,
            "2-line lock-pinned display copy must keep NoBreak so host cannot extra-wrap"
        );
        assert!(
            body.autosize_width,
            "must WidthOnly-grow to the longest lock line"
        );
    }

    #[test]
    fn lock_pinned_article_title_keeps_nobreak_widthonly() {
        let mut els = vec![tb("doc.title", 0, 499_000, false, "TopLeftPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.no_break = true;
            t.autosize_height = true;
            t.lock_line_count = 2;
            t.runs[0].text =
                "The Architecture of Thought: Foundations of Latent\nRepresentation and Human-Centric Reasoning"
                    .into();
            t.runs[0].size_pt = 19.0;
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(title) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            title.no_break,
            "lock-pinned title must keep NoBreak, got unglued {:?}",
            title.runs[0].text
        );
        assert!(
            title.autosize_width,
            "must WidthOnly so host metrics do not insert a third wrap"
        );
        assert!(
            !title.autosize_height,
            "WidthOnly replaces HeightOnly on 2-line titles"
        );
        assert!(
            title.runs[0].text.contains('\n'),
            "lock break must stay"
        );
    }

    #[test]
    fn wide_multiline_left_restores_heightonly_when_widthonly_cleared() {
        // Letter body: tight ink triggers WidthOnly, then ensure_nobreak_width
        // grows into the page margin and drops WidthOnly. HeightOnly must
        // return or InDesign clips the NoBreak paragraph.
        let mut els = vec![tb("letter.p2", 42_000, 510_000, true, "CenterLeftPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text =
                "Your thoughtful advice and gentle encouragement arrived\nat the exact moment they were most needed.\nIt is a rare and precious gift to have someone who listens\nwith such sincere empathy and offers wisdom with such grace."
                    .into();
            t.autosize_height = false;
            t.no_break = true;
            t.lock_line_count = 4;
        }
        apply(&mut els, 595_000);
        let PageElement::TextBox(body) = &els[0] else {
            panic!("textbox")
        };
        assert!(
            !body.autosize_width,
            "wide left body must not WidthOnly-shrink"
        );
        assert!(
            body.autosize_height,
            "multi-line left must HeightOnly after WidthOnly cleared"
        );
        assert!(
            !body.no_break,
            "full-width lock-pinned body must wrap so InDesign does not hide the story"
        );
    }

    #[test]
    fn stacked_graphic_caption_keeps_centerpoint_widthonly() {
        let mut els = vec![
            shape(202_000, 53_865, 36_000, 36_000),
            tb("caption", 202_393, 37_214, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[1] {
            t.rect.y = Pt(91_865);
            t.rect.height = Pt(4_620);
            t.runs[0].text = "SCAN TO VERIFY".into();
        }
        apply(&mut els, 252_000);
        let PageElement::TextBox(cap) = &els[1] else {
            panic!("textbox")
        };
        assert!(cap.autosize_width, "caption under graphic keeps WidthOnly");
        assert_eq!(cap.autosize_refer, "CenterPoint");
    }

    #[test]
    fn page_centered_label_keeps_centerpoint_widthonly() {
        // Invitation monogram: tight ink, box centered on A5-width page.
        let page_w = 360_000i128;
        let mut els = vec![tb("card.front.header.monogram", 158_087, 43_825, true, "CenterLeftPoint")];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text = "E\u{00A0}·\u{00A0}L".into();
            t.runs[0].tracking = 0;
        }
        apply(&mut els, page_w);
        let PageElement::TextBox(mono) = &els[0] else {
            panic!("textbox")
        };
        assert_eq!(mono.align, TextAlign::Center);
        assert_eq!(mono.autosize_refer, "CenterPoint");
        assert!(
            mono.autosize_width,
            "page-centered label must keep WidthOnly, not open_right expand"
        );
        assert_eq!(mono.rect.width.0, 43_825, "lock width must stay");
        assert_eq!(mono.rect.x.0, 158_087, "lock left edge must stay");
    }

    #[test]
    fn page_centered_grid_column_stays_left_aligned() {
        // bauhaus-geometric card2: middle column sits on the page midline but
        // ink is flush-left. page_center_autosize must not promote to Center.
        let page_w = 595_000i128;
        let col_w = 148_333i128;
        let col_x = (page_w - col_w) / 2;
        let mut els = vec![
            tb("card1.tag", col_x - col_w - 10_000, col_w, true, "CenterLeftPoint"),
            tb("card2.tag", col_x, col_w, true, "CenterLeftPoint"),
            tb("card3.tag", col_x + col_w + 10_000, col_w, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[1] {
            t.runs[0].text = "WERKSTATT 02 / TYPO".into();
        }
        apply(&mut els, page_w);
        let PageElement::TextBox(mid) = &els[1] else {
            panic!("textbox")
        };
        assert_eq!(
            mid.align,
            TextAlign::Left,
            "middle grid column must stay left-aligned"
        );
        assert_ne!(
            mid.autosize_refer,
            "CenterPoint",
            "column label must not re-anchor about center"
        );
    }

    #[test]
    fn stacked_left_column_beside_hairline_stays_lock_width() {
        // color-block card.back.person_*: 136pt left column, 6pt gap, 0.6pt
        // divider. WidthOnly about ItemTransform centers the delegate block.
        let mut els = vec![
            tb("person_tag", 12_000, 136_000, false, "CenterLeftPoint"),
            tb("person_name", 12_000, 136_000, true, "CenterLeftPoint"),
            shape(154_000, 0, 600, 94_200),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.runs[0].text = "DELEGATE SPECIFICATION".into();
            t.runs[0].tracking = 0;
        }
        if let PageElement::TextBox(t) = &mut els[1] {
            t.rect.y = Pt(17_000);
            t.runs[0].text = "DR. AVERY CHEN".into();
            t.runs[0].tracking = 0;
        }
        apply(&mut els, 252_000);
        for el in &els[..2] {
            let PageElement::TextBox(t) = el else {
                panic!("textbox")
            };
            assert_eq!(t.align, TextAlign::Left);
            assert!(
                !t.autosize_width,
                "{} must keep lock width, not WidthOnly-shrink",
                t.node_id
            );
            assert_eq!(t.rect.x.0, 12_000, "left edge must stay");
            assert!(
                t.rect.width.0 >= 136_000 && t.rect.width.0 <= 142_000,
                "{} width {} should stay in the column (maybe 6pt gutter pad)",
                t.node_id,
                t.rect.width.0
            );
        }
    }

    #[test]
    fn stacked_cta_below_column_body_keeps_nobreak() {
        // 3-col grid: lock-pinned body beside another card, CTA 4pt below.
        // Dropping NoBreak + HeightOnly extra-wraps the first lock line and
        // overprints the CTA.
        let mut els = vec![
            tb("col", 0, 140_000, true, "CenterLeftPoint"),
            tb("body", 300_000, 140_000, true, "CenterLeftPoint"),
            tb("cta", 300_000, 90_000, true, "CenterLeftPoint"),
        ];
        if let PageElement::TextBox(t) = &mut els[0] {
            t.rect.height = Pt(40_000);
        }
        if let PageElement::TextBox(t) = &mut els[1] {
            t.runs[0].text = "Continue to Part 04: Typographic Color\n& Serial Rhythm.".into();
            t.rect.height = Pt(21_000);
        }
        if let PageElement::TextBox(t) = &mut els[2] {
            t.runs[0].text = "EXPLORE ESSAY 10 >>".into();
            t.rect.y = Pt(25_000);
            t.rect.height = Pt(10_000);
        }
        apply(&mut els, 500_000);
        let PageElement::TextBox(body) = &els[1] else {
            panic!("textbox")
        };
        assert!(
            body.no_break,
            "body above a CTA must not extra-wrap into it"
        );
        assert!(
            !body.autosize_height,
            "must not HeightOnly through the CTA, got height autosize"
        );
        assert!(!body.autosize_width, "must not WidthOnly through the column");
        assert_eq!(body.rect.x.0, 300_000, "column left edge stays");
        assert!(
            body.rect.width.0 > 140_000,
            "must grow into the open page, got {}",
            body.rect.width.0
        );
        assert!(
            body.rect.x.0 + body.rect.width.0 <= 500_000,
            "must not grow past the page"
        );
        assert!(
            body.runs[0].text.contains('\n'),
            "lock wrap before the CTA must stay"
        );
        assert_eq!(body.rect.y.0 + body.rect.height.0, 21_000, "lock height stays");
    }
}
