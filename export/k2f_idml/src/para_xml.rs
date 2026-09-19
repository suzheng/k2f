use crate::coord::fmt_pt;
use crate::ir::{ScriptPos, TextAlign, TextRun};
use crate::xml::escape_xml;

pub(crate) fn write_paras(
    align: TextAlign,
    runs: &[TextRun],
    hts: &mut usize,
    no_break: bool,
    first_line_indent_pt: f64,
    left_indent_pt: f64,
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
        let end_with_br = semantic_newlines && i + 1 < paras.len();
        body.push_str(&para_xml(
            align,
            &para.runs,
            &fallback,
            hts,
            no_break,
            indent,
            left_indent_pt,
            para.space_after_pt,
            semantic_newlines,
            end_with_br,
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
        face_style: "Regular".into(),
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
    left_indent_pt: f64,
    space_after_pt: f64,
    semantic_newlines: bool,
    end_with_br: bool,
) -> String {
    let just = align.justification();
    let leading = runs
        .iter()
        .find_map(|r| r.leading_pt)
        .or(fallback.leading_pt);
    let size_pt = runs
        .iter()
        .find(|r| r.size_pt > 0.0)
        .map(|r| r.size_pt)
        .filter(|s| *s > 0.0)
        .unwrap_or(fallback.size_pt);
    let lead_attr = leading
        .map(|v| format!(r#" Leading="{}""#, fmt_pt(v)))
        .unwrap_or_default();
    // HL Single ignores attribute Leading and uses AutoLeading % of point
    // size (default 120). AutoLeading=0 collapses lines. Match lock leading.
    let auto_lead_attr = auto_leading_attr(leading, size_pt);
    let lead_prop = leading_unit_xml(leading, "        ");
    let indent_attr = if first_line_indent_pt.abs() > 0.0005 {
        format!(r#" FirstLineIndent="{}""#, fmt_pt(first_line_indent_pt))
    } else {
        String::new()
    };
    let left_attr = if left_indent_pt.abs() > 0.0005 {
        format!(r#" LeftIndent="{}""#, fmt_pt(left_indent_pt))
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
    if end_with_br {
        let br_run = runs.last().unwrap_or(fallback);
        inner.push_str(&char_range_br(br_run, no_break));
    }
    format!(
        r#"    <ParagraphStyleRange AppliedParagraphStyle="ParagraphStyle/$ID/[No paragraph style]" Justification="{just}" Hyphenation="false" SpaceBefore="0" SpaceAfter="{}"{lead_attr}{auto_lead_attr}{left_attr}{indent_attr}>
      <Properties>
        <AppliedComposer>$ID/HL Single</AppliedComposer>
{lead_prop}      </Properties>
{inner}    </ParagraphStyleRange>
"#,
        fmt_pt(space_after_pt),
    )
}

fn auto_leading_attr(leading: Option<f64>, size_pt: f64) -> String {
    match leading {
        Some(v) if v > 0.0 && size_pt > 0.0 => {
            format!(r#" AutoLeading="{}""#, fmt_pt(v / size_pt * 100.0))
        }
        _ => String::new(),
    }
}

fn leading_unit_xml(leading: Option<f64>, indent: &str) -> String {
    match leading {
        Some(v) if v > 0.0 => format!("{indent}<Leading type=\"unit\">{}</Leading>\n", fmt_pt(v)),
        _ => String::new(),
    }
}

fn font_properties_xml(font: &str, leading: Option<f64>) -> String {
    let lead = leading_unit_xml(leading, "          ");
    format!(
        "        <Properties>\n          <AppliedFont type=\"string\">{font}</AppliedFont>\n{lead}        </Properties>\n"
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
    let style = escape_xml(&run.idml_font_style());
    let mut attrs = format!(
        r#"AppliedCharacterStyle="CharacterStyle/$ID/[No character style]" PointSize="{size}" FillColor="{fill}" FontStyle="{style}""#
    );
    attrs.push_str(&synthetic_face_attrs(run));
    if let Some(lead) = run.leading_pt {
        attrs.push_str(&format!(r#" Leading="{}""#, fmt_pt(lead)));
    }
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
    let props = font_properties_xml(&font, run.leading_pt);
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
{props}{kids}      </CharacterStyleRange>
"#
    )
}

fn synthetic_face_attrs(run: &TextRun) -> String {
    let mut s = String::new();
    if let Some(deg) = run.synthetic_skew_deg() {
        s.push_str(&format!(r#" Skew="{}""#, fmt_pt(deg)));
    }
    if let Some(w) = run.synthetic_bold_stroke_pt() {
        // Only StrokeWeight + fill color. StrokeType="$ID/Solid" drops the
        // stroke; StrokeAlignment is a page-item attr (ignored / outlines).
        s.push_str(&format!(
            r#" StrokeWeight="{}" StrokeColor="Color/k2f_{}""#,
            fmt_pt(w),
            run.color_hex,
        ));
    }
    s
}

fn char_range_br(run: &TextRun, no_break: bool) -> String {
    let size = fmt_pt(run.size_pt);
    let fill = format!("Color/k2f_{}", run.color_hex);
    let style = escape_xml(&run.idml_font_style());
    let mut attrs = format!(
        r#"AppliedCharacterStyle="CharacterStyle/$ID/[No character style]" PointSize="{size}" FillColor="{fill}" FontStyle="{style}""#
    );
    attrs.push_str(&synthetic_face_attrs(run));
    if let Some(lead) = run.leading_pt {
        attrs.push_str(&format!(r#" Leading="{}""#, fmt_pt(lead)));
    }
    if no_break {
        attrs.push_str(r#" NoBreak="true""#);
    }
    let font = escape_xml(&run.font_name);
    let props = font_properties_xml(&font, run.leading_pt);
    format!(
        r#"      <CharacterStyleRange {attrs}>
{props}        <Br/>
      </CharacterStyleRange>
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_newlines_emit_br_between_paragraphs() {
        let mut run = default_run();
        run.text = "Line one\nLine two".into();
        run.leading_pt = Some(14.0);
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, 0.0, true);
        assert_eq!(xml.matches("<ParagraphStyleRange").count(), 2);
        assert_eq!(
            xml.matches("<Br/>").count(),
            1,
            "IDML needs a Br between semantic paragraphs, got {xml}"
        );
        assert!(
            xml.contains(r#"Leading="14.000""#),
            "character and paragraph Leading must be numeric, got {xml}"
        );
        assert!(
            xml.contains(r#"<Leading type="unit">14.000</Leading>"#),
            "IDML Leading must also be a unit property, got {xml}"
        );
        // default_run size 12pt → 14/12*100
        assert!(
            xml.contains(r#"AutoLeading="116.667""#),
            "HL Single uses AutoLeading % of point size, got {xml}"
        );
    }

    #[test]
    fn autoleading_percent_matches_lock_leading_over_size() {
        let mut run = default_run();
        run.text = "Body copy".into();
        run.size_pt = 10.5;
        run.leading_pt = Some(16.8);
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, 0.0, false);
        assert!(
            xml.contains(r#"AutoLeading="160.000""#),
            "16.8pt / 10.5pt = 160% auto leading, got {xml}"
        );
    }

    #[test]
    fn lock_wrap_newlines_use_br_within_one_paragraph() {
        let mut run = default_run();
        run.text = "Line one\nLine two".into();
        run.leading_pt = Some(14.0);
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, true, 0.0, 0.0, false);
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
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, 0.0, true);
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
        assert_eq!(
            xml.matches("<Br/>").count(),
            1,
            "paragraph mark between Hello and World, got {xml}"
        );
        assert!(
            !xml.contains("<Content>Hello\n\nWorld</Content>"),
            "must not dump the first run into the gap, got {xml}"
        );
    }

    #[test]
    fn hanging_list_emits_left_indent_and_negative_first_line() {
        let mut run = default_run();
        run.text = "•\u{00A0}Item".into();
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, -18.0, 18.0, false);
        assert!(
            xml.contains(r#"LeftIndent="18.000""#),
            "list wrap must hang at body start, got {xml}"
        );
        assert!(
            xml.contains(r#"FirstLineIndent="-18.000""#),
            "literal marker sits in the hanging gutter, got {xml}"
        );
    }

    #[test]
    fn book_subfamily_is_the_idml_font_style() {
        let mut run = default_run();
        run.text = "Body".into();
        run.font_name = "DejaVu Serif".into();
        run.face_style = "Book".into();
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, 0.0, false);
        assert!(
            xml.contains(r#"FontStyle="Book""#),
            "Book faces must not be advertised as Regular, got {xml}"
        );
        assert!(
            !xml.contains(r#"FontStyle="Regular""#),
            "got {xml}"
        );
    }

    #[test]
    fn regular_italic_uses_skew_not_missing_face() {
        let mut run = default_run();
        run.text = "Thank You".into();
        run.italic = true;
        run.bold = true;
        run.size_pt = 27.0;
        run.face_style = "Regular".into();
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Center, &[run], &mut hts, true, 0.0, 0.0, false);
        assert!(
            xml.contains(r#"FontStyle="Regular""#),
            "Regular-only packages must not advertise Bold Italic, got {xml}"
        );
        assert!(
            !xml.contains(r#"FontStyle="Bold Italic""#),
            "got {xml}"
        );
        assert!(
            xml.contains(r#"Skew="12.000""#),
            "synthetic italic must shear like K2F paint, got {xml}"
        );
        assert!(
            xml.contains(r#"StrokeWeight="0.600""#),
            "synthetic bold must stroke Regular at 1/45 em, got {xml}"
        );
        assert!(
            xml.contains(r#"StrokeColor="Color/k2f_000000""#),
            "faux-bold stroke must use the fill color, not default Black, got {xml}"
        );
    }

    #[test]
    fn real_italic_face_is_not_double_skewed() {
        let mut run = default_run();
        run.text = "Cite".into();
        run.italic = true;
        run.face_style = "Italic".into();
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, 0.0, false);
        assert!(xml.contains(r#"FontStyle="Italic""#), "got {xml}");
        assert!(!xml.contains("Skew="), "real italic must not faux-skew, got {xml}");
    }

    #[test]
    fn real_bold_face_is_not_stroked() {
        let mut run = default_run();
        run.text = "Title".into();
        run.bold = true;
        run.face_style = "Bold".into();
        run.size_pt = 24.0;
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, 0.0, false);
        assert!(xml.contains(r#"FontStyle="Bold""#), "got {xml}");
        assert!(
            !xml.contains("StrokeWeight="),
            "real bold must not faux-stroke, got {xml}"
        );
    }

    #[test]
    fn regular_bold_uses_fill_colored_stroke() {
        let mut run = default_run();
        run.text = "Title".into();
        run.bold = true;
        run.face_style = "Regular".into();
        run.size_pt = 26.0;
        run.color_hex = "1A1A2E".into();
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, 0.0, false);
        assert!(xml.contains(r#"FontStyle="Regular""#), "got {xml}");
        assert!(
            !xml.contains(r#"FontStyle="Bold""#),
            "must not advertise a missing Bold cut, got {xml}"
        );
        assert!(
            xml.contains(r#"StrokeWeight="0.578""#),
            "26pt / 45 = 0.578pt stroke, got {xml}"
        );
        assert!(
            xml.contains(r#"StrokeColor="Color/k2f_1A1A2E""#),
            "stroke must match fill, got {xml}"
        );
        assert!(
            !xml.contains("StrokeAlignment="),
            "extra alignment dropped or outlined the stroke, got {xml}"
        );
        assert!(
            !xml.contains("StrokeType="),
            "StrokeType on characters drops or outlines the stroke, got {xml}"
        );
    }

    #[test]
    fn small_regular_bold_uses_em_stroke() {
        let mut run = default_run();
        run.text = "Date".into();
        run.bold = true;
        run.face_style = "Regular".into();
        run.size_pt = 9.0;
        let mut hts = 0usize;
        let xml = write_paras(TextAlign::Left, &[run], &mut hts, false, 0.0, 0.0, false);
        assert!(
            xml.contains(r#"StrokeWeight="0.200""#),
            "9pt / 45 = 0.2pt, got {xml}"
        );
    }
}
