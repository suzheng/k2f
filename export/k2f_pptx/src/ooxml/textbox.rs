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
        tb.last_line_spc_pts,
        tb.spc_aft_pts,
        tb.mar_l_emu,
        tb.list_start,
        hyperlink_rids,
        "      ",
    );
    let (lins, rins) = (tb.l_ins_emu, tb.r_ins_emu);
    let (anchor, tins) = body_anchor_tins(tb);
    // Host metrics that are wider than rustybuzz wrap an extra line in a
    // lock-tight frame. Clip is the DrawingML default in some hosts.
    let overflow = r#" vertOverflow="overflow" horzOverflow="overflow""#;
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
        <a:bodyPr wrap="{wrap}" lIns="{lins}" tIns="{tins}" rIns="{rins}" bIns="0" rtlCol="0" anchor="{anchor}"{overflow}>
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
        wrap = if tb.wrap { "square" } else { "none" },
    )
}

/// Compact pills (~1.9× face) use `anchor=ctr`. Tall boxes keep lock `tIns`
/// because Impress/Writer often ignore DrawingML `anchor=ctr`.
fn body_anchor_tins(tb: &TextBox) -> (&'static str, i64) {
    if tb.vert_center && box_is_compact_center(tb) {
        ("ctr", 0)
    } else {
        ("t", tb.t_ins_emu)
    }
}

fn box_is_compact_center(tb: &TextBox) -> bool {
    let hundredths = tb
        .runs
        .iter()
        .filter(|r| r.text != "\n" && !r.text.is_empty())
        .map(|r| i64::from(r.sz_hundredths_pt))
        .max()
        .unwrap_or(1200);
    // 1pt = 12700 EMU; sz is hundredths of a point.
    let face_emu = hundredths.saturating_mul(127);
    face_emu > 0 && tb.cy_emu <= face_emu.saturating_mul(5) / 2
}

pub(crate) fn txbody_inner(
    runs: &[TextRun],
    align: crate::ir::TextAlign,
    bullet: bool,
    numbered: bool,
    preserve: bool,
    line_spc_pts: Option<i32>,
    last_line_spc_pts: Option<i32>,
    spc_aft_pts: Option<i32>,
    mar_l_emu: i64,
    list_start: u32,
    hyperlink_rids: &BTreeMap<String, String>,
    indent: &str,
) -> String {
    // Lock-wrapped lists pin `\n` + NBSP pad. New `<a:p>` resets hanging so
    // continuation starts in the marker gutter. Keep one paragraph + `<a:br/>`.
    let paras = if bullet || numbered {
        vec![runs.to_vec()]
    } else {
        paragraph_runs(runs)
    };
    let mut body = String::new();
    let n = paras.len();
    for (i, para) in paras.iter().enumerate() {
        let list_on = i == 0;
        let last = i + 1 == n;
        // Stacked one-line paras: face-size lnSpc so PowerPoint does not add
        // lock pitch *and* spcAft. Impress ignores that lnSpc; spcAft is the gap.
        let stacked = n > 1 && spc_aft_pts.is_some();
        let spc = if stacked {
            last_line_spc_pts.or(line_spc_pts)
        } else if last && n > 1 {
            last_line_spc_pts.or(line_spc_pts)
        } else {
            line_spc_pts
        };
        body.push_str(&paragraph_xml(
            para,
            align,
            list_on && bullet,
            list_on && numbered,
            preserve,
            spc,
            if last { None } else { spc_aft_pts },
            n == 1,
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
    _bullet: bool,
    _numbered: bool,
    preserve: bool,
    line_spc_pts: Option<i32>,
    spc_aft_pts: Option<i32>,
    wrap_pct: bool,
    _mar_l_emu: i64,
    _list_start: u32,
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
    p.push('>');
    if let Some(pts) = line_spc_pts {
        p.push_str(&ln_spc_xml(pts, runs, wrap_pct));
    }
    if let Some(aft) = spc_aft_pts {
        p.push_str(&format!(
            r#"<a:spcAft><a:spcPts val="{aft}"/></a:spcAft>"#
        ));
    }
    // Markers are literal text runs (see prepend_literal_*). Native
    // a:buChar / a:buAutoNum in floating frames often collapse or renumber
    // by host document order.
    p.push_str("<a:buNone/>");
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

/// Impress honors `spcPct` on wrapped runs and ignores `spcPts` (FIX). Use
/// percent of the run face when this is real lock leading (not a one-line pin).
fn ln_spc_xml(pts: i32, runs: &[TextRun], prefer_pct: bool) -> String {
    if prefer_pct {
        if let Some(sz) = runs.iter().map(|r| r.sz_hundredths_pt).max() {
            if sz > 0 && pts.saturating_mul(10) > sz.saturating_mul(11) {
                let pct = (i64::from(pts) * 100_000 / i64::from(sz)).clamp(1, 1_000_000);
                return format!(r#"<a:lnSpc><a:spcPct val="{pct}"/></a:lnSpc>"#);
            }
        }
    }
    format!(r#"<a:lnSpc><a:spcPts val="{pts}"/></a:lnSpc>"#)
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
    let text = run.text.replace('\u{2028}', "\n");
    if !text.contains('\n') {
        return drawing_run_t(&rpr, &run.text, preserve_box);
    }
    let mut out = String::new();
    let mut first = true;
    for piece in text.split('\n') {
        if !first {
            out.push_str(&format!("<a:br>{rpr}</a:br>\n"));
        }
        first = false;
        if !piece.is_empty() {
            out.push_str(&drawing_run_t(&rpr, piece, preserve_box));
        }
    }
    out
}

fn drawing_run_t(rpr: &str, text: &str, preserve_box: bool) -> String {
    let preserve = preserve_box
        || text.starts_with(char::is_whitespace)
        || text.ends_with(char::is_whitespace);
    let space = if preserve {
        r#" xml:space="preserve""#
    } else {
        ""
    };
    format!("<a:r>{rpr}<a:t{space}>{}</a:t></a:r>\n", escape_xml(text))
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
    fn list_wrap_stays_one_paragraph_with_break() {
        let tb = crate::ir::TextBox {
            node_id: "item".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 1_000_000,
            cy_emu: 200_000,
            runs: vec![
                run("•\u{00A0}Senior managers"),
                run("\u{2028}"),
                run("\u{00A0}\u{00A0}leads."),
            ],
            align: TextAlign::Left,
            bullet: true,
            numbered: false,
            preserve_whitespace: false,
            wrap: false,
            line_spc_pts: Some(1820),
            last_line_spc_pts: None,
            spc_aft_pts: None,
            t_ins_emu: 0,
            vert_center: false,
            l_ins_emu: 0,
            r_ins_emu: 0,
            mar_l_emu: 185_458,
            list_start: 1,
        };
        let xml = textbox_sp_xml(&tb, 2, &BTreeMap::new());
        assert_eq!(xml.matches("<a:p>").count(), 1, "{xml}");
        assert!(xml.contains("<a:br>"), "{xml}");
        assert!(xml.contains("leads."), "{xml}");
        assert!(!xml.contains("marL="), "{xml}");
    }

    #[test]
    fn numbered_paragraph_uses_literal_marker_not_autonum() {
        let xml = paragraph_xml(
            &[run("3.\u{00A0}Dong")],
            TextAlign::Left,
            false,
            true,
            false,
            None,
            None,
            false,
            200_000,
            3,
            &BTreeMap::new(),
            "",
        );
        assert!(xml.contains("<a:buNone/>"), "{xml}");
        assert!(!xml.contains("buAutoNum"), "{xml}");
        assert!(!xml.contains("marL="), "{xml}");
        assert!(!xml.contains("indent="), "{xml}");
        assert!(xml.contains(">3."), "{xml}");
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
            last_line_spc_pts: None,
            spc_aft_pts: None,
            t_ins_emu: 0,
            vert_center: false,
            l_ins_emu: 0,
            r_ins_emu: 0,
            mar_l_emu: 0,
            list_start: 1,
        };
        let xml = textbox_sp_xml(&tb, 2, &BTreeMap::new());
        assert!(xml.contains("<a:noAutofit/>"), "{xml}");
        assert!(xml.contains(r#"wrap="square""#), "{xml}");
        assert!(xml.contains(r#"algn="ctr""#), "{xml}");
        assert!(xml.contains(r#"vertOverflow="overflow""#), "{xml}");
        assert!(xml.contains(r#"horzOverflow="overflow""#), "{xml}");
    }

    #[test]
    fn vert_center_emits_anchor_ctr_and_zero_tins() {
        let tb = crate::ir::TextBox {
            node_id: "cell".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 1_000_000,
            cy_emu: 240_000, // ~19pt, 12pt face → compact pill
            runs: vec![run("Approved")],
            align: TextAlign::Left,
            bullet: false,
            numbered: false,
            preserve_whitespace: false,
            wrap: true,
            line_spc_pts: None,
            last_line_spc_pts: None,
            spc_aft_pts: None,
            t_ins_emu: 80_000,
            vert_center: true,
            l_ins_emu: 0,
            r_ins_emu: 0,
            mar_l_emu: 0,
            list_start: 1,
        };
        let xml = textbox_sp_xml(&tb, 2, &BTreeMap::new());
        assert!(xml.contains(r#"anchor="ctr""#), "{xml}");
        assert!(
            xml.contains(r#"tIns="0""#),
            "compact centered pill must not keep lock tIns, got {xml}"
        );
    }

    #[test]
    fn tall_centered_box_emits_tins_not_anchor_ctr() {
        let tb = crate::ir::TextBox {
            node_id: "cell".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 1_000_000,
            cy_emu: 540_000, // ~42pt cell
            runs: vec![run("01 Identity")],
            align: TextAlign::Left,
            bullet: false,
            numbered: false,
            preserve_whitespace: false,
            wrap: true,
            line_spc_pts: Some(1820),
            last_line_spc_pts: None,
            spc_aft_pts: None,
            t_ins_emu: 80_000,
            vert_center: true,
            l_ins_emu: 0,
            r_ins_emu: 0,
            mar_l_emu: 0,
            list_start: 1,
        };
        let xml = textbox_sp_xml(&tb, 2, &BTreeMap::new());
        assert!(xml.contains(r#"anchor="t""#), "{xml}");
        assert!(
            xml.contains(r#"tIns="80000""#),
            "tall cell must keep lock first-line pad, got {xml}"
        );
    }

    #[test]
    fn bold_run_emits_b_and_family_name() {
        let mut r = run("Hello");
        r.font_name = "Roboto".into();
        r.bold = true;
        let xml = paragraph_xml(
            &[r],
            TextAlign::Left,
            false,
            false,
            false,
            None,
            None,
            false,
            0,
            1,
            &BTreeMap::new(),
            "",
        );
        assert!(xml.contains(r#"b="1""#), "{xml}");
        assert!(xml.contains(r#"typeface="Roboto""#), "{xml}");
    }

    #[test]
    fn last_pinned_paragraph_uses_face_spacing() {
        let mut r1 = run("line one");
        r1.text = "line one\nline two".into();
        let tb = crate::ir::TextBox {
            node_id: "quote".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 1_000_000,
            cy_emu: 200_000,
            runs: vec![r1],
            align: TextAlign::Left,
            bullet: false,
            numbered: false,
            preserve_whitespace: false,
            wrap: false,
            line_spc_pts: Some(1820),
            last_line_spc_pts: Some(1300),
            spc_aft_pts: None,
            t_ins_emu: 0,
            vert_center: false,
            l_ins_emu: 0,
            r_ins_emu: 0,
            mar_l_emu: 0,
            list_start: 1,
        };
        let xml = textbox_sp_xml(&tb, 2, &BTreeMap::new());
        assert!(xml.contains(r#"<a:spcPts val="1820"/>"#), "{xml}");
        assert!(xml.contains(r#"<a:spcPts val="1300"/>"#), "{xml}");
        let first = xml.find(r#"<a:spcPts val="1820"/>"#).unwrap();
        let last = xml.find(r#"<a:spcPts val="1300"/>"#).unwrap();
        assert!(first < last, "last para must use face spacing, got {xml}");
    }

    #[test]
    fn stacked_paras_use_face_lnspc_and_spc_aft() {
        let mut r1 = run("line one");
        r1.text = "line one\nline two".into();
        r1.sz_hundredths_pt = 2400;
        let tb = crate::ir::TextBox {
            node_id: "title".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 1_000_000,
            cy_emu: 200_000,
            runs: vec![r1],
            align: TextAlign::Left,
            bullet: false,
            numbered: false,
            preserve_whitespace: false,
            wrap: false,
            line_spc_pts: Some(2832),
            last_line_spc_pts: Some(2400),
            spc_aft_pts: Some(432),
            t_ins_emu: 0,
            vert_center: false,
            l_ins_emu: 0,
            r_ins_emu: 0,
            mar_l_emu: 0,
            list_start: 1,
        };
        let xml = textbox_sp_xml(&tb, 2, &BTreeMap::new());
        assert_eq!(xml.matches(r#"<a:spcPts val="2400"/>"#).count(), 2, "{xml}");
        assert_eq!(xml.matches(r#"<a:spcAft><a:spcPts val="432"/>"#).count(), 1, "{xml}");
        assert!(!xml.contains(r#"<a:spcPts val="2832"/>"#), "{xml}");
        assert!(!xml.contains("spcPct"), "{xml}");
        let aft = xml.find(r#"<a:spcAft>"#).unwrap();
        let last_p = xml.rfind("<a:p>").unwrap();
        assert!(aft < last_p, "spcAft belongs on the first paragraph, got {xml}");
    }
}
