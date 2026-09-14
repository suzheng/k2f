use crate::coord::{fmt_pt, DOM, NS};
use crate::ir::{ScriptPos, TextBox, TextRun};
use crate::xml::escape_xml;

const XML_DECL: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;

pub fn story_xml(tb: &TextBox, story_self: &str) -> String {
    let paras = paragraphs(&tb.runs);
    let mut body = String::new();
    let mut hts = 0usize;
    let n = paras.len();
    for (i, para) in paras.iter().enumerate() {
        body.push_str(&para_xml(tb, para, &mut hts, i + 1 == n));
    }
    format!(
        r#"{XML_DECL}
<idPkg:Story xmlns:idPkg="{NS}" DOMVersion="{DOM}">
  <Story Self="{story_self}" AppliedTOCStyle="n" TrackChanges="false" StoryTitle="$ID/" AppliedNamedGrid="n">
    <StoryPreference OpticalMarginAlignment="false" OpticalMarginSize="12" FrameType="TextFrameType" StoryOrientation="Horizontal" StoryDirection="LeftToRightDirection"/>
{body}  </Story>
</idPkg:Story>
"#
    )
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

fn para_xml(tb: &TextBox, runs: &[TextRun], hts: &mut usize, _last: bool) -> String {
    let just = tb.align.justification();
    let leading = runs
        .iter()
        .find_map(|r| r.leading_pt)
        .or_else(|| tb.runs.first().and_then(|r| r.leading_pt));
    let lead_attr = leading
        .map(|v| format!(r#" Leading="{}""#, fmt_pt(v)))
        .unwrap_or_default();
    let mut inner = String::new();
    if runs.is_empty() {
        inner.push_str(&char_range(&empty_run(tb), hts, true));
    } else {
        let last = runs.len() - 1;
        for (i, run) in runs.iter().enumerate() {
            inner.push_str(&char_range(run, hts, i == last));
        }
    }
    format!(
        r#"    <ParagraphStyleRange AppliedParagraphStyle="ParagraphStyle/$ID/[No paragraph style]" Justification="{just}" Hyphenation="false" AutoLeading="0"{lead_attr}>
      <Properties>
        <AppliedComposer>$ID/HL Single</AppliedComposer>
      </Properties>
{inner}    </ParagraphStyleRange>
"#
    )
}

fn empty_run(tb: &TextBox) -> TextRun {
    tb.runs.first().cloned().unwrap_or(TextRun {
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
    })
}

fn char_range(run: &TextRun, hts: &mut usize, with_br: bool) -> String {
    let size = fmt_pt(run.size_pt);
    let fill = format!("Color/k2f_{}", run.color_hex);
    let style = font_style(run);
    let mut attrs = format!(
        r#"AppliedCharacterStyle="CharacterStyle/$ID/[No character style]" PointSize="{size}" FillColor="{fill}" FontStyle="{style}""#
    );
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
