use crate::coord::fmt_pt;
use crate::ir::{ScriptPos, TextAlign, TextRun};
use crate::xml::escape_xml;

pub(crate) fn write_paras(
    align: TextAlign,
    runs: &[TextRun],
    hts: &mut usize,
    no_break: bool,
    first_line_indent_pt: f64,
    semantic_newlines: bool,
) -> String {
    let paras = paragraphs(runs, semantic_newlines);
    let fallback = runs.first().cloned().unwrap_or_else(default_run);
    let paras = if semantic_newlines {
        fold_blank_paras(paras, &fallback)
    } else {
        paras
            .into_iter()
            .map(|runs| FoldedPara {
                runs,
                space_after_pt: 0.0,
            })
            .collect()
    };
    let mut body = String::new();
    for (i, para) in paras.iter().enumerate() {
        let indent = if i == 0 { first_line_indent_pt } else { 0.0 };
        body.push_str(&para_xml(
            align,
            &para.runs,
            &fallback,
            hts,
            no_break,
            indent,
            para.space_after_pt,
            semantic_newlines,
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

fn paragraphs(runs: &[TextRun], semantic_newlines: bool) -> Vec<Vec<TextRun>> {
    if !semantic_newlines {
        let para: Vec<TextRun> = runs.iter().cloned().collect();
        return if para.is_empty() {
            vec![Vec::new()]
        } else {
            vec![para]
        };
    }
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

struct FoldedPara {
    runs: Vec<TextRun>,
    space_after_pt: f64,
}

/// `\n\n` becomes an empty paragraph. Using the first run as fallback would
/// reprint its text in the gap; fold the blank into SpaceAfter instead.
fn fold_blank_paras(paras: Vec<Vec<TextRun>>, fallback: &TextRun) -> Vec<FoldedPara> {
    let gap = fallback
        .leading_pt
        .filter(|v| *v > 0.0)
        .unwrap_or(fallback.size_pt)
        .max(0.0);
    let mut out: Vec<FoldedPara> = Vec::new();
    for para in paras {
        if para.is_empty() {
            if let Some(prev) = out.last_mut() {
                prev.space_after_pt += gap;
            }
            continue;
        }
        out.push(FoldedPara {
            runs: para,
            space_after_pt: 0.0,
        });
    }
    if out.is_empty() {
        out.push(FoldedPara {
            runs: Vec::new(),
            space_after_pt: 0.0,
        });
    }
    out
}

fn para_xml(
    align: TextAlign,
    runs: &[TextRun],
    fallback: &TextRun,
    hts: &mut usize,
    no_break: bool,
    first_line_indent_pt: f64,
    space_after_pt: f64,
    semantic_newlines: bool,
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
        let mut empty = fallback.clone();
        empty.text.clear();
        inner.push_str(&char_range(&empty, hts, no_break, semantic_newlines));
    } else {
        for run in runs {
            inner.push_str(&char_range(run, hts, no_break, semantic_newlines));
        }
    }
    format!(
        r#"    <ParagraphStyleRange AppliedParagraphStyle="ParagraphStyle/$ID/[No paragraph style]" Justification="{just}" Hyphenation="false" SpaceBefore="0" SpaceAfter="{}"{lead_attr}{indent_attr}>
      <Properties>
        <AppliedComposer>$ID/HL Single</AppliedComposer>
      </Properties>
{inner}    </ParagraphStyleRange>
"#,
        fmt_pt(space_after_pt),
    )
}

fn char_range(run: &TextRun, hts: &mut usize, no_break: bool, semantic_newlines: bool) -> String {
    if !semantic_newlines && run.text.contains('\n') && !run.auto_page_number {
        let mut out = String::new();
        let parts: Vec<&str> = run.text.split('\n').collect();
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                let mut piece = run.clone();
                piece.text = part.to_string();
                out.push_str(&char_range(&piece, hts, no_break, true));
            }
            if i + 1 < parts.len() {
                out.push_str(&char_range_br(run, no_break));
            }
        }
        return out;
    }
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
    format!(
        r#"      <CharacterStyleRange {attrs}>
        <Properties>
          <AppliedFont type="string">{font}</AppliedFont>
        </Properties>
{kids}      </CharacterStyleRange>
"#
    )
}

fn char_range_br(run: &TextRun, no_break: bool) -> String {
    let size = fmt_pt(run.size_pt);
    let fill = format!("Color/k2f_{}", run.color_hex);
    let style = font_style(run);
    let mut attrs = format!(
        r#"AppliedCharacterStyle="CharacterStyle/$ID/[No character style]" PointSize="{size}" FillColor="{fill}" FontStyle="{style}""#
    );
    if no_break {
        attrs.push_str(r#" NoBreak="true""#);
    }
    let font = escape_xml(&run.font_name);
    format!(
        r#"      <CharacterStyleRange {attrs}>
        <Properties>
          <AppliedFont type="string">{font}</AppliedFont>
        </Properties>
        <Br/>
      </CharacterStyleRange>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_newlines_do_not_emit_redundant_br() {
        let mut run = default_run();
        run.text = "Line one\nLine two".into();
        run.leading_pt = Some(14.0);
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, true);
        assert_eq!(xml.matches("<ParagraphStyleRange").count(), 2);
        assert!(
            !xml.contains("<Br/>"),
            "ParagraphStyleRange already ends the paragraph, got {xml}"
        );
    }

    #[test]
    fn lock_wrap_newlines_use_br_within_one_paragraph() {
        let mut run = default_run();
        run.text = "Line one\nLine two".into();
        run.leading_pt = Some(14.0);
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, true, 0.0, false);
        assert_eq!(
            xml.matches("<ParagraphStyleRange").count(),
            1,
            "lock wrap must stay one paragraph, got {xml}"
        );
        assert!(
            xml.contains("<Br/>"),
            "lock wrap must hard-break with Br, got {xml}"
        );
    }

    #[test]
    fn blank_semantic_lines_become_space_after_not_fallback_text() {
        let mut run = default_run();
        run.text = "Hello\n\nWorld".into();
        run.leading_pt = Some(14.0);
        run.size_pt = 10.0;
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, true);
        assert_eq!(
            xml.matches("<ParagraphStyleRange").count(),
            2,
            "blank line must not emit a third paragraph, got {xml}"
        );
        assert!(
            xml.contains(r#"SpaceAfter="14.000""#),
            "\\n\\n must become SpaceAfter from leading, got {xml}"
        );
        assert_eq!(xml.matches("<Content>Hello</Content>").count(), 1);
        assert_eq!(xml.matches("<Content>World</Content>").count(), 1);
        assert!(
            !xml.contains("<Content>Hello\n\nWorld</Content>"),
            "must not dump the first run into the gap, got {xml}"
        );
    }
}
