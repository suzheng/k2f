use crate::coord::fmt_pt;
use crate::ir::{ScriptPos, TextAlign, TextRun};
use crate::xml::escape_xml;

pub(crate) fn write_paras(
    align: TextAlign,
    runs: &[TextRun],
    hts: &mut usize,
    no_break: bool,
    first_line_indent_pt: f64,
) -> String {
    let paras = paragraphs(runs);
    let n = paras.len();
    let fallback = runs.first().cloned().unwrap_or_else(default_run);
    let mut body = String::new();
    for (i, para) in paras.iter().enumerate() {
        let indent = if i == 0 { first_line_indent_pt } else { 0.0 };
        body.push_str(&para_xml(
            align,
            para,
            &fallback,
            hts,
            i + 1 == n,
            no_break,
            indent,
        ));
    }
    body
}

pub(crate) fn default_run() -> TextRun {
    TextRun {
        text: String::new(),
        font_name: "Roboto".into(),
        size_pt: 12.0,
        bold: false,
        italic: false,
        underline: false,
        strike: false,
        color_hex: "000000".into(),
        hyperlink: None,
        script: ScriptPos::Baseline,
        leading_pt: None,
        auto_page_number: false,
        tracking: 0,
    }
}

fn paragraphs(runs: &[TextRun]) -> Vec<Vec<TextRun>> {
    let mut paras: Vec<Vec<TextRun>> = Vec::new();
    let mut cur: Vec<TextRun> = Vec::new();
    for run in runs {
        if run.auto_page_number {
            cur.push(run.clone());
            continue;
        }
        let mut rest = run.text.as_str();
        while let Some(i) = rest.find('\n') {
            let head = &rest[..i];
            if !head.is_empty() {
                let mut piece = run.clone();
                piece.text = head.to_string();
                cur.push(piece);
            }
            paras.push(std::mem::take(&mut cur));
            rest = &rest[i + 1..];
        }
        if !rest.is_empty() {
            let mut piece = run.clone();
            piece.text = rest.to_string();
            cur.push(piece);
        }
    }
    if !cur.is_empty() {
        paras.push(cur);
    }
    if paras.is_empty() {
        paras.push(Vec::new());
    }
    paras
}

fn para_xml(
    align: TextAlign,
    runs: &[TextRun],
    fallback: &TextRun,
    hts: &mut usize,
    last_para: bool,
    no_break: bool,
    first_line_indent_pt: f64,
) -> String {
    let just = align.justification();
    let leading = runs
        .iter()
        .find_map(|r| r.leading_pt)
        .or(fallback.leading_pt);
    let lead_attr = leading
        .map(|v| format!(r#" Leading="{}""#, fmt_pt(v)))
        .unwrap_or_default();
    let indent_attr = if first_line_indent_pt > 0.0 {
        format!(r#" FirstLineIndent="{}""#, fmt_pt(first_line_indent_pt))
    } else {
        String::new()
    };
    let mut inner = String::new();
    if runs.is_empty() {
        inner.push_str(&char_range(fallback, hts, !last_para, no_break));
    } else {
        let last = runs.len() - 1;
        for (i, run) in runs.iter().enumerate() {
            inner.push_str(&char_range(run, hts, i == last && !last_para, no_break));
        }
    }
    format!(
        r#"    <ParagraphStyleRange AppliedParagraphStyle="ParagraphStyle/$ID/[No paragraph style]" Justification="{just}" Hyphenation="false" SpaceBefore="0" SpaceAfter="0"{lead_attr}{indent_attr}>
      <Properties>
        <AppliedComposer>$ID/HL Single</AppliedComposer>
      </Properties>
{inner}    </ParagraphStyleRange>
"#
    )
}

fn char_range(run: &TextRun, hts: &mut usize, with_br: bool, no_break: bool) -> String {
    let size = fmt_pt(run.size_pt);
    let fill = format!("Color/k2f_{}", run.color_hex);
    let style = font_style(run);
    let mut attrs = format!(
        r#"AppliedCharacterStyle="CharacterStyle/$ID/[No character style]" PointSize="{size}" FillColor="{fill}" FontStyle="{style}""#
    );
    if no_break {
        attrs.push_str(r#" NoBreak="true""#);
    }
    if run.underline {
        attrs.push_str(r#" Underline="true""#);
    }
    if run.strike {
        attrs.push_str(r#" StrikeThru="true""#);
    }
    match run.script {
        ScriptPos::Sub => attrs.push_str(r#" Position="Subscript""#),
        ScriptPos::Super => attrs.push_str(r#" Position="Superscript""#),
        ScriptPos::Baseline => {}
    }
    if run.tracking != 0 {
        attrs.push_str(&format!(r#" Tracking="{}""#, run.tracking));
    }
    let font = escape_xml(&run.font_name);
    let mut kids = String::new();
    if run.auto_page_number {
        kids.push_str("        <AutoPageNumber/>\n");
    } else if let Some(url) = &run.hyperlink {
        let id = *hts;
        *hts += 1;
        kids.push_str(&format!(
            "        <HyperlinkTextSource Self=\"kHts{id}\" Name=\"{name}\" Hidden=\"false\">\n          <Content>{c}</Content>\n        </HyperlinkTextSource>\n",
            name = escape_xml(url),
            c = escape_xml(&run.text),
        ));
    } else {
        kids.push_str(&format!(
            "        <Content>{}</Content>\n",
            escape_xml(&run.text)
        ));
    }
    if with_br {
        kids.push_str("        <Br/>\n");
    }
    format!(
        r#"      <CharacterStyleRange {attrs}>
        <Properties>
          <AppliedFont type="string">{font}</AppliedFont>
        </Properties>
{kids}      </CharacterStyleRange>
"#
    )
}

fn font_style(run: &TextRun) -> &'static str {
    match (run.bold, run.italic) {
        (true, true) => "Bold Italic",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (false, false) => "Regular",
    }
}
