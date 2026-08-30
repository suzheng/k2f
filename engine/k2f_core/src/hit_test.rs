use crate::{GeometryNode, GlyphPosition, LayoutResult, Page, Pt};
use serde::{Deserialize, Serialize};

/// Leaf-first hit: innermost geometry box, then ancestors.
/// Children are tested back-to-front (last child wins when boxes overlap).
/// A box is hittable over its full rectangle, including padding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hit {
    pub ids: Vec<String>,
    pub char_range: Option<[usize; 2]>,
}

impl Hit {
    pub fn leaf(&self) -> Option<&str> {
        self.ids.first().map(String::as_str)
    }
}

pub fn box_contains(node: &GeometryNode, x: Pt, y: Pt) -> bool {
    if node.width.0 <= 0 || node.height.0 <= 0 {
        return false;
    }
    x >= node.x && y >= node.y && x < node.x + node.width && y < node.y + node.height
}

pub fn hit_geometry(node: &GeometryNode, x: Pt, y: Pt) -> Option<Vec<String>> {
    for child in node.children.iter().rev() {
        if let Some(mut ids) = hit_geometry(child, x, y) {
            ids.push(node.id.clone());
            return Some(ids);
        }
    }
    if box_contains(node, x, y) {
        Some(vec![node.id.clone()])
    } else {
        None
    }
}

pub fn hit_page(page: &Page, x: Pt, y: Pt) -> Option<Hit> {
    let ids = hit_geometry(&page.root, x, y)?;
    let mut boxes = Vec::new();
    collect_boxes(&page.root, &ids[0], &mut boxes);
    let char_range = boxes
        .iter()
        .find(|leaf| box_contains(leaf, x, y))
        .and_then(|leaf| hit_char_range(leaf, x, y));
    Some(Hit { ids, char_range })
}

/// Character range for the glyph whose advance box contains `(x, y)`.
///
/// `GlyphPosition.cluster` is a UTF-8 **character** index. Missing clusters
/// (old locks: every cluster is 0) return `None` so callers can fall back to
/// the whole node. Decorative glyphs use `CLUSTER_NOT_SOURCE` and are skipped.
pub fn hit_char_range(node: &GeometryNode, x: Pt, y: Pt) -> Option<[usize; 2]> {
    if !clusters_usable(&node.glyphs) {
        return None;
    }
    let glyph = glyph_at(node, x, y)?;
    if glyph.cluster == GlyphPosition::CLUSTER_NOT_SOURCE {
        return None;
    }
    let start = glyph.cluster as usize;
    Some([start, start + 1])
}

pub fn clusters_usable(glyphs: &[GlyphPosition]) -> bool {
    glyphs
        .iter()
        .any(|g| g.cluster != 0 && g.cluster != GlyphPosition::CLUSTER_NOT_SOURCE)
}

fn glyph_at<'a>(node: &'a GeometryNode, x: Pt, y: Pt) -> Option<&'a GlyphPosition> {
    for glyph in node.glyphs.iter().rev() {
        if glyph.cluster == GlyphPosition::CLUSTER_NOT_SOURCE {
            continue;
        }
        if glyph.x_advance.0 <= 0 {
            continue;
        }
        let left = node.x + glyph.x_offset;
        let top = node.y + glyph.y_offset;
        let right = left + glyph.x_advance;
        let bottom = line_bottom(node, top);
        if x >= left && x < right && y >= top && y < bottom {
            return Some(glyph);
        }
    }
    None
}

fn line_bottom(node: &GeometryNode, line_top: Pt) -> Pt {
    let mut tops: Vec<Pt> = node.glyphs.iter().map(|g| node.y + g.y_offset).collect();
    tops.sort();
    tops.dedup();
    for pair in tops.windows(2) {
        if pair[0] == line_top {
            return pair[1];
        }
    }
    node.y + node.height
}

pub fn hit_layout(layout: &LayoutResult, page: usize, x: Pt, y: Pt) -> Option<Hit> {
    hit_page(layout.pages.get(page)?, x, y)
}

pub fn with_text_range(mut hit: Hit, text: Option<&str>) -> Hit {
    if let Some(t) = text {
        hit.char_range = Some([0, t.chars().count()]);
    }
    hit
}

/// Semantic IDs only: drop synthetic page roots such as `root_page_0`.
pub fn semantic_ids(ids: &[String], exists: impl Fn(&str) -> bool) -> Vec<String> {
    ids.iter()
        .filter(|id| exists(id.as_str()))
        .cloned()
        .collect()
}

pub fn collect_boxes<'a>(node: &'a GeometryNode, id: &str, out: &mut Vec<&'a GeometryNode>) {
    if node.id == id {
        out.push(node);
    }
    for child in &node.children {
        collect_boxes(child, id, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geo(
        id: &str,
        x: i128,
        y: i128,
        w: i128,
        h: i128,
        children: Vec<GeometryNode>,
    ) -> GeometryNode {
        GeometryNode {
            id: id.into(),
            x: Pt(x),
            y: Pt(y),
            width: Pt(w),
            height: Pt(h),
            glyphs: vec![],
            text_runs: vec![],
            fill_rects: vec![],
            children,
        }
    }

    fn glyph(cluster: u32, x: i128, advance: i128) -> GlyphPosition {
        GlyphPosition {
            glyph_id: 1,
            cluster,
            x_offset: Pt(x),
            y_offset: Pt(0),
            x_advance: Pt(advance),
            y_advance: Pt(0),
        }
    }

    #[test]
    fn hit_point_in_glyph_returns_char_index() {
        let mut node = geo("ab", 0, 0, 200, 20, vec![]);
        node.glyphs = vec![glyph(0, 0, 50), glyph(1, 50, 50)];
        assert_eq!(hit_char_range(&node, Pt(10), Pt(10)), Some([0, 1]));
        assert_eq!(hit_char_range(&node, Pt(75), Pt(10)), Some([1, 2]));
    }

    #[test]
    fn all_zero_clusters_mean_missing_and_return_none() {
        let mut node = geo("old", 0, 0, 200, 20, vec![]);
        node.glyphs = vec![glyph(0, 0, 50), glyph(0, 50, 50)];
        assert_eq!(hit_char_range(&node, Pt(75), Pt(10)), None);
    }

    #[test]
    fn decorative_glyphs_are_skipped() {
        let mut node = geo("item", 0, 0, 200, 20, vec![]);
        node.glyphs = vec![
            glyph(GlyphPosition::CLUSTER_NOT_SOURCE, 0, 20),
            glyph(0, 20, 50),
            glyph(1, 70, 50),
        ];
        assert_eq!(hit_char_range(&node, Pt(10), Pt(10)), None);
        assert_eq!(hit_char_range(&node, Pt(30), Pt(10)), Some([0, 1]));
        assert_eq!(hit_char_range(&node, Pt(90), Pt(10)), Some([1, 2]));
    }

    #[test]
    fn leaf_wins_over_ancestors() {
        let tree = geo(
            "page",
            0,
            0,
            1000,
            1000,
            vec![geo(
                "parent",
                10,
                10,
                400,
                400,
                vec![geo("leaf", 20, 20, 50, 50, vec![])],
            )],
        );
        let ids = hit_geometry(&tree, Pt(30), Pt(30)).unwrap();
        assert_eq!(ids, vec!["leaf", "parent", "page"]);
    }

    #[test]
    fn padding_hits_parent_when_outside_children() {
        let tree = geo(
            "parent",
            0,
            0,
            100,
            100,
            vec![geo("child", 20, 20, 20, 20, vec![])],
        );
        let ids = hit_geometry(&tree, Pt(5), Pt(5)).unwrap();
        assert_eq!(ids, vec!["parent"]);
    }

    #[test]
    fn later_sibling_wins_when_boxes_overlap() {
        let tree = geo(
            "root",
            0,
            0,
            100,
            100,
            vec![
                geo("back", 0, 0, 80, 80, vec![]),
                geo("front", 10, 10, 80, 80, vec![]),
            ],
        );
        let ids = hit_geometry(&tree, Pt(20), Pt(20)).unwrap();
        assert_eq!(ids[0], "front");
    }

    #[test]
    fn miss_outside_all_boxes() {
        let tree = geo("root", 0, 0, 10, 10, vec![]);
        assert!(hit_geometry(&tree, Pt(20), Pt(20)).is_none());
    }

    #[test]
    fn zero_size_box_is_not_hittable() {
        let tree = geo("empty", 0, 0, 0, 10, vec![]);
        assert!(hit_geometry(&tree, Pt(0), Pt(0)).is_none());
    }

    #[test]
    fn overflowing_child_is_still_hittable() {
        let tree = geo(
            "parent",
            0,
            0,
            50,
            50,
            vec![geo("out", 40, 40, 30, 30, vec![])],
        );
        let ids = hit_geometry(&tree, Pt(60), Pt(60)).unwrap();
        assert_eq!(ids[0], "out");
    }
}
