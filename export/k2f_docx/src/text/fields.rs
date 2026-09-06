use crate::ir::{DocField, TextRun};

const TOKENS: [(&str, DocField); 2] = [
    ("{{page_current}}", DocField::Page),
    ("{{page_total}}", DocField::NumPages),
];

pub(crate) fn has_page_tokens(text: &str) -> bool {
    TOKENS.iter().any(|(tok, _)| text.contains(tok))
}

/// Rewrite `{{page_current}}` / `{{page_total}}` into Word fields.
/// Tokens may be split across adjacent paint runs; search the concatenated text.
pub(crate) fn expand_fields(runs: Vec<TextRun>) -> Vec<TextRun> {
    if runs.is_empty() {
        return runs;
    }
    let full: String = runs.iter().map(|r| r.text.as_str()).collect();
    if TOKENS.iter().all(|(tok, _)| !full.contains(tok)) {
        return runs;
    }
    let mut out = Vec::new();
    for (start, end, field) in field_spans(&full) {
        if let Some(field) = field {
            let mut run = style_at(&runs, start).clone();
            run.text = "1".into();
            run.field = Some(field);
            out.push(run);
        } else {
            out.extend(slice_runs(&runs, start, end));
        }
    }
    out
}

fn field_spans(full: &str) -> Vec<(usize, usize, Option<DocField>)> {
    let mut pos = 0;
    let mut out = Vec::new();
    while pos < full.len() {
        let next = TOKENS
            .iter()
            .filter_map(|(tok, field)| full[pos..].find(tok).map(|i| (pos + i, *tok, *field)))
            .min_by_key(|(i, _, _)| *i);
        let Some((i, tok, field)) = next else {
            out.push((pos, full.len(), None));
            break;
        };
        if i > pos {
            out.push((pos, i, None));
        }
        out.push((i, i + tok.len(), Some(field)));
        pos = i + tok.len();
    }
    out
}

fn style_at(runs: &[TextRun], byte_off: usize) -> &TextRun {
    let mut pos = 0;
    for run in runs {
        let end = pos + run.text.len();
        if byte_off < end || (byte_off == pos && run.text.is_empty()) {
            return run;
        }
        pos = end;
    }
    runs.last().expect("non-empty runs")
}

fn slice_runs(runs: &[TextRun], start: usize, end: usize) -> Vec<TextRun> {
    let mut pos = 0;
    let mut out = Vec::new();
    for run in runs {
        let a = pos;
        let b = pos + run.text.len();
        pos = b;
        let lo = start.max(a);
        let hi = end.min(b);
        if lo >= hi {
            continue;
        }
        let mut piece = run.clone();
        piece.text = run.text[lo - a..hi - a].to_string();
        piece.field = None;
        if !piece.text.is_empty() {
            out.push(piece);
        }
    }
    out
}
