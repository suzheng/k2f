use super::cell::cell_borders;
use crate::ir::{BorderStroke, CellBorders, TableBox, TableCell, TextAlign, TextBox};
use crate::ooxml;
use crate::xml::escape_xml;
use k2f_core::Border;
use std::collections::BTreeMap;

pub fn table_cell_wml(align: TextAlign, border: Option<&Border>) -> String {
    let cell = TableCell {
        node_id: "cell".into(),
        width_twips: 1440,
        runs: Vec::new(),
        align,
        fill_hex: None,
        preserve_whitespace: false,
        borders: cell_borders(border).unwrap_or_default(),
    };
    format!(
        r#"<root xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
{}</root>"#,
        tc_xml(&cell, 1440, &BTreeMap::new())
    )
}

pub(crate) fn table_anchor(
    tbl: &TableBox,
    doc_pr_id: u32,
    hyperlink_rids: &BTreeMap<String, String>,
) -> String {
    ooxml::wp_anchor(
        tbl.x_emu,
        tbl.y_emu,
        tbl.cx_emu,
        tbl.cy_emu,
        tbl.relative_height,
        false,
        doc_pr_id,
        &tbl.node_id,
        "http://schemas.microsoft.com/office/word/2010/wordprocessingShape",
        &table_wsp_xml(tbl, hyperlink_rids),
    )
}

fn table_wsp_xml(tbl: &TableBox, hyperlink_rids: &BTreeMap<String, String>) -> String {
    format!(
        r#"                <wps:wsp>
                  <wps:cNvSpPr txBox="1"/>
                  <wps:spPr>
                    <a:xfrm>
                      <a:off x="0" y="0"/>
                      <a:ext cx="{cx}" cy="{cy}"/>
                    </a:xfrm>
                    <a:prstGeom prst="rect">
                      <a:avLst/>
                    </a:prstGeom>
                    <a:noFill/>
                    <a:ln>
                      <a:noFill/>
                    </a:ln>
                  </wps:spPr>
                  <wps:txbx>
                    <w:txbxContent>
{tbl_xml}                      <w:p/>
                    </w:txbxContent>
                  </wps:txbx>
                  <wps:bodyPr wrap="square" lIns="0" tIns="0" rIns="0" bIns="0" anchor="t"/>
                </wps:wsp>
"#,
        cx = tbl.cx_emu,
        cy = tbl.cy_emu,
        tbl_xml = tbl_xml(tbl, hyperlink_rids),
    )
}

fn tbl_xml(tbl: &TableBox, hyperlink_rids: &BTreeMap<String, String>) -> String {
    let mut grid = String::new();
    for w in &tbl.col_widths_twips {
        grid.push_str(&format!(
            "                          <w:gridCol w:w=\"{w}\"/>\n"
        ));
    }
    let mut rows = String::new();
    for row in &tbl.rows {
        rows.push_str(&tr_xml(row, &tbl.col_widths_twips, hyperlink_rids));
    }
    format!(
        r#"                      <w:tbl>
                        <w:tblPr>
                          <w:tblW w:w="{w}" w:type="dxa"/>
                          <w:tblLayout w:type="fixed"/>
                          <w:tblCellMar>
                            <w:top w:w="0" w:type="dxa"/>
                            <w:left w:w="0" w:type="dxa"/>
                            <w:bottom w:w="0" w:type="dxa"/>
                            <w:right w:w="0" w:type="dxa"/>
                          </w:tblCellMar>
                        </w:tblPr>
                        <w:tblGrid>
{grid}                        </w:tblGrid>
{rows}                      </w:tbl>
"#,
        w = tbl.width_twips,
    )
}

fn tr_xml(
    row: &crate::ir::TableRow,
    col_widths: &[i64],
    hyperlink_rids: &BTreeMap<String, String>,
) -> String {
    let mut cells = String::new();
    for (i, cell) in row.cells.iter().enumerate() {
        let w = col_widths.get(i).copied().unwrap_or(cell.width_twips);
        cells.push_str(&tc_xml(cell, w, hyperlink_rids));
    }
    format!(
        r#"                        <w:tr>
                          <w:trPr>
                            <w:trHeight w:val="{h}" w:hRule="atLeast"/>
                          </w:trPr>
{cells}                        </w:tr>
"#,
        h = row.height_twips,
    )
}

fn tc_xml(cell: &TableCell, width_twips: i64, hyperlink_rids: &BTreeMap<String, String>) -> String {
    let shd = match &cell.fill_hex {
        Some(hex) => format!(
            "                              <w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"{hex}\"/>\n"
        ),
        None => String::new(),
    };
    let dummy = TextBox {
        node_id: cell.node_id.clone(),
        x_emu: 0,
        y_emu: 0,
        cx_emu: 0,
        cy_emu: 0,
        runs: cell.runs.clone(),
        align: cell.align,
        bullet: false,
        numbered: false,
        ilvl: 0,
        l_ins_emu: 0,
        t_ins_emu: 0,
        r_ins_emu: 0,
        b_ins_emu: 0,
        line_twips: None,
        vert_center: false,
        preserve_whitespace: cell.preserve_whitespace,
        relative_height: 0,
    };
    format!(
        r#"                          <w:tc>
                            <w:tcPr>
                              <w:tcW w:w="{width_twips}" w:type="dxa"/>
{borders}{shd}                            </w:tcPr>
{paras}                          </w:tc>
"#,
        borders = tc_borders_xml(&cell.borders),
        paras = ooxml::txbx_paragraphs(&dummy, hyperlink_rids),
    )
}

fn tc_borders_xml(b: &CellBorders) -> String {
    format!(
        r#"                              <w:tcBorders>
                                {top}
                                {left}
                                {bottom}
                                {right}
                              </w:tcBorders>
"#,
        top = edge_xml("top", b.top.as_ref()),
        left = edge_xml("left", b.left.as_ref()),
        bottom = edge_xml("bottom", b.bottom.as_ref()),
        right = edge_xml("right", b.right.as_ref()),
    )
}

fn edge_xml(name: &str, stroke: Option<&BorderStroke>) -> String {
    match stroke {
        Some(s) => format!(
            r#"<w:{name} w:val="{val}" w:sz="{sz}" w:space="0" w:color="{color}"/>"#,
            val = s.val,
            sz = s.sz,
            color = escape_xml(&s.color_hex),
        ),
        None => format!(r#"<w:{name} w:val="nil"/>"#),
    }
}
