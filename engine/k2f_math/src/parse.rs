use crate::atom::{big_operator, operator_name, space_command, symbol_atom, AtomClass};
use crate::error::MathError;
use crate::token::{tokenize, Token};

mod env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Delim {
    Null,
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvKind {
    Matrix,
    PMatrix,
    BMatrix,
    Align,
    Cases,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathNode {
    Row(Vec<MathNode>),
    Atom {
        ch: char,
        class: AtomClass,
    },
    /// `\sum` / `\prod` / `\int` glyph. `limits_above` is the TeX default in display.
    BigOp {
        ch: char,
        limits_above: bool,
    },
    /// Upright function name (`\sin`, `\lim`, …).
    OperatorName {
        text: String,
        limits_under: bool,
    },
    /// Explicit spacing, in math units.
    Space(i128),
    Frac {
        num: Box<MathNode>,
        den: Box<MathNode>,
    },
    Sqrt {
        body: Box<MathNode>,
    },
    Script {
        base: Box<MathNode>,
        sup: Option<Box<MathNode>>,
        sub: Option<Box<MathNode>>,
    },
    Text(String),
    LeftRight {
        left: Delim,
        inner: Box<MathNode>,
        right: Delim,
    },
    Env {
        kind: EnvKind,
        rows: Vec<Vec<MathNode>>,
    },
}

pub fn parse_tex(src: &str) -> Result<MathNode, MathError> {
    let tokens = tokenize(src)?;
    let mut p = Parser {
        tokens: &tokens,
        i: 0,
    };
    let node = p.parse_row()?;
    if p.i != tokens.len() {
        return Err(MathError::Parse("unexpected tokens after formula".into()));
    }
    Ok(node)
}

pub(crate) struct Parser<'a> {
    tokens: &'a [Token],
    i: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.i)
    }

    fn bump(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.i)?;
        self.i += 1;
        Some(t)
    }

    fn parse_row(&mut self) -> Result<MathNode, MathError> {
        self.parse_list(|t| matches!(t, Token::GroupClose))
    }

    fn parse_list(&mut self, stop: impl Fn(&Token) -> bool) -> Result<MathNode, MathError> {
        let mut items = Vec::new();
        loop {
            match self.peek() {
                None => break,
                Some(t) if stop(t) => break,
                Some(Token::Sup) | Some(Token::Sub) => {
                    return Err(MathError::Parse("script (^/_) with no base atom".into()));
                }
                _ => items.push(self.parse_item()?),
            }
        }
        Ok(match items.len() {
            0 => MathNode::Row(vec![]),
            1 => items.pop().unwrap(),
            _ => MathNode::Row(items),
        })
    }

    fn parse_item(&mut self) -> Result<MathNode, MathError> {
        let mut base = self.parse_nucleus()?;
        let mut sup = None;
        let mut sub = None;
        loop {
            match self.peek() {
                Some(Token::Sup) => {
                    self.i += 1;
                    if sup.is_some() {
                        return Err(MathError::Parse("double superscript".into()));
                    }
                    sup = Some(Box::new(self.parse_argument()?));
                }
                Some(Token::Sub) => {
                    self.i += 1;
                    if sub.is_some() {
                        return Err(MathError::Parse("double subscript".into()));
                    }
                    sub = Some(Box::new(self.parse_argument()?));
                }
                _ => break,
            }
        }
        if sup.is_some() || sub.is_some() {
            base = MathNode::Script {
                base: Box::new(base),
                sup,
                sub,
            };
        }
        Ok(base)
    }

    fn parse_nucleus(&mut self) -> Result<MathNode, MathError> {
        let Some(tok) = self.bump().cloned() else {
            return Err(MathError::Parse("unexpected end of formula".into()));
        };
        match tok {
            Token::Char(c) => {
                let (ch, class) = crate::atom::char_atom(c)?;
                Ok(MathNode::Atom { ch, class })
            }
            Token::Text(s) => Ok(MathNode::Text(s)),
            Token::GroupOpen => {
                let inner = self.parse_row()?;
                match self.bump() {
                    Some(Token::GroupClose) => Ok(inner),
                    _ => Err(MathError::Parse("unmatched '{'".into())),
                }
            }
            Token::GroupClose => Err(MathError::Parse("unexpected '}'".into())),
            Token::Sup | Token::Sub => Err(MathError::Parse("script with no base atom".into())),
            Token::AlignTab => Err(MathError::Parse("'&' outside alignment environment".into())),
            Token::LineBreak => Err(MathError::Parse("'\\\\' outside alignment environment".into())),
            Token::Command(name) => self.parse_command(&name),
        }
    }

    fn parse_command(&mut self, name: &str) -> Result<MathNode, MathError> {
        match name {
            "frac" => {
                let num = Box::new(self.parse_argument()?);
                let den = Box::new(self.parse_argument()?);
                Ok(MathNode::Frac { num, den })
            }
            "sqrt" => {
                if matches!(self.peek(), Some(Token::Char('['))) {
                    return Err(MathError::Unsupported(
                        "indexed root \\sqrt[n]{...} is not in v1".into(),
                    ));
                }
                let body = Box::new(self.parse_argument()?);
                Ok(MathNode::Sqrt { body })
            }
            "left" => self.parse_left_right(),
            "begin" => self.parse_begin(),
            "right" => Err(MathError::Parse("unexpected '\\right'".into())),
            "end" => Err(MathError::Parse("unexpected '\\end'".into())),
            "over" | "underline" | "overline" | "hat" | "vec" | "color" | "def" | "newcommand" => {
                Err(MathError::Unsupported(format!("unknown command '\\{name}'")))
            }
            _ => {
                if let Some((ch, class)) = symbol_atom(name) {
                    return Ok(MathNode::Atom { ch, class });
                }
                if let Some((ch, limits_above)) = big_operator(name) {
                    return Ok(MathNode::BigOp { ch, limits_above });
                }
                if let Some((text, limits_under)) = operator_name(name) {
                    return Ok(MathNode::OperatorName {
                        text: text.to_string(),
                        limits_under,
                    });
                }
                if let Some(mu) = space_command(name) {
                    return Ok(MathNode::Space(mu));
                }
                Err(MathError::Unsupported(format!(
                    "unknown command '\\{name}'"
                )))
            }
        }
    }

    fn parse_left_right(&mut self) -> Result<MathNode, MathError> {
        let left = self.parse_delimiter()?;
        let inner = self.parse_list(|t| matches!(t, Token::Command(s) if s == "right"))?;
        match self.bump() {
            Some(Token::Command(s)) if s == "right" => {}
            _ => return Err(MathError::Parse("missing '\\right'".into())),
        }
        let right = self.parse_delimiter()?;
        Ok(MathNode::LeftRight {
            left,
            inner: Box::new(inner),
            right,
        })
    }

    fn parse_delimiter(&mut self) -> Result<Delim, MathError> {
        let Some(tok) = self.bump().cloned() else {
            return Err(MathError::Parse("missing delimiter after \\left/\\right".into()));
        };
        match tok {
            Token::Char('.') => Ok(Delim::Null),
            Token::Char(c) if matches!(c, '(' | ')' | '[' | ']' | '|' | '/') => Ok(Delim::Char(c)),
            Token::Command(name) => match name.as_str() {
                "{" => Ok(Delim::Char('{')),
                "}" => Ok(Delim::Char('}')),
                "|" | "Vert" => Ok(Delim::Char('\u{2016}')),
                "vert" => Ok(Delim::Char('|')),
                "langle" => Ok(Delim::Char('\u{27E8}')),
                "rangle" => Ok(Delim::Char('\u{27E9}')),
                "backslash" => Ok(Delim::Char('\\')),
                other => Err(MathError::Parse(format!(
                    "invalid delimiter '\\{other}'"
                ))),
            },
            other => Err(MathError::Parse(format!("invalid delimiter {other:?}"))),
        }
    }

    /// One TeX argument: a `{...}` group, or a single unbraced nucleus (no scripts).
    fn parse_argument(&mut self) -> Result<MathNode, MathError> {
        match self.peek() {
            None => Err(MathError::Parse("missing command argument".into())),
            Some(Token::GroupOpen) => {
                self.i += 1;
                let inner = self.parse_row()?;
                match self.bump() {
                    Some(Token::GroupClose) => Ok(inner),
                    _ => Err(MathError::Parse("unmatched '{'".into())),
                }
            }
            Some(Token::Sup) | Some(Token::Sub) => {
                Err(MathError::Parse("script used as a command argument".into()))
            }
            Some(Token::GroupClose) => Err(MathError::Parse("unexpected '}'".into())),
            _ => self.parse_nucleus(),
        }
    }
}
