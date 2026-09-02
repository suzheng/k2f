use crate::ir::{TableBox, TableCell, TextAlign};
use crate::ooxml::textbox::txbody_inner;
use crate::xml::escape_xml;
use std::collections::BTreeMap;

const CELL_LN: &str =
    r#"<a:solidFill><a:srgbClr val="D0D0D0"/></a:solidFill><a:prstDash val="solid"/>"#;

pub(crate) fn table_graphic_frame_xml(
    table: &TableBox,
    cnv_id: u32,
    hyperlink_rids: &BTreeMap<String, String>,
) -> String {
    let name = escape_xml(&table.node_id);
    let mut grid = String::new();
    for w in &table.col_widths_emu {
        grid.push_str(&format!("            <a:gridCol w=\"{w}\"/>\n"));
    }
    let mut rows = String::new();
    for row in &table.rows {
        rows.push_str(&format!("          <a:tr h=\"{}\">\n", row.height_emu));
        for cell in &row.cells {
            rows.push_str(&cell_xml(cell, hyperlink_rids));
        }
        rows.push_str("          </a:tr>\n");
    }
    format!(
        r#"    <p:graphicFrame>
      <p:nvGraphicFramePr>
        <p:cNvPr id="{id}" name="{name}"/>
        <p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr>
        <p:nvPr/>
      </p:nvGraphicFramePr>
      <p:xfrm>
        <a:off x="{x}" y="{y}"/>
        <a:ext cx="{cx}" cy="{cy}"/>
      </p:xfrm>
      <a:graphic>
        <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">
          <a:tbl>
            <a:tblPr/>
            <a:tblGrid>
{grid}            </a:tblGrid>
{rows}          </a:tbl>
        </a:graphicData>
      </a:graphic>
    </p:graphicFrame>
"#,
        id = cnv_id,
        x = table.x_emu,
        y = table.y_emu,
        cx = table.cx_emu,
        cy = table.cy_emu,
    )
}

fn cell_xml(cell: &TableCell, hyperlink_rids: &BTreeMap<String, String>) -> String {
    let body = txbody_inner(
        &cell.runs,
        TextAlign::Left,
        false,
        false,
        cell.preserve_whitespace,
        hyperlink_rids,
        "                ",
    );
    let fill = match &cell.fill_hex {
        Some(hex) => format!("<a:solidFill><a:srgbClr val=\"{hex}\"/></a:solidFill>"),
        None => String::new(),
    };
    let cell_id = escape_xml(&cell.node_id);
    format!(
        r#"            <a:tc>
              <a:txBody>
                <a:bodyPr wrap="square" lIns="0" tIns="0" rIns="0" bIns="0" rtlCol="0" anchor="t"/>
                <a:lstStyle/>
{body}              </a:txBody>
              <a:tcPr marL="0" marR="0" marT="0" marB="0">
                <a:lnL w="6350">{CELL_LN}</a:lnL>
                <a:lnR w="6350">{CELL_LN}</a:lnR>
                <a:lnT w="6350">{CELL_LN}</a:lnT>
                <a:lnB w="6350">{CELL_LN}</a:lnB>
                {fill}
              </a:tcPr>
            </a:tc><!--{cell_id}-->
"#
    )
}
