use crate::coord::fmt_pt;
use crate::ir::{CellBorders, LineDash, TableBox, TableCell};
use crate::para_xml;

pub fn cell_edge_attrs(borders: &CellBorders) -> String {
    let mut s = String::new();
    push_edge(&mut s, "Top", &borders.top);
    push_edge(&mut s, "Left", &borders.left);
    push_edge(&mut s, "Bottom", &borders.bottom);
    push_edge(&mut s, "Right", &borders.right);
    s
}

pub(crate) fn table_xml(tbl: &TableBox, table_self: &str) -> String {
    let n_rows = tbl.rows.len();
    let header = tbl.header_rows.min(n_rows);
    let body = n_rows.saturating_sub(header);
    let ncols = tbl.col_widths_pt.len();
    let mut inner = String::new();
    for (c, w) in tbl.col_widths_pt.iter().enumerate() {
        inner.push_str(&format!(
            "      <Column Self=\"{table_self}Col{c}\" Name=\"{c}\" SingleColumnWidth=\"{}\"/>\n",
            fmt_pt(*w)
        ));
    }
    for (r, row) in tbl.rows.iter().enumerate() {
        inner.push_str(&format!(
            "      <Row Self=\"{table_self}Row{r}\" Name=\"{r}\" SingleRowHeight=\"{}\"/>\n",
            fmt_pt(row.height_pt)
        ));
    }
    let mut hts = 0usize;
    for (r, row) in tbl.rows.iter().enumerate() {
        for (c, cell) in row.cells.iter().enumerate() {
            inner.push_str(&cell_xml(table_self, r, c, cell, &mut hts));
        }
    }
    format!(
        r#"    <Table Self="{table_self}" HeaderRowCount="{header}" FooterRowCount="0" BodyRowCount="{body}" ColumnCount="{ncols}" AppliedTableStyle="TableStyle/$ID/[No table style]" TableDirection="LeftToRightDirection">
{inner}    </Table>
"#
    )
}

fn cell_xml(table_self: &str, r: usize, c: usize, cell: &TableCell, hts: &mut usize) -> String {
    let just = cell.align.justification();
    let vert = if cell.vert_center {
        "CenterAlign"
    } else {
        "TopAlign"
    };
    let fill = match &cell.fill_hex {
        Some(hex) => format!("FillColor=\"Color/k2f_{hex}\""),
        None => "FillColor=\"Swatch/None\"".into(),
    };
    let edges = cell_edge_attrs(&cell.borders);
    let paras = para_xml::write_paras(cell.align, &cell.runs, hts, false);
    format!(
        "      <Cell Self=\"{table_self}_r{r}c{c}\" Name=\"{c}:{r}\" RowSpan=\"1\" ColumnSpan=\"1\" AppliedCellStyle=\"CellStyle/$ID/[None]\" Justification=\"{just}\" VerticalJustification=\"{vert}\" {fill}{edges}>\n{paras}      </Cell>\n"
    )
}

fn push_edge(s: &mut String, name: &str, stroke: &Option<crate::ir::BorderStroke>) {
    match stroke {
        Some(st) => s.push_str(&format!(
            r#" {name}EdgeStrokeWeight="{}" {name}EdgeStrokeColor="Color/k2f_{}" {name}EdgeStrokeType="{}""#,
            fmt_pt(st.weight_pt),
            st.color_hex,
            stroke_type(st.dash),
        )),
        None => s.push_str(&format!(r#" {name}EdgeStrokeWeight="0""#)),
    }
}

fn stroke_type(dash: LineDash) -> &'static str {
    match dash {
        LineDash::Solid => "$ID/Solid",
        LineDash::Dash => "$ID/Dashed",
        LineDash::Dot => "$ID/Dotted",
    }
}
