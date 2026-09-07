use crate::{NodeContent, SemanticNode, TableDataSource};

pub fn find_node<'a>(node: &'a SemanticNode, id: &str) -> Option<&'a SemanticNode> {
    if node.id == id {
        return Some(node);
    }
    match &node.content {
        NodeContent::Container { children } => children.iter().find_map(|c| find_node(c, id)),
        NodeContent::Table(spec) => match &spec.data {
            TableDataSource::Inline { rows } => {
                rows.iter().flatten().find_map(|c| find_node(c, id))
            }
            TableDataSource::Asset { .. } => None,
        },
        _ => None,
    }
}

pub fn find_node_mut<'a>(node: &'a mut SemanticNode, id: &str) -> Option<&'a mut SemanticNode> {
    if node.id == id {
        return Some(node);
    }
    match &mut node.content {
        NodeContent::Container { children } => {
            children.iter_mut().find_map(|c| find_node_mut(c, id))
        }
        NodeContent::Table(spec) => match &mut spec.data {
            TableDataSource::Inline { rows } => {
                rows.iter_mut().flatten().find_map(|c| find_node_mut(c, id))
            }
            TableDataSource::Asset { .. } => None,
        },
        _ => None,
    }
}

pub fn for_each_node(node: &SemanticNode, visit: &mut impl FnMut(&SemanticNode)) {
    visit(node);
    match &node.content {
        NodeContent::Container { children } => {
            for child in children {
                for_each_node(child, visit);
            }
        }
        NodeContent::Table(spec) => {
            if let TableDataSource::Inline { rows } = &spec.data {
                for row in rows {
                    for cell in row {
                        for_each_node(cell, visit);
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn for_each_node_mut(node: &mut SemanticNode, visit: &mut impl FnMut(&mut SemanticNode)) {
    visit(node);
    match &mut node.content {
        NodeContent::Container { children } => {
            for child in children {
                for_each_node_mut(child, visit);
            }
        }
        NodeContent::Table(spec) => {
            if let TableDataSource::Inline { rows } = &mut spec.data {
                for row in rows {
                    for cell in row {
                        for_each_node_mut(cell, visit);
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GridTrack, TableDataSource, TableSpec};

    fn text(id: &str, s: &str) -> SemanticNode {
        SemanticNode {
            id: id.to_string(),
            role: "body".to_string(),
            content: NodeContent::Text(s.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn finds_nested_and_table_cells() {
        let root = SemanticNode {
            id: "root".into(),
            role: "document".into(),
            content: NodeContent::Container {
                children: vec![
                    text("a", "one"),
                    SemanticNode {
                        id: "t".into(),
                        role: "table".into(),
                        content: NodeContent::Table(TableSpec {
                            column_widths: vec![GridTrack::Fr { fr: 1 }],
                            header_rows: 0,
                            gap: 0,
                            row_gap: None,
                            column_gap: None,
                            data: TableDataSource::Inline {
                                rows: vec![vec![text("t.c0", "cell")]],
                            },
                        }),
                        ..Default::default()
                    },
                ],
            },
            ..Default::default()
        };
        assert_eq!(find_node(&root, "a").unwrap().id, "a");
        assert_eq!(find_node(&root, "t.c0").unwrap().id, "t.c0");
        assert!(find_node(&root, "missing").is_none());
        let mut root = root;
        find_node_mut(&mut root, "a").unwrap().content = NodeContent::Text("two".into());
        match &find_node(&root, "a").unwrap().content {
            NodeContent::Text(s) => assert_eq!(s, "two"),
            _ => panic!("expected text"),
        }
    }
}
