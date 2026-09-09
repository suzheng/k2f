use crate::ir::{BorderStroke, LineDash, TableBox, TableCell};
use crate::ooxml::textbox::txbody_inner;
use crate::xml::escape_xml;
use std::collections::BTreeMap;

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
        cell.align,
        false,
        false,
        cell.preserve_whitespace,
        None,
        0,
        1,
        hyperlink_rids,
        "                ",
    );
    let fill = match &cell.fill_hex {
        Some(hex) => format!(
            "<a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill>",
            escape_xml(hex)
        ),
        None => "<a:solidFill><a:srgbClr val=\"FFFFFE\"/></a:solidFill>".into(),
    };
    let cell_id = escape_xml(&cell.node_id);
    let anchor = if cell.vert_center { "ctr" } else { "t" };
    format!(
        r#"            <a:tc>
              <a:txBody>
                <a:bodyPr wrap="square" lIns="0" tIns="0" rIns="0" bIns="0" rtlCol="0" anchor="{anchor}"/>
                <a:lstStyle/>
{body}              </a:txBody>
              <a:tcPr marL="0" marR="0" marT="0" marB="0">
                {ln_l}
                {ln_r}
                {ln_t}
                {ln_b}
                {fill}
              </a:tcPr>
            </a:tc><!--{cell_id}-->
"#,
        ln_l = ln_xml("lnL", cell.borders.left.as_ref()),
        ln_r = ln_xml("lnR", cell.borders.right.as_ref()),
        ln_t = ln_xml("lnT", cell.borders.top.as_ref()),
        ln_b = ln_xml("lnB", cell.borders.bottom.as_ref()),
    )
}

fn ln_xml(tag: &str, stroke: Option<&BorderStroke>) -> String {
    match stroke {
        Some(s) => {
            let dash = match s.dash {
                LineDash::Solid => "solid",
                LineDash::Dash => "dash",
                LineDash::Dot => "sysDot",
            };
            format!(
                r#"<a:{tag} w="{w}"><a:solidFill><a:srgbClr val="{hex}"/></a:solidFill><a:prstDash val="{dash}"/></a:{tag}>"#,
                w = s.w_emu,
                hex = escape_xml(&s.color_hex),
            )
        }
        None => format!(r#"<a:{tag}><a:noFill/></a:{tag}>"#),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{CellBorders, TableCell, TextAlign};

    fn dummy_cell(align: TextAlign, borders: CellBorders) -> TableCell {
        TableCell {
            node_id: "c".into(),
            runs: Vec::new(),
            align,
            fill_hex: None,
            preserve_whitespace: false,
            borders,
            vert_center: false,
        }
    }

    #[test]
    fn no_lock_border_emits_nofill_not_d0d0d0() {
        let xml = cell_xml(
            &dummy_cell(TextAlign::Left, CellBorders::default()),
            &BTreeMap::new(),
        );
        assert!(
            !xml.contains("D0D0D0"),
            "must not fake four-side #D0D0D0, got {xml}"
        );
        for edge in ["lnL", "lnR", "lnT", "lnB"] {
            assert!(
                xml.contains(&format!("<a:{edge}><a:noFill/></a:{edge}>")),
                "missing noFill on {edge}, got {xml}"
            );
        }
    }

    #[test]
    fn right_align_emits_algn_r() {
        let xml = cell_xml(
            &dummy_cell(TextAlign::Right, CellBorders::default()),
            &BTreeMap::new(),
        );
        assert!(
            xml.contains(r#"algn="r""#),
            "right-aligned lock cell must not be hardcoded left, got {xml}"
        );
    }

    #[test]
    fn lock_bottom_edge_keeps_color() {
        let borders = CellBorders {
            bottom: Some(BorderStroke {
                color_hex: "1A73E8".into(),
                w_emu: 12700,
                dash: LineDash::Solid,
            }),
            ..CellBorders::default()
        };
        let xml = cell_xml(&dummy_cell(TextAlign::Left, borders), &BTreeMap::new());
        assert!(
            xml.contains(r#"<a:lnB w="12700">"#),
            "bottom width, got {xml}"
        );
        assert!(xml.contains("1A73E8"), "lock color, got {xml}");
        assert!(
            xml.contains("<a:lnL><a:noFill/></a:lnL>"),
            "other edges stay empty, got {xml}"
        );
    }

    #[test]
    fn vert_center_emits_anchor_ctr() {
        let mut cell = dummy_cell(TextAlign::Left, CellBorders::default());
        cell.vert_center = true;
        let xml = cell_xml(&cell, &BTreeMap::new());
        assert!(
            xml.contains(r#"anchor="ctr""#),
            "centered lock cell must not hardcode anchor=t, got {xml}"
        );
    }
}
