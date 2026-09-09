use crate::ir::{DocField, ScriptPos, TextAlign, TextBox, TextRun};
use crate::xml::{escape_xml, word_hex_color};
use std::collections::BTreeMap;

pub(crate) fn textbox_wsp_xml(tb: &TextBox, hyperlink_rids: &BTreeMap<String, String>) -> String {
    let body = txbx_content(tb, hyperlink_rids);
    let anchor = if tb.vert_center { "ctr" } else { "t" };
    let fill = match &tb.fill_hex {
        Some(hex) => super::drawing::solid_fill_xml(&word_hex_color(hex), tb.fill_alpha),
        None => "                    <a:noFill/>\n".into(),
    };
    let (l, t, r, b) = if tb.numbered || tb.bullet {
        // Marker column is a hanging indent on the paragraph, not shape padding.
        // Extra lIns plus numbering indent shrinks wrap width and clips citations.
        (0, tb.t_ins_emu, tb.r_ins_emu, tb.b_ins_emu)
    } else {
        (tb.l_ins_emu, tb.t_ins_emu, tb.r_ins_emu, tb.b_ins_emu)
    };
    let overflow = if tb.wrap {
        ""
    } else {
        r#" vertOverflow="overflow" horzOverflow="overflow""#
    };
    let geom = if tb.corner_emu <= 0 {
        "                    <a:prstGeom prst=\"rect\">\n                      <a:avLst/>\n                    </a:prstGeom>\n".to_string()
    } else {
        let adj = crate::shape::round_rect_adj(tb.corner_emu, tb.cx_emu, tb.cy_emu);
        format!(
            "                    <a:prstGeom prst=\"roundRect\">\n                      <a:avLst>\n                        <a:gd name=\"adj\" fmla=\"val {adj}\"/>\n                      </a:avLst>\n                    </a:prstGeom>\n"
        )
    };
    format!(
        r#"                <wps:wsp>
                  <wps:cNvSpPr txBox="1"/>
                  <wps:spPr>
                    <a:xfrm>
                      <a:off x="0" y="0"/>
                      <a:ext cx="{cx}" cy="{cy}"/>
                    </a:xfrm>
{geom}{fill}                    <a:ln>
                      <a:noFill/>
                    </a:ln>
                  </wps:spPr>
                  <wps:txbx>
                    <w:txbxContent>
{body}                    </w:txbxContent>
                  </wps:txbx>
                  <wps:bodyPr wrap="{wrap}" lIns="{l}" tIns="{t}" rIns="{r}" bIns="{b}" anchor="{anchor}"{overflow}>
                    <a:noAutofit/>
                  </wps:bodyPr>
                </wps:wsp>
"#,
        cx = tb.cx_emu,
        cy = tb.cy_emu,
        wrap = if tb.wrap { "square" } else { "none" },
        l = l,
        t = t,
        r = r,
        b = b,
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
    // CT_PPr is an xsd:sequence: numPr, then spacing, then ind, then jc.
    // Word (especially Mac) refuses to open the package if these are out of order.
    let mut ppr = String::from("                        <w:pPr>\n");
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
    if let Some(line) = tb.line_twips {
        ppr.push_str(&format!(
            r#"                          <w:spacing w:line="{line}" w:lineRule="exact"/>
"#
        ));
    }
    if tb.numbered || tb.bullet {
        let hang = crate::coord::emu_to_twips(tb.l_ins_emu).max(0);
        if hang > 0 {
            ppr.push_str(&format!(
                r#"                          <w:ind w:left="{hang}" w:hanging="{hang}"/>
"#
            ));
        }
    }
    ppr.push_str(&format!(
        r#"                          <w:jc w:val="{}"/>
"#,
        tb.align.jc_val()
    ));
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
    // CT_RPr sequence: rFonts, b, i, strike, color, spacing, sz, szCs, u, vertAlign.
    // w14:textFill is an extension and must come after the 2006 children.
    let color = word_hex_color(&run.color_hex);
    let mut s = format!(
        r#"                          <w:rPr>
                            <w:rFonts w:ascii="{f}" w:hAnsi="{f}" w:eastAsia="{f}" w:cs="{f}"/>
"#,
        f = escape_xml(&run.font_name),
    );
    if run.bold {
        s.push_str("                            <w:b/>\n");
    }
    if run.italic {
        s.push_str("                            <w:i/>\n");
    }
    if run.strike {
        s.push_str("                            <w:strike/>\n");
    }
    s.push_str(&format!(
        r#"                            <w:color w:val="{color}"/>
"#
    ));
    if run.tracking_twips != 0 {
        s.push_str(&format!(
            r#"                            <w:spacing w:val="{}"/>
"#,
            run.tracking_twips
        ));
    }
    s.push_str(&format!(
        r#"                            <w:sz w:val="{sz}"/>
                            <w:szCs w:val="{sz}"/>
"#,
        sz = run.sz_half_points,
    ));
    if run.underline {
        s.push_str(&format!(
            "                            <w:u w:val=\"single\" w:color=\"{color}\"/>\n"
        ));
    }
    match run.script {
        ScriptPos::Sub => {
            s.push_str("                            <w:vertAlign w:val=\"subscript\"/>\n")
        }
        ScriptPos::Super => {
            s.push_str("                            <w:vertAlign w:val=\"superscript\"/>\n")
        }
        ScriptPos::Baseline => {}
    }
    s.push_str(&format!(
        r#"                            <w14:textFill>
                              <w14:solidFill>
                                <w14:srgbClr w14:val="{color}"/>
                              </w14:solidFill>
                            </w14:textFill>
"#
    ));
    s.push_str("                          </w:rPr>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::TextBox;

    fn box_with(numbered: bool, line_twips: Option<i64>) -> TextBox {
        TextBox {
            node_id: "n".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 1,
            cy_emu: 1,
            runs: Vec::new(),
            align: TextAlign::Left,
            bullet: false,
            numbered,
            ilvl: 0,
            l_ins_emu: 0,
            t_ins_emu: 0,
            r_ins_emu: 0,
            b_ins_emu: 0,
            line_twips,
            vert_center: false,
            preserve_whitespace: false,
            relative_height: 1,
            fill_hex: None,
            fill_alpha: 255,
            wrap: true,
            corner_emu: 0,
        }
    }

    fn child_order(xml: &str, tags: &[&str]) {
        let mut last = 0usize;
        for tag in tags {
            let at = xml
                .find(tag)
                .unwrap_or_else(|| panic!("missing {tag} in {xml}"));
            assert!(at >= last, "{tag} out of order in {xml}");
            last = at;
        }
    }

    #[test]
    fn ppr_emits_numpr_then_spacing_then_jc() {
        let xml = paragraph_xml(&box_with(true, Some(240)), &[], &BTreeMap::new());
        child_order(&xml, &["<w:numPr>", "<w:spacing", "<w:jc "]);
    }

    #[test]
    fn ppr_emits_ind_between_spacing_and_jc_for_lists() {
        let mut tb = box_with(true, Some(240));
        tb.l_ins_emu = 635 * 360;
        let xml = paragraph_xml(&tb, &[], &BTreeMap::new());
        child_order(&xml, &["<w:numPr>", "<w:spacing", "<w:ind ", "<w:jc "]);
        assert!(xml.contains(r#"w:hanging="360""#), "{xml}");
    }

    #[test]
    fn body_pr_emits_no_autofit() {
        let mut tb = box_with(false, None);
        tb.align = TextAlign::Center;
        tb.wrap = true;
        let xml = textbox_wsp_xml(&tb, &BTreeMap::new());
        assert!(xml.contains("<a:noAutofit/>"), "{xml}");
        assert!(xml.contains(r#"wrap="square""#), "{xml}");
        assert!(xml.contains(r#"w:jc w:val="center""#), "{xml}");
    }

    #[test]
    fn rpr_follows_schema_order_with_w14_last() {
        let run = TextRun {
            text: "a".into(),
            font_name: "Calibri".into(),
            sz_half_points: 22,
            bold: true,
            italic: true,
            underline: true,
            strike: true,
            color_hex: "000001".into(),
            hyperlink: None,
            script: ScriptPos::Super,
            tracking_twips: 20,
            field: None,
        };
        let xml = rpr_xml(&run);
        child_order(
            &xml,
            &[
                "<w:rFonts",
                "<w:b/>",
                "<w:i/>",
                "<w:strike/>",
                "<w:color",
                "<w:spacing",
                "<w:sz ",
                "<w:szCs",
                "<w:u ",
                "<w:vertAlign",
                "<w14:textFill>",
            ],
        );
    }
}
