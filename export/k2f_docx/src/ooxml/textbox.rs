use crate::ir::{DocField, ScriptPos, TextAlign, TextBox, TextRun};
use crate::xml::escape_xml;
use std::collections::BTreeMap;

pub(crate) fn textbox_wsp_xml(tb: &TextBox, hyperlink_rids: &BTreeMap<String, String>) -> String {
    let body = txbx_content(tb, hyperlink_rids);
    let anchor = if tb.vert_center { "ctr" } else { "t" };
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
{body}                    </w:txbxContent>
                  </wps:txbx>
                  <wps:bodyPr wrap="square" lIns="{l}" tIns="{t}" rIns="{r}" bIns="{b}" anchor="{anchor}"/>
                </wps:wsp>
"#,
        cx = tb.cx_emu,
        cy = tb.cy_emu,
        l = tb.l_ins_emu,
        t = tb.t_ins_emu,
        r = tb.r_ins_emu,
        b = tb.b_ins_emu,
    )
}

fn txbx_content(tb: &TextBox, hyperlink_rids: &BTreeMap<String, String>) -> String {
    let paras = split_paragraphs(&tb.runs);
    let mut xml = String::new();
    for para in &paras {
        xml.push_str(&paragraph_xml(tb, para, hyperlink_rids));
    }
    if paras.is_empty() {
        xml.push_str(&paragraph_xml(tb, &[], hyperlink_rids));
    }
    xml
}

pub(crate) fn txbx_paragraphs(tb: &TextBox, hyperlink_rids: &BTreeMap<String, String>) -> String {
    txbx_content(tb, hyperlink_rids)
}

fn split_paragraphs(runs: &[TextRun]) -> Vec<Vec<TextRun>> {
    let mut paras: Vec<Vec<TextRun>> = vec![Vec::new()];
    for run in runs {
        let mut rest = run.text.as_str();
        while let Some(i) = rest.find('\n') {
            let mut head = &rest[..i];
            if let Some(s) = head.strip_suffix('\r') {
                head = s;
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
        } else if run.field.is_some() {
            paras.last_mut().unwrap().push(run.clone());
        }
    }
    paras
}

fn paragraph_xml(
    tb: &TextBox,
    runs: &[TextRun],
    hyperlink_rids: &BTreeMap<String, String>,
) -> String {
    let mut ppr = format!(
        r#"                        <w:pPr>
                          <w:jc w:val="{}"/>
"#,
        tb.align.jc_val()
    );
    if let Some(line) = tb.line_twips {
        ppr.push_str(&format!(
            r#"                          <w:spacing w:line="{line}" w:lineRule="exact"/>
"#
        ));
    }
    if tb.numbered || tb.bullet {
        let num_id = if tb.numbered { 2 } else { 1 };
        ppr.push_str(&format!(
            r#"                          <w:numPr>
                            <w:ilvl w:val="{}"/>
                            <w:numId w:val="{num_id}"/>
                          </w:numPr>
"#,
            tb.ilvl
        ));
    }
    ppr.push_str("                        </w:pPr>\n");
    let mut body = String::new();
    for run in runs {
        body.push_str(&run_xml(
            run,
            tb.preserve_whitespace,
            tb.align,
            hyperlink_rids,
        ));
    }
    format!(
        r#"                      <w:p>
{ppr}{body}                      </w:p>
"#
    )
}

fn run_xml(
    run: &TextRun,
    preserve_box: bool,
    _align: TextAlign,
    hyperlink_rids: &BTreeMap<String, String>,
) -> String {
    if let Some(field) = run.field {
        return field_xml(run, field);
    }
    let inner = styled_t(run, preserve_box);
    if let Some(url) = &run.hyperlink {
        if let Some(rid) = hyperlink_rids.get(url) {
            return format!(
                r#"                        <w:hyperlink r:id="{rid}">
{inner}                        </w:hyperlink>
"#
            );
        }
    }
    inner
}

fn field_xml(run: &TextRun, field: DocField) -> String {
    let instr = match field {
        DocField::Page => " PAGE ",
        DocField::NumPages => " NUMPAGES ",
    };
    let rpr = rpr_xml(run);
    format!(
        r#"                        <w:r>
{rpr}                          <w:fldChar w:fldCharType="begin"/>
                        </w:r>
                        <w:r>
{rpr}                          <w:instrText xml:space="preserve">{instr}</w:instrText>
                        </w:r>
                        <w:r>
{rpr}                          <w:fldChar w:fldCharType="separate"/>
                        </w:r>
                        <w:r>
{rpr}                          <w:t xml:space="preserve">{}</w:t>
                        </w:r>
                        <w:r>
{rpr}                          <w:fldChar w:fldCharType="end"/>
                        </w:r>
"#,
        escape_xml(&run.text)
    )
}

fn styled_t(run: &TextRun, preserve_box: bool) -> String {
    let rpr = rpr_xml(run);
    let space = if preserve_box
        || run.text.starts_with(' ')
        || run.text.ends_with(' ')
        || run.text.contains("  ")
    {
        r#" xml:space="preserve""#
    } else {
        ""
    };
    format!(
        r#"                        <w:r>
{rpr}                          <w:t{space}>{}</w:t>
                        </w:r>
"#,
        escape_xml(&run.text)
    )
}

fn rpr_xml(run: &TextRun) -> String {
    let mut s = format!(
        r#"                          <w:rPr>
                            <w:rFonts w:ascii="{f}" w:hAnsi="{f}" w:eastAsia="{f}" w:cs="{f}"/>
                            <w:sz w:val="{sz}"/>
                            <w:szCs w:val="{sz}"/>
                            <w:color w:val="{}"/>
"#,
        run.color_hex,
        f = escape_xml(&run.font_name),
        sz = run.sz_half_points,
    );
    if run.bold {
        s.push_str("                            <w:b/>\n");
    }
    if run.italic {
        s.push_str("                            <w:i/>\n");
    }
    if run.underline {
        s.push_str(r#"                            <w:u w:val="single"/>"#);
        s.push('\n');
    }
    if run.strike {
        s.push_str("                            <w:strike/>\n");
    }
    match run.script {
        ScriptPos::Sub => {
            s.push_str(r#"                            <w:vertAlign w:val="subscript"/>"#)
        }
        ScriptPos::Super => {
            s.push_str(r#"                            <w:vertAlign w:val="superscript"/>"#)
        }
        ScriptPos::Baseline => {}
    }
    if run.script != ScriptPos::Baseline {
        s.push('\n');
    }
    if run.tracking_twips != 0 {
        s.push_str(&format!(
            r#"                            <w:spacing w:val="{}"/>
"#,
            run.tracking_twips
        ));
    }
    s.push_str("                          </w:rPr>\n");
    s
}
