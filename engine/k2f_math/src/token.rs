use crate::error::MathError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Char(char),
    /// Control sequence without the leading backslash: `alpha`, `frac`, `,`, `{`.
    Command(String),
    /// Verbatim body of `\text{...}` / `\mathrm{...}`, where spaces are significant.
    Text(String),
    Sup,
    Sub,
    GroupOpen,
    GroupClose,
    /// Alignment tab `&` (only valid inside `matrix`/`align`/`cases`).
    AlignTab,
    /// `\\` row break (only valid inside alignment environments).
    LineBreak,
}

/// Source whitespace is dropped (TeX convention); only `\text` keeps its spaces,
/// which is why that body is captured here rather than in the parser.
pub fn tokenize(src: &str) -> Result<Vec<Token>, MathError> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '{' => {
                out.push(Token::GroupOpen);
                i += 1;
            }
            '}' => {
                out.push(Token::GroupClose);
                i += 1;
            }
            '^' => {
                out.push(Token::Sup);
                i += 1;
            }
            '_' => {
                out.push(Token::Sub);
                i += 1;
            }
            '\\' => {
                i += 1;
                let Some(&first) = chars.get(i) else {
                    return Err(MathError::Parse(
                        "trailing '\\' with no command name".into(),
                    ));
                };
                if first.is_ascii_alphabetic() {
                    let start = i;
                    while chars.get(i).is_some_and(|c| c.is_ascii_alphabetic()) {
                        i += 1;
                    }
                    let name: String = chars[start..i].iter().collect();
                    if name == "text" || name == "mathrm" {
                        let (body, next) = read_verbatim_group(&chars, i, &name)?;
                        out.push(Token::Text(body));
                        i = next;
                    } else {
                        out.push(Token::Command(name));
                    }
                } else if first == '\\' {
                    out.push(Token::LineBreak);
                    i += 1;
                } else {
                    out.push(Token::Command(first.to_string()));
                    i += 1;
                }
            }
            '$' => {
                return Err(MathError::Parse(
                    "'$' must not appear inside math source".into(),
                ))
            }
            '&' => {
                out.push(Token::AlignTab);
                i += 1;
            }
            '#' | '%' | '~' => return Err(MathError::Unsupported(format!("character {c:?}"))),
            _ => {
                out.push(Token::Char(c));
                i += 1;
            }
        }
    }
    Ok(out)
}

/// Reads `{...}` starting at `from` (after optional spaces), returning the body
/// and the index just past the closing brace.
fn read_verbatim_group(
    chars: &[char],
    from: usize,
    command: &str,
) -> Result<(String, usize), MathError> {
    let mut i = from;
    while chars.get(i).is_some_and(|c| c.is_whitespace()) {
        i += 1;
    }
    if chars.get(i) != Some(&'{') {
        return Err(MathError::Parse(format!("\\{command} requires '{{...}}'")));
    }
    i += 1;
    let start = i;
    let mut depth = 1usize;
    while let Some(&c) = chars.get(i) {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let body: String = chars[start..i].iter().collect();
                    return Ok((body, i + 1));
                }
            }
            '\\' => {
                return Err(MathError::Unsupported(format!(
                    "commands are not allowed inside \\{command}{{...}}"
                )))
            }
            _ => {}
        }
        i += 1;
    }
    Err(MathError::Parse(format!(
        "unmatched '{{' after \\{command}"
    )))
}
