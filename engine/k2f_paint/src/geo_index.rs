use k2f_core::{GeometryNode, Rect};
use std::collections::HashMap;

/// Index all geometry boxes by semantic id (same id may appear multiple times on one page).
pub fn index_geometry_multi<'a>(
    node: &'a GeometryNode,
    out: &mut HashMap<String, Vec<&'a GeometryNode>>,
) {
    out.entry(node.id.clone()).or_default().push(node);
    for c in &node.children {
        index_geometry_multi(c, out);
    }
}

pub fn index_geometry_multi_str<'a>(
    node: &'a GeometryNode,
    out: &mut HashMap<&'a str, Vec<&'a GeometryNode>>,
) {
    out.entry(node.id.as_str()).or_default().push(node);
    for c in &node.children {
        index_geometry_multi_str(c, out);
    }
}

fn rect_matches(geo: &GeometryNode, rect: &Rect) -> bool {
    geo.x == rect.x
        && geo.y == rect.y
        && geo.width == rect.width
        && geo.height == rect.height
}

/// Resolve the geometry fragment for a paint op. Prefers an exact rect match when
/// multiple boxes share `node_id` (multi-column / fragmented text on one page).
pub fn geo_for_op<'a>(
    geos: &HashMap<String, Vec<&'a GeometryNode>>,
    node_id: &str,
    rect: &Rect,
) -> Option<&'a GeometryNode> {
    let list = geos.get(node_id)?;
    match list.as_slice() {
        [] => None,
        [only] => Some(*only),
        many => many
            .iter()
            .copied()
            .find(|g| rect_matches(g, rect))
            .or_else(|| many.first().copied()),
    }
}

pub fn geo_for_op_str<'a>(
    geos: &HashMap<&'a str, Vec<&'a GeometryNode>>,
    node_id: &str,
    rect: &Rect,
) -> Option<&'a GeometryNode> {
    let list = geos.get(node_id)?;
    match list.as_slice() {
        [] => None,
        [only] => Some(*only),
        many => many
            .iter()
            .copied()
            .find(|g| rect_matches(g, rect))
            .or_else(|| many.first().copied()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::Pt;

    fn geo(id: &str, x: i128, y: i128, w: i128, h: i128) -> GeometryNode {
        GeometryNode {
            id: id.into(),
            x: Pt(x),
            y: Pt(y),
            width: Pt(w),
            height: Pt(h),
            glyphs: vec![],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        }
    }

    #[test]
    fn picks_rect_match_among_duplicates() {
        let a = geo("p", 10, 20, 100, 30);
        let b = geo("p", 200, 20, 100, 40);
        let mut map = HashMap::new();
        map.insert("p".into(), vec![&a, &b]);
        let rect = Rect {
            x: Pt(200),
            y: Pt(20),
            width: Pt(100),
            height: Pt(40),
        };
        let got = geo_for_op(&map, "p", &rect).unwrap();
        assert_eq!(got.x, Pt(200));
        assert_eq!(got.height, Pt(40));
    }
}
