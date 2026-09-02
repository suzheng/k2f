use crate::ir::{SlideElement, SlideIR, TextBox, TextRun};
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
        hyperlink_rids,
        "      ",
    );
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
        <a:bodyPr wrap="square" lIns="0" tIns="0" rIns="0" bIns="0" rtlCol="0" anchor="t"/>
        <a:lstStyle/>
{body}      </p:txBody>
    </p:sp>
"#,
        id = cnv_id,
        x = tb.x_emu,
        y = tb.y_emu,
        cx = tb.cx_emu,
        cy = tb.cy_emu,
    )
}

pub(crate) fn txbody_inner(
    runs: &[TextRun],
    align: crate::ir::TextAlign,
    bullet: bool,
    numbered: bool,
    preserve: bool,
    hyperlink_rids: &BTreeMap<String, String>,
    indent: &str,
) -> String {
    let paras = paragraph_runs(runs);
    let mut body = String::new();
    for para in &paras {
        body.push_str(&paragraph_xml(
            para,
            align,
            bullet,
            numbered,
            preserve,
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
    hyperlink_rids: &BTreeMap<String, String>,
    indent: &str,
) -> String {
    let mut p = String::new();
    p.push_str(indent);
    p.push_str("<a:p>\n");
    p.push_str(indent);
    p.push_str("  <a:pPr algn=\"");
    p.push_str(align_token(align));
    p.push_str("\">");
    if numbered {
        p.push_str(r#"<a:buFont typeface="Arial"/><a:buAutoNum type="arabicPeriod"/>"#);
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
    }
}

fn run_xml(run: &TextRun, preserve_box: bool, hyperlink_rids: &BTreeMap<String, String>) -> String {
    let mut rpr = format!(
        r#"<a:rPr lang="en-US" sz="{sz}" dirty="0""#,
        sz = run.sz_hundredths_pt
    );
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
