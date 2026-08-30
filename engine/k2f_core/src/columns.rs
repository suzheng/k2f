use crate::{K2FError, LayoutHint, NodeContent, SemanticNode};
use serde::{Deserialize, Serialize};

/// Whether a node spans all columns of an enclosing columns container.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ColumnSpan {
    #[default]
    None,
    All,
}

impl ColumnSpan {
    pub fn is_none(&self) -> bool {
        matches!(self, ColumnSpan::None)
    }

    pub fn is_all(&self) -> bool {
        matches!(self, ColumnSpan::All)
    }
}

/// Validate columns layout hints and column_span placement across the tree.
pub(crate) fn validate_columns_tree(root: &SemanticNode) -> Result<(), K2FError> {
    validate_columns_node(root, false)
}

fn is_columns_layout(layout: &Option<LayoutHint>) -> bool {
    matches!(layout, Some(LayoutHint::Columns { .. }))
}

fn validate_columns_node(node: &SemanticNode, inside_columns: bool) -> Result<(), K2FError> {
    let entering_columns = is_columns_layout(&node.layout);

    if node.column_span == ColumnSpan::All {
        if entering_columns {
            return Err(K2FError::ColumnsInvalid {
                node_id: node.id.clone(),
                reason: "column_span all cannot be set on a columns container".into(),
            });
        }
        if !inside_columns {
            return Err(K2FError::ColumnSpanOutsideColumns {
                node_id: node.id.clone(),
            });
        }
    }

    if entering_columns {
        if inside_columns {
            return Err(K2FError::ColumnsNested {
                node_id: node.id.clone(),
            });
        }
        match &node.layout {
            Some(LayoutHint::Columns { count, gap }) => {
                if !(2..=4).contains(count) {
                    return Err(K2FError::ColumnsInvalid {
                        node_id: node.id.clone(),
                        reason: format!("count must be 2..=4 (got {count})"),
                    });
                }
                if *gap < 0 {
                    return Err(K2FError::ColumnsInvalid {
                        node_id: node.id.clone(),
                        reason: format!("gap must be >= 0 (got {gap})"),
                    });
                }
            }
            _ => {}
        }
        if !matches!(node.content, NodeContent::Container { .. }) {
            return Err(K2FError::ColumnsInvalid {
                node_id: node.id.clone(),
                reason: "columns layout requires container content".into(),
            });
        }
    }

    let child_inside = inside_columns || entering_columns;
    match &node.content {
        NodeContent::Container { children } => {
            for c in children {
                validate_columns_node(c, child_inside)?;
            }
        }
        NodeContent::Table(spec) => {
            if let crate::TableDataSource::Inline { rows } = &spec.data {
                for row in rows {
                    for cell in row {
                        validate_columns_node(cell, child_inside)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GridTrack, NodeContent, TableDataSource, TableSpec};

    fn text(id: &str) -> SemanticNode {
        SemanticNode {
            id: id.into(),
            role: "body".into(),
            content: NodeContent::Text("hi".into()),
            ..Default::default()
        }
    }

    fn columns_container(id: &str, children: Vec<SemanticNode>) -> SemanticNode {
        SemanticNode {
            id: id.into(),
            role: "section".into(),
            layout: Some(LayoutHint::Columns {
                count: 2,
                gap: 12000,
            }),
            content: NodeContent::Container { children },
            ..Default::default()
        }
    }

    #[test]
    fn accepts_columns_with_in_flow_span() {
        let mut img = text("fig");
        img.role = "body".into();
        img.content = NodeContent::Image {
            src: "assets/images/a.png".into(),
            width: crate::Pt(10000),
            height: crate::Pt(10000),
        };
        img.column_span = ColumnSpan::All;
        let root = columns_container("body", vec![text("p1"), img, text("p2")]);
        validate_columns_tree(&root).unwrap();
    }

    #[test]
    fn rejects_span_outside_columns() {
        let mut n = text("fig");
        n.column_span = ColumnSpan::All;
        let err = validate_columns_tree(&n).unwrap_err();
        assert!(matches!(err, K2FError::ColumnSpanOutsideColumns { .. }));
    }

    #[test]
    fn rejects_nested_columns() {
        let inner = columns_container("inner", vec![text("p")]);
        let outer = columns_container("outer", vec![inner]);
        let err = validate_columns_tree(&outer).unwrap_err();
        assert!(matches!(err, K2FError::ColumnsNested { .. }));
    }

    #[test]
    fn rejects_invalid_count() {
        let mut n = columns_container("bad", vec![text("p")]);
        n.layout = Some(LayoutHint::Columns { count: 1, gap: 0 });
        let err = validate_columns_tree(&n).unwrap_err();
        assert!(matches!(err, K2FError::ColumnsInvalid { .. }));
    }

    #[test]
    fn rejects_span_on_columns_container() {
        let mut n = columns_container("c", vec![text("p")]);
        n.column_span = ColumnSpan::All;
        let err = validate_columns_tree(&n).unwrap_err();
        assert!(matches!(err, K2FError::ColumnsInvalid { .. }));
    }

    #[test]
    fn column_span_omits_none_in_json() {
        let n = text("p");
        let v: serde_json::Value = serde_json::to_value(&n).unwrap();
        assert!(v.get("column_span").is_none());
        let mut spanned = text("fig");
        spanned.column_span = ColumnSpan::All;
        let v: serde_json::Value = serde_json::to_value(&spanned).unwrap();
        assert_eq!(v["column_span"], "all");
    }

    #[test]
    fn columns_layout_roundtrips() {
        let n = columns_container("c", vec![text("p")]);
        let json = serde_json::to_string(&n).unwrap();
        let back: SemanticNode = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.layout,
            Some(LayoutHint::Columns {
                count: 2,
                gap: 12000
            })
        );
    }

    #[test]
    fn validates_through_table_cells() {
        let mut cell = text("cell");
        cell.column_span = ColumnSpan::All;
        let table = SemanticNode {
            id: "t".into(),
            role: "table".into(),
            content: NodeContent::Table(TableSpec {
                column_widths: vec![GridTrack::Fr { fr: 1 }],
                header_rows: 0,
                gap: 0,
                data: TableDataSource::Inline {
                    rows: vec![vec![cell]],
                },
            }),
            ..Default::default()
        };
        let err = validate_columns_tree(&table).unwrap_err();
        assert!(matches!(err, K2FError::ColumnSpanOutsideColumns { .. }));
    }
}
