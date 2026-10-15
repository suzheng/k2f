//! Post-layout diagnostics. Not part of the lock or appearance hash.

use crate::fixed_size::fixed_size_hint;
use crate::pagination::content_height;
use crate::resolved_style::padding_for_role_variant;
use crate::theme::Theme;
use k2f_core::{
    collect_boxes, for_each_node, GeometryNode, GridTrack, LayoutHint, LayoutResult, Manifest,
    NodeContent, Pt, SemanticNode,
};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Minimum share of the page content box a box must occupy before unused
/// space is reported (skips small cards / `ex_grid` demos).
const MIN_HEIGHT_NUM: i128 = 40;
const MIN_HEIGHT_DEN: i128 = 100;
/// Unused below as a share of the inner box (grower / pinned shell).
const UNUSED_NUM: i128 = 12;
const UNUSED_DEN: i128 = 100;
/// Unused below as a share of the page content box (`PAGE_UNDERFILL`).
const PAGE_UNUSED_NUM: i128 = 25;
const PAGE_UNUSED_DEN: i128 = 100;
/// Skip fill diagnostics on miniature canvases (e.g. 252×144pt cards).
/// Percentage leftover there is inset, not a hollow A4 grower.
const MIN_PAGE_FOR_DIAGS: i128 = 180_000;
/// Absolute unused floor so a few points of breathing room never warn.
const MIN_UNUSED_ABS: i128 = 36_000;

const GROWER_HINT: &str = "nest {fr:1} in the grower; do not pack an auto-height stack";
const PAGE_HINT_ONE: &str =
    "one-page form: copy ex_filled_page.json; set height to page_h − margins; leftover on table or notes {fr:1}";
const PAGE_HINT_MID: &str =
    "do not pre-paginate with break_before; one flow tree, engine fills the page";
/// Ignore sub-point rounding when comparing laid-out text to its box.
const OVERFLOW_ABS: i128 = 1_000;
const OVERFLOW_HINT: &str = "shorten copy or grow the row; text paints outside its box";
const FONT_SYNTHETIC_HINT: &str =
    "declare font_faces so this face is selected; otherwise bold/italic stays synthetic";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDiagKind {
    Slack,
    PageUnderfill,
    ContentOverflow,
    FontSynthetic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutDiag {
    pub kind: LayoutDiagKind,
    pub node_id: String,
    pub unused_below: Pt,
    pub unused_above: Pt,
    pub inner_height: Pt,
    pub after_id: Option<String>,
    pub hint: &'static str,
    pub page: Option<usize>,
}

impl LayoutDiag {
    pub fn unused_pct(&self) -> i128 {
        if self.inner_height.0 <= 0 {
            0
        } else {
            self.unused_below.0 * 100 / self.inner_height.0
        }
    }
}

impl fmt::Display for LayoutDiag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let after = self.after_id.as_deref().unwrap_or("-");
        match self.kind {
            LayoutDiagKind::Slack => write!(
                f,
                "LAYOUT_SLACK id={} unused_below={} ({}%) unused_above={} after={} hint={}",
                self.node_id,
                self.unused_below.0,
                self.unused_pct(),
                self.unused_above.0,
                after,
                self.hint
            ),
            LayoutDiagKind::PageUnderfill => write!(
                f,
                "PAGE_UNDERFILL page={} unused_below={} ({}%) after={} hint={}",
                self.page.unwrap_or(0),
                self.unused_below.0,
                self.unused_pct(),
                after,
                self.hint
            ),
            LayoutDiagKind::ContentOverflow => write!(
                f,
                "CONTENT_OVERFLOW id={} overflow={} hint={}",
                self.node_id, self.unused_below.0, self.hint
            ),
            LayoutDiagKind::FontSynthetic => write!(
                f,
                "FONT_SYNTHETIC id={} unused={} hint={}",
                self.node_id, after, self.hint
            ),
        }
    }
}

pub fn layout_slack_diags(
    manifest: &Manifest,
    layout: &LayoutResult,
    theme: &Theme,
) -> Vec<LayoutDiag> {
    let content_h = content_height(&manifest.page_config);
    if content_h.0 < MIN_PAGE_FOR_DIAGS {
        return vec![];
    }
    let min_h = Pt(content_h.0 * MIN_HEIGHT_NUM / MIN_HEIGHT_DEN);

    let mut diags = Vec::new();
    let mut seen = HashSet::new();
    for_each_node(&manifest.root, &mut |node| {
        if let Some(h) = fixed_size_hint(&node.layout).height {
            if h.0 >= min_h.0 {
                consider(node, layout, theme, &mut diags, &mut seen);
            }
        }
        consider_fr_occupants(node, layout, theme, min_h, &mut diags, &mut seen);
    });
    consider_pages(manifest, layout, theme, &mut diags);
    diags
}

fn consider_fr_occupants(
    node: &SemanticNode,
    layout: &LayoutResult,
    theme: &Theme,
    min_h: Pt,
    diags: &mut Vec<LayoutDiag>,
    seen: &mut HashSet<String>,
) {
    let Some(LayoutHint::Grid { columns, rows, .. }) = &node.layout else {
        return;
    };
    let NodeContent::Container { children } = &node.content else {
        return;
    };
    if columns.is_empty() {
        return;
    }
    let resolved_rows = k2f_core::grid_rows_for_children(columns, rows, children.len());
    let rows = resolved_rows.as_slice();
    let ncols = columns.len();
    for (idx, child) in children.iter().enumerate() {
        let r = idx / ncols;
        if r >= rows.len() {
            break;
        }
        if !matches!(rows[r], GridTrack::Fr { .. }) {
            continue;
        }
        let Some(geo) = find_geo(layout, &child.id) else {
            continue;
        };
        if geo.height.0 < min_h.0 {
            continue;
        }
        consider(child, layout, theme, diags, seen);
    }
}

fn consider(
    node: &SemanticNode,
    layout: &LayoutResult,
    theme: &Theme,
    diags: &mut Vec<LayoutDiag>,
    seen: &mut HashSet<String>,
) {
    if !seen.insert(node.id.clone()) {
        return;
    }
    let Some(geo) = find_geo(layout, &node.id) else {
        return;
    };
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), theme)
        .unwrap_or(crate::resolved_style::EdgeInsets::ZERO);
    if let Some(diag) = slack_for_box(&node.id, geo, padding, paints_own_box(&node.content)) {
        diags.push(diag);
    }
}

fn paints_own_box(content: &NodeContent) -> bool {
    !matches!(
        content,
        NodeContent::Container { .. } | NodeContent::Table(_)
    )
}

fn find_geo<'a>(layout: &'a LayoutResult, id: &str) -> Option<&'a GeometryNode> {
    for page in &layout.pages {
        let mut boxes = Vec::new();
        collect_boxes(&page.root, id, &mut boxes);
        if let Some(geo) = boxes.first().copied() {
            return Some(geo);
        }
    }
    None
}

fn slack_for_box(
    id: &str,
    geo: &GeometryNode,
    padding: crate::resolved_style::EdgeInsets,
    paints_own_box: bool,
) -> Option<LayoutDiag> {
    let inner_h = Pt((geo.height.0 - padding.vertical().0).max(0));
    if inner_h.0 <= 0 {
        return None;
    }
    let inner_top = geo.y + padding.top;
    let inner_bottom = inner_top + inner_h;

    let (unused_above, unused_below, after_id) = if geo.children.is_empty() {
        if paints_own_box {
            return None;
        }
        (Pt::ZERO, inner_h, None)
    } else {
        let first = geo.children.first().unwrap();
        let last = geo.children.last().unwrap();
        let above = Pt((first.y.0 - inner_top.0).max(0));
        let below = Pt((inner_bottom.0 - last.y.0 - last.height.0).max(0));
        (above, below, Some(last.id.clone()))
    };

    if unused_below.0 < MIN_UNUSED_ABS || unused_below.0 * UNUSED_DEN < inner_h.0 * UNUSED_NUM {
        return None;
    }
    if unused_above.0 > 0 && unused_below.0 <= unused_above.0 * 2 {
        return None;
    }

    Some(LayoutDiag {
        kind: LayoutDiagKind::Slack,
        node_id: id.to_string(),
        unused_below,
        unused_above,
        inner_height: inner_h,
        after_id,
        hint: GROWER_HINT,
        page: None,
    })
}

fn running_ids(manifest: &Manifest) -> HashSet<String> {
    let mut ids = HashSet::new();
    for rb in &manifest.running_blocks {
        for_each_node(&rb.node, &mut |n| {
            ids.insert(n.id.clone());
        });
    }
    ids
}

fn consider_pages(
    manifest: &Manifest,
    layout: &LayoutResult,
    theme: &Theme,
    diags: &mut Vec<LayoutDiag>,
) {
    let n = layout.pages.len();
    if n == 0 {
        return;
    }
    let pad =
        padding_for_role_variant(&manifest.root.role, manifest.root.variant.as_deref(), theme)
            .unwrap_or(crate::resolved_style::EdgeInsets::ZERO);
    let content_top = manifest.page_config.margin[0] + pad.top;
    let content_h = content_height(&manifest.page_config) - pad.vertical();
    if content_h.0 <= 0 {
        return;
    }
    let content_bottom = content_top + content_h;
    let running = running_ids(manifest);

    for (i, page) in layout.pages.iter().enumerate() {
        if n > 1 && i + 1 == n {
            continue;
        }
        let mut lowest: Option<(&GeometryNode, Pt)> = None;
        let mut highest_top: Option<Pt> = None;
        for child in &page.root.children {
            if running.contains(&child.id) {
                continue;
            }
            let bottom = child.y + child.height;
            match lowest {
                None => lowest = Some((child, bottom)),
                Some((_, b)) if bottom > b => lowest = Some((child, bottom)),
                _ => {}
            }
            match highest_top {
                None => highest_top = Some(child.y),
                Some(t) if child.y < t => highest_top = Some(child.y),
                _ => {}
            }
        }
        let Some((after, last_bottom)) = lowest else {
            continue;
        };
        let unused_below = Pt((content_bottom.0 - last_bottom.0).max(0));
        let unused_above = Pt((highest_top.unwrap_or(content_top).0 - content_top.0).max(0));
        if unused_below.0 < MIN_UNUSED_ABS
            || unused_below.0 * PAGE_UNUSED_DEN < content_h.0 * PAGE_UNUSED_NUM
        {
            continue;
        }
        let hint = if n == 1 { PAGE_HINT_ONE } else { PAGE_HINT_MID };
        diags.push(LayoutDiag {
            kind: LayoutDiagKind::PageUnderfill,
            node_id: page.root.id.clone(),
            unused_below,
            unused_above,
            inner_height: content_h,
            after_id: Some(after.id.clone()),
            hint,
            page: Some(i),
        });
    }
}

struct SemSnap {
    role: String,
    variant: Option<String>,
    check_glyphs: bool,
}

/// Text or a child box painting past the box it was given.
///
/// Grid cells clamp the child box, then text layout still draws every line.
/// Fixed stacks and `columns` keep the child's measured height, so the box
/// itself can extend past the parent. Form fields clip glyphs and are skipped.
/// Warning only — not stored in the lock.
pub fn content_overflow_diags(
    manifest: &Manifest,
    layout: &LayoutResult,
    theme: &Theme,
) -> Vec<LayoutDiag> {
    let sem = index_semantic(manifest);
    let mut diags = Vec::new();
    let mut seen = HashSet::new();
    for page in &layout.pages {
        walk_overflow(&page.root, &sem, theme, &mut diags, &mut seen);
    }
    diags
}

pub(crate) fn font_synthetic_diag(family: &str, unused_face: &str) -> LayoutDiag {
    LayoutDiag {
        kind: LayoutDiagKind::FontSynthetic,
        node_id: family.to_string(),
        unused_below: Pt::ZERO,
        unused_above: Pt::ZERO,
        inner_height: Pt::ZERO,
        after_id: Some(unused_face.to_string()),
        hint: FONT_SYNTHETIC_HINT,
        page: None,
    }
}

fn index_semantic(manifest: &Manifest) -> HashMap<String, SemSnap> {
    let mut map = HashMap::new();
    let mut put = |node: &SemanticNode| {
        map.insert(
            node.id.clone(),
            SemSnap {
                role: node.role.clone(),
                variant: node.variant.clone(),
                check_glyphs: matches!(
                    node.content,
                    NodeContent::Text(_) | NodeContent::CodeBlock(_)
                ),
            },
        );
    };
    for_each_node(&manifest.root, &mut put);
    for block in &manifest.running_blocks {
        for_each_node(&block.node, &mut put);
    }
    map
}

fn walk_overflow(
    node: &GeometryNode,
    sem: &HashMap<String, SemSnap>,
    theme: &Theme,
    diags: &mut Vec<LayoutDiag>,
    seen: &mut HashSet<String>,
) {
    if let Some(snap) = sem.get(&node.id) {
        if snap.check_glyphs {
            if let Some(overflow) = glyph_overflow(node, snap, theme) {
                push_overflow(&node.id, overflow, diags, seen);
            }
        }
    }
    let padding = sem
        .get(&node.id)
        .and_then(|snap| padding_for_role_variant(&snap.role, snap.variant.as_deref(), theme).ok())
        .unwrap_or(crate::resolved_style::EdgeInsets::ZERO);
    let inner_bottom = node.y.0 + node.height.0 - padding.bottom.0;
    for child in &node.children {
        let past = child.y.0 + child.height.0 - inner_bottom;
        if past > OVERFLOW_ABS {
            push_overflow(&child.id, past, diags, seen);
        }
        walk_overflow(child, sem, theme, diags, seen);
    }
}

fn push_overflow(
    id: &str,
    overflow: i128,
    diags: &mut Vec<LayoutDiag>,
    seen: &mut HashSet<String>,
) {
    if !seen.insert(id.to_string()) {
        return;
    }
    diags.push(LayoutDiag {
        kind: LayoutDiagKind::ContentOverflow,
        node_id: id.to_string(),
        unused_below: Pt(overflow),
        unused_above: Pt::ZERO,
        inner_height: Pt::ZERO,
        after_id: None,
        hint: OVERFLOW_HINT,
        page: None,
    });
}

/// Bottom of the last line box minus the node height, when text was laid out
/// past a clamped box. Line top is the y most glyphs on that line share, so a
/// mark sitting off the baseline does not look like an extra line.
fn glyph_overflow(node: &GeometryNode, snap: &SemSnap, theme: &Theme) -> Option<i128> {
    let style =
        crate::resolved_style::resolve_text_style(&snap.role, snap.variant.as_deref(), theme);
    let line_h = crate::text_layout::line_height_for_style(&style).0;
    if line_h <= 0 || node.glyphs.is_empty() {
        return None;
    }
    let line_top = last_line_top(&node.glyphs)?;
    let past = line_top + line_h - node.height.0;
    if past > OVERFLOW_ABS {
        Some(past)
    } else {
        None
    }
}

fn last_line_top(glyphs: &[k2f_core::GlyphPosition]) -> Option<i128> {
    let mut counts: HashMap<i128, usize> = HashMap::new();
    for glyph in glyphs {
        *counts.entry(glyph.y_offset.0).or_default() += 1;
    }
    let best = counts.values().copied().max().unwrap_or(1);
    if best >= 2 {
        counts
            .into_iter()
            .filter(|(_, n)| *n >= 2)
            .map(|(y, _)| y)
            .max()
    } else {
        counts.into_keys().max()
    }
}
