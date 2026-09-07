use crate::ir::{ScriptPos, SlideElement, SlideIR, TextBox, TextRun};
use crate::xml::escape_xml;
use std::collections::BTreeMap;

pub(crate) fn textbox_sp_xml(
    tb: &TextBox,
    cnv_id: u32,
    hyperlink_rids: &BTreeMap<String, String>,
) -> String {
    let name = escape_xml(&tb.node_id);
    let body = txbody_inner(
        &tb.runs,
        tb.align,
        tb.bullet,
        tb.numbered,
        tb.preserve_whitespace,
        tb.line_spc_pts,
        tb.mar_l_emu,
        tb.list_start,
        hyperlink_rids,
        "      ",
    );
    let (lins, rins) = if tb.numbered || tb.bullet {
        (0, tb.r_ins_emu)
    } else {
        (tb.l_ins_emu, tb.r_ins_emu)
    };
    let overflow = if tb.wrap {
        ""
    } else {
        r#" vertOverflow="overflow" horzOverflow="overflow""#
    };
    format!(
        r#"    <p:sp>
      <p:nvSpPr>
        <p:cNvPr id="{id}" name="{name}"/>
        <p:cNvSpPr txBox="1"/>
        <p:nvPr/>
      </p:nvSpPr>
      <p:spPr>
        <a:xfrm>
          <a:off x="{x}" y="{y}"/>
          <a:ext cx="{cx}" cy="{cy}"/>
        </a:xfrm>
        <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        <a:noFill/>
        <a:ln><a:noFill/></a:ln>
      </p:spPr>
      <p:txBody>
        <a:bodyPr wrap="{wrap}" lIns="{lins}" tIns="{tins}" rIns="{rins}" bIns="0" rtlCol="0" anchor="t"{overflow}>
          <a:noAutofit/>
        </a:bodyPr>
        <a:lstStyle/>
{body}      </p:txBody>
    </p:sp>
"#,
        id = cnv_id,
        x = tb.x_emu,
        y = tb.y_emu,
        cx = tb.cx_emu,
        cy = tb.cy_emu,
        tins = tb.t_ins_emu,
        wrap = if tb.wrap { "square" } else { "none" },
    )
}

pub(crate) fn txbody_inner(
    runs: &[TextRun],
    align: crate::ir::TextAlign,
    bullet: bool,
    numbered: bool,
    preserve: bool,
    line_spc_pts: Option<i32>,
    mar_l_emu: i64,
    list_start: u32,
    hyperlink_rids: &BTreeMap<String, String>,
    indent: &str,
) -> String {
    let paras = paragraph_runs(runs);
    let mut body = String::new();
    for (i, para) in paras.iter().enumerate() {
        let list_on = i == 0;
        body.push_str(&paragraph_xml(
            para,
            align,
            list_on && bullet,
            list_on && numbered,
            preserve,
            line_spc_pts,
            if list_on { mar_l_emu } else { 0 },
            if list_on { list_start } else { 1 },
            hyperlink_rids,
            indent,
        ));
    }
    if paras.is_empty() {
        body.push_str(indent);
        body.push_str("<a:p/>\n");
    }
    body
}

fn paragraph_runs(runs: &[TextRun]) -> Vec<Vec<TextRun>> {
    let mut paras: Vec<Vec<TextRun>> = vec![Vec::new()];
    for run in runs {
        let mut rest = run.text.as_str();
        while let Some(i) = rest.find('\n') {
            let mut head = &rest[..i];
            if let Some(stripped) = head.strip_suffix('\r') {
                head = stripped;
            }
            if !head.is_empty() {
                let mut piece = run.clone();
                piece.text = head.to_string();
                paras.last_mut().unwrap().push(piece);
            }
            paras.push(Vec::new());
            rest = &rest[i + 1..];
        }
        if !rest.is_empty() {
            let mut piece = run.clone();
            piece.text = rest.to_string();
            paras.last_mut().unwrap().push(piece);
        }
    }
    paras
}

fn paragraph_xml(
    runs: &[TextRun],
    align: crate::ir::TextAlign,
    bullet: bool,
    numbered: bool,
    preserve: bool,
    line_spc_pts: Option<i32>,
    mar_l_emu: i64,
    list_start: u32,
    hyperlink_rids: &BTreeMap<String, String>,
    indent: &str,
) -> String {
    let mut p = String::new();
    p.push_str(indent);
    p.push_str("<a:p>\n");
    p.push_str(indent);
    p.push_str("  <a:pPr algn=\"");
    p.push_str(align_token(align));
    p.push_str("\"");
    if mar_l_emu > 0 {
        p.push_str(&format!(r#" marL="{mar_l_emu}" indent="-{mar_l_emu}""#));
    }
    p.push('>');
    if let Some(pts) = line_spc_pts {
        p.push_str(&format!(r#"<a:lnSpc><a:spcPts val="{pts}"/></a:lnSpc>"#));
    }
    if numbered || bullet {
        let clr = runs
            .first()
            .map(|r| r.color_hex.as_str())
            .unwrap_or("000001");
        p.push_str(&format!(r#"<a:buClr><a:srgbClr val="{clr}"/></a:buClr>"#));
    }
    if numbered {
        p.push_str(&format!(
            r#"<a:buFont typeface="Arial"/><a:buAutoNum type="arabicPeriod" startAt="{start}"/>"#,
            start = list_start.max(1),
        ));
    } else if bullet {
        p.push_str(r#"<a:buFont typeface="Arial"/><a:buChar char="•"/>"#);
    } else {
        p.push_str("<a:buNone/>");
    }
    p.push_str("</a:pPr>\n");
    for run in runs {
        p.push_str(indent);
        p.push_str("  ");
        p.push_str(&run_xml(run, preserve, hyperlink_rids));
    }
    p.push_str(indent);
    p.push_str("</a:p>\n");
    p
}

fn align_token(align: crate::ir::TextAlign) -> &'static str {
    match align {
        crate::ir::TextAlign::Left => "l",
        crate::ir::TextAlign::Center => "ctr",
        crate::ir::TextAlign::Right => "r",
        crate::ir::TextAlign::Justify => "just",
    }
}

fn run_xml(run: &TextRun, preserve_box: bool, hyperlink_rids: &BTreeMap<String, String>) -> String {
    let mut rpr = format!(
        r#"<a:rPr lang="en-US" sz="{sz}" dirty="0""#,
        sz = run.sz_hundredths_pt
    );
    if run.tracking_spc != 0 {
        rpr.push_str(&format!(r#" spc="{}""#, run.tracking_spc));
    }
    if run.bold {
        rpr.push_str(r#" b="1""#);
    }
    if run.italic {
        rpr.push_str(r#" i="1""#);
    }
    if run.underline {
        rpr.push_str(r#" u="sng""#);
    }
    if run.strike {
        rpr.push_str(r#" strike="sngStrike""#);
    }
    match run.script {
        ScriptPos::Super => rpr.push_str(r#" baseline="30000""#),
        ScriptPos::Sub => rpr.push_str(r#" baseline="-25000""#),
        ScriptPos::Baseline => {}
    }
    rpr.push('>');
    rpr.push_str(&format!(
        r#"<a:solidFill><a:srgbClr val="{}"/></a:solidFill>"#,
        run.color_hex
    ));
    let face = escape_xml(&run.font_name);
    rpr.push_str(&format!(
        r#"<a:latin typeface="{face}"/><a:ea typeface="{face}"/><a:cs typeface="{face}"/>"#
    ));
    if let Some(url) = &run.hyperlink {
        if let Some(rid) = hyperlink_rids.get(url) {
            rpr.push_str(&format!(r#"<a:hlinkClick r:id="{rid}"/>"#));
        }
    }
    rpr.push_str("</a:rPr>");
    let preserve = preserve_box
        || run.text.starts_with(char::is_whitespace)
        || run.text.ends_with(char::is_whitespace);
    let space = if preserve {
        r#" xml:space="preserve""#
    } else {
        ""
    };
    format!(
        "<a:r>{rpr}<a:t{space}>{}</a:t></a:r>\n",
        escape_xml(&run.text)
    )
}

pub(crate) fn collect_hyperlink_urls(slide: &SlideIR) -> Vec<String> {
    let mut urls = Vec::new();
    for el in &slide.elements {
        match el {
            SlideElement::TextBox(tb) => push_urls(&tb.runs, &mut urls),
            SlideElement::Table(t) => {
                for row in &t.rows {
                    for cell in &row.cells {
                        push_urls(&cell.runs, &mut urls);
                    }
                }
            }
            SlideElement::Shape(_) | SlideElement::Picture(_) | SlideElement::Raster(_) => {}
        }
    }
    urls
}

fn push_urls(runs: &[TextRun], urls: &mut Vec<String>) {
    for run in runs {
        if let Some(url) = &run.hyperlink {
            if !url.is_empty() && !urls.iter().any(|u| u == url) {
                urls.push(url.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ScriptPos, TextAlign, TextRun};

    fn run(text: &str) -> TextRun {
        TextRun {
            text: text.into(),
            font_name: "Times New Roman".into(),
            sz_hundredths_pt: 1200,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "000001".into(),
            hyperlink: None,
            script: ScriptPos::Baseline,
            tracking_spc: 0,
        }
    }

    #[test]
    fn numbered_paragraph_pins_bullet_color() {
        let xml = paragraph_xml(
            &[run("Dong")],
            TextAlign::Left,
            false,
            true,
            false,
            None,
            200_000,
            3,
            &BTreeMap::new(),
            "",
        );
        assert!(
            xml.contains(r#"<a:buClr><a:srgbClr val="000001"/></a:buClr>"#),
            "list numbers must not use theme Automatic, got {xml}"
        );
        assert!(xml.contains(r#"marL="200000""#), "{xml}");
        assert!(xml.contains(r#"indent="-200000""#), "{xml}");
        assert!(xml.contains(r#"startAt="3""#), "{xml}");
    }

    #[test]
    fn body_pr_emits_no_autofit_and_center_algn() {
        let tb = crate::ir::TextBox {
            node_id: "title".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 1_000_000,
            cy_emu: 200_000,
            runs: vec![run("Title")],
            align: TextAlign::Center,
            bullet: false,
            numbered: false,
            preserve_whitespace: false,
            wrap: true,
            line_spc_pts: None,
            t_ins_emu: 0,
            l_ins_emu: 0,
            r_ins_emu: 0,
            mar_l_emu: 0,
            list_start: 1,
        };
        let xml = textbox_sp_xml(&tb, 2, &BTreeMap::new());
        assert!(xml.contains("<a:noAutofit/>"), "{xml}");
        assert!(xml.contains(r#"wrap="square""#), "{xml}");
        assert!(xml.contains(r#"algn="ctr""#), "{xml}");
    }
}
