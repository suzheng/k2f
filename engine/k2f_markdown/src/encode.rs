use crate::escape::escape_md;
use k2f_core::Modifier;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Link = 0,
    Strong = 1,
    Em = 2,
    Strike = 3,
    Code = 4,
    Math = 5,
}

#[derive(Clone)]
struct Span {
    start: usize,
    end: usize,
    kind: Kind,
    extra: String,
}

pub fn apply_inline_markdown(text: &str, modifiers: &[Modifier]) -> String {
    let mut spans: Vec<Span> = modifiers.iter().filter_map(span_of).collect();
    if spans.is_empty() {
        return escape_md(text);
    }
    spans.sort_by_key(|s| (s.start, s.kind, s.end));
    let mut closes = spans.clone();
    closes.sort_by_key(|s| (s.end, std::cmp::Reverse(s.kind), std::cmp::Reverse(s.start)));

    let mut out = String::new();
    let mut pos = 0;
    let mut oi = 0;
    let mut ci = 0;
    let mut in_code = 0u32;
    let mut in_math = 0u32;
    while pos <= text.len() {
        while ci < closes.len() && closes[ci].end == pos {
            if closes[ci].kind == Kind::Code {
                in_code = in_code.saturating_sub(1);
            }
            if closes[ci].kind == Kind::Math {
                in_math = in_math.saturating_sub(1);
            }
            out.push_str(&close_tag(&closes[ci]));
            ci += 1;
        }
        while oi < spans.len() && spans[oi].start == pos {
            if spans[oi].kind == Kind::Code {
                in_code += 1;
            }
            if spans[oi].kind == Kind::Math {
                in_math += 1;
            }
            out.push_str(&open_tag(&spans[oi]));
            oi += 1;
        }
        if pos == text.len() {
            break;
        }
        let mut next = text.len();
        if oi < spans.len() {
            next = next.min(spans[oi].start);
        }
        if ci < closes.len() {
            next = next.min(closes[ci].end);
        }
        let slice = &text[pos..next];
        if in_math > 0 {
            // Object-replacement placeholder is not part of Markdown.
        } else if in_code > 0 {
            out.push_str(slice);
        } else {
            out.push_str(&escape_md(slice));
        }
        pos = next;
    }
    out
}

fn span_of(m: &Modifier) -> Option<Span> {
    if m.range[0] >= m.range[1] {
        return None;
    }
    let (kind, extra) = match (m.mod_type.as_str(), m.intent.as_str()) {
        ("emphasis", "strong") => (Kind::Strong, String::new()),
        ("emphasis", "emphasis") => (Kind::Em, String::new()),
        ("emphasis", "code") => (Kind::Code, String::new()),
        ("strikethrough", _) => (Kind::Strike, String::new()),
        ("link", url) => (Kind::Link, url.to_string()),
        ("math", tex) => (Kind::Math, tex.to_string()),
        _ => return None,
    };
    Some(Span {
        start: m.range[0],
        end: m.range[1],
        kind,
        extra,
    })
}

fn open_tag(s: &Span) -> String {
    match s.kind {
        Kind::Link => "[".into(),
        Kind::Strong => "**".into(),
        Kind::Em => "*".into(),
        Kind::Strike => "~~".into(),
        Kind::Code => "`".into(),
        Kind::Math => format!("${}$", s.extra),
    }
}

fn close_tag(s: &Span) -> String {
    match s.kind {
        Kind::Link => format!("]({})", s.extra),
        Kind::Strong => "**".into(),
        Kind::Em => "*".into(),
        Kind::Strike => "~~".into(),
        Kind::Code => "`".into(),
        Kind::Math => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_literal_stars() {
        let md = apply_inline_markdown("a * b * c", &[]);
        assert_eq!(md, r"a \* b \* c");
    }

    #[test]
    fn wraps_strong_and_link() {
        let md = apply_inline_markdown(
            "ab",
            &[
                Modifier {
                    range: [0, 2],
                    mod_type: "emphasis".into(),
                    intent: "strong".into(),
                },
                Modifier {
                    range: [0, 2],
                    mod_type: "link".into(),
                    intent: "https://e.com".into(),
                },
            ],
        );
        assert_eq!(md, "[**ab**](https://e.com)");
    }

    #[test]
    fn emits_inline_math_dollars_without_placeholder() {
        let text = "a\u{FFFC}b";
        let start = text.find('\u{FFFC}').unwrap();
        let md = apply_inline_markdown(
            text,
            &[Modifier {
                range: [start, start + '\u{FFFC}'.len_utf8()],
                mod_type: "math".into(),
                intent: "a/b".into(),
            }],
        );
        assert_eq!(md, "a$a/b$b");
    }
}
