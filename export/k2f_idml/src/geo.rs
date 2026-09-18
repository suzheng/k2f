use k2f_core::GeometryNode;

pub(crate) fn find_geo<'a>(node: &'a GeometryNode, id: &str) -> Option<&'a GeometryNode> {
    if node.id == id {
        return Some(node);
    }
    node.children.iter().find_map(|c| find_geo(c, id))
}
