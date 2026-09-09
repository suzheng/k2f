use super::{EnvKind, MathNode, Parser};
use crate::error::MathError;
use crate::token::Token;

impl Parser<'_> {
    pub(super) fn parse_begin(&mut self) -> Result<MathNode, MathError> {
        let name = self.parse_env_name()?;
        let kind = env_kind(&name)?;
        let rows = self.parse_env_body(&name)?;
        Ok(MathNode::Env { kind, rows })
    }

    fn parse_env_name(&mut self) -> Result<String, MathError> {
        match self.bump() {
            Some(Token::GroupOpen) => {}
            _ => return Err(MathError::Parse("\\begin/\\end requires '{name}'".into())),
        }
        let mut name = String::new();
        loop {
            match self.bump().cloned() {
                Some(Token::GroupClose) => break,
                Some(Token::Char(c)) if c.is_ascii_alphabetic() => name.push(c),
                Some(_) => {
                    return Err(MathError::Parse("invalid environment name".into()));
                }
                None => return Err(MathError::Parse("unmatched '{' in environment name".into())),
            }
        }
        if name.is_empty() {
            return Err(MathError::Parse("empty environment name".into()));
        }
        Ok(name)
    }

    fn parse_env_body(&mut self, expected: &str) -> Result<Vec<Vec<MathNode>>, MathError> {
        let mut rows: Vec<Vec<MathNode>> = Vec::new();
        let mut row: Vec<MathNode> = Vec::new();
        loop {
            let cell = self.parse_list(is_cell_stop)?;
            row.push(cell);
            match self.peek() {
                Some(Token::AlignTab) => {
                    self.i += 1;
                }
                Some(Token::LineBreak) => {
                    self.i += 1;
                    rows.push(std::mem::take(&mut row));
                }
                Some(Token::Command(s)) if s == "end" => {
                    rows.push(std::mem::take(&mut row));
                    self.parse_end(expected)?;
                    break;
                }
                None => return Err(MathError::Parse(format!("missing \\end{{{expected}}}"))),
                Some(Token::GroupClose) => {
                    return Err(MathError::Parse("unexpected '}' in environment".into()))
                }
                _ => {
                    return Err(MathError::Parse(format!(
                        "unexpected token in \\begin{{{expected}}}"
                    )))
                }
            }
        }
        if rows
            .last()
            .is_some_and(|r| r.len() == 1 && is_empty_cell(&r[0]))
        {
            rows.pop();
        }
        if rows.is_empty() {
            rows.push(vec![MathNode::Row(vec![])]);
        }
        Ok(rows)
    }

    fn parse_end(&mut self, expected: &str) -> Result<(), MathError> {
        match self.bump() {
            Some(Token::Command(s)) if s == "end" => {}
            _ => return Err(MathError::Parse("missing \\end".into())),
        }
        let name = self.parse_env_name()?;
        if name != expected {
            return Err(MathError::Parse(format!(
                "\\begin{{{expected}}} closed by \\end{{{name}}}"
            )));
        }
        Ok(())
    }
}

fn is_cell_stop(t: &Token) -> bool {
    matches!(t, Token::AlignTab | Token::LineBreak | Token::GroupClose)
        || matches!(t, Token::Command(s) if s == "end")
}

fn is_empty_cell(n: &MathNode) -> bool {
    matches!(n, MathNode::Row(v) if v.is_empty())
}

fn env_kind(name: &str) -> Result<EnvKind, MathError> {
    match name {
        "matrix" => Ok(EnvKind::Matrix),
        "pmatrix" => Ok(EnvKind::PMatrix),
        "bmatrix" => Ok(EnvKind::BMatrix),
        "align" => Ok(EnvKind::Align),
        "cases" => Ok(EnvKind::Cases),
        other => Err(MathError::Unsupported(format!(
            "unknown environment '{other}'"
        ))),
    }
}
