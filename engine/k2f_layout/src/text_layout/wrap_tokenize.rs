use super::types::TextRun;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FragKind {
    Text,
    Whitespace,
    Newline,
    Atom,
}

#[derive(Debug, Clone)]
pub(crate) struct Fragment {
    pub(crate) kind: FragKind,
    pub(crate) run: Option<TextRun>, // present for Text/Whitespace; absent for Newline
}

pub(crate) fn split_run_for_wrapping(run: &TextRun) -> Vec<Fragment> {
    if run.math_tex.is_some() {
        return vec![Fragment {
            kind: FragKind::Atom,
            run: Some(run.clone()),
        }];
    }

    let mut out: Vec<Fragment> = Vec::new();
    let s = run.text.as_str();

    let mut seg_start: usize = 0;
    let mut i: usize = 0;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        let ch_len = ch.len_utf8();

        // Hard newline boundaries.
        if ch == '\n' || ch == '\r' {
            if seg_start < i {
                out.push(Fragment {
                    kind: FragKind::Text,
                    run: Some(TextRun {
                        start: run.start + seg_start,
                        end: run.start + i,
                        style: run.style.clone(),
                        text: s[seg_start..i].to_string(),
                        math_tex: None,
                    }),
                });
            }
            out.push(Fragment {
                kind: FragKind::Newline,
                run: None,
            });
            i += ch_len;
            seg_start = i;
            continue;
        }

        // Group whitespace (excluding newlines).
        if ch.is_whitespace() {
            if seg_start < i {
                out.push(Fragment {
                    kind: FragKind::Text,
                    run: Some(TextRun {
                        start: run.start + seg_start,
                        end: run.start + i,
                        style: run.style.clone(),
                        text: s[seg_start..i].to_string(),
                        math_tex: None,
                    }),
                });
            }
            let ws_start = i;
            i += ch_len;
            while i < s.len() {
                let ch2 = s[i..].chars().next().unwrap();
                if ch2 == '\n' || ch2 == '\r' || !ch2.is_whitespace() {
                    break;
                }
                i += ch2.len_utf8();
            }
            let ws_end = i;
            out.push(Fragment {
                kind: FragKind::Whitespace,
                run: Some(TextRun {
                    start: run.start + ws_start,
                    end: run.start + ws_end,
                    style: run.style.clone(),
                    text: s[ws_start..ws_end].to_string(),
                    math_tex: None,
                }),
            });
            seg_start = i;
            continue;
        }

        // Optional break after hyphen-minus or soft hyphen (UAX #14 BA).
        if ch == '-' || ch == '\u{00AD}' {
            i += ch_len;
            out.push(Fragment {
                kind: FragKind::Text,
                run: Some(TextRun {
                    start: run.start + seg_start,
                    end: run.start + i,
                    style: run.style.clone(),
                    text: s[seg_start..i].to_string(),
                    math_tex: None,
                }),
            });
            seg_start = i;
            continue;
        }

        i += ch_len;
    }

    if seg_start < s.len() {
        out.push(Fragment {
            kind: FragKind::Text,
            run: Some(TextRun {
                start: run.start + seg_start,
                end: run.start + s.len(),
                style: run.style.clone(),
                text: s[seg_start..].to_string(),
                math_tex: None,
            }),
        });
    }

    out
}
