//! The v1 whitelist: TeX command → Unicode code point, plus TeX atom classes
//! and the inter-atom spacing table that classes drive.

use crate::error::MathError;
use crate::style::MathStyle;

/// TeX atom class. `Space` is an explicit spacing command, which never picks up
/// additional inter-atom space from its neighbours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomClass {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Inner,
    Space,
}

/// Literal source characters allowed outside `\text`.
pub(crate) fn char_atom(c: char) -> Result<(char, AtomClass), MathError> {
    let class = match c {
        '0'..='9' | 'a'..='z' | 'A'..='Z' => AtomClass::Ord,
        '.' | '!' | '?' | '|' | '/' | '\'' => AtomClass::Ord,
        '+' | '-' | '*' => AtomClass::Bin,
        '=' | '<' | '>' | ':' => AtomClass::Rel,
        ',' | ';' => AtomClass::Punct,
        '(' | '[' => AtomClass::Open,
        ')' | ']' => AtomClass::Close,
        _ => {
            return Err(MathError::Unsupported(format!(
                "character {c:?} (U+{:04X})",
                c as u32
            )))
        }
    };
    Ok((c, class))
}

/// Symbol commands. `None` means "not a symbol"; the parser then tries the
/// structural commands before reporting an unknown command.
pub(crate) fn symbol_atom(name: &str) -> Option<(char, AtomClass)> {
    use AtomClass::*;
    let entry = match name {
        // Greek, lowercase (TeX names).
        "alpha" => ('\u{03B1}', Ord),
        "beta" => ('\u{03B2}', Ord),
        "gamma" => ('\u{03B3}', Ord),
        "delta" => ('\u{03B4}', Ord),
        "epsilon" => ('\u{03F5}', Ord),
        "varepsilon" => ('\u{03B5}', Ord),
        "zeta" => ('\u{03B6}', Ord),
        "eta" => ('\u{03B7}', Ord),
        "theta" => ('\u{03B8}', Ord),
        "vartheta" => ('\u{03D1}', Ord),
        "iota" => ('\u{03B9}', Ord),
        "kappa" => ('\u{03BA}', Ord),
        "lambda" => ('\u{03BB}', Ord),
        "mu" => ('\u{03BC}', Ord),
        "nu" => ('\u{03BD}', Ord),
        "xi" => ('\u{03BE}', Ord),
        "pi" => ('\u{03C0}', Ord),
        "varpi" => ('\u{03D6}', Ord),
        "rho" => ('\u{03C1}', Ord),
        "varrho" => ('\u{03F1}', Ord),
        "sigma" => ('\u{03C3}', Ord),
        "varsigma" => ('\u{03C2}', Ord),
        "tau" => ('\u{03C4}', Ord),
        "upsilon" => ('\u{03C5}', Ord),
        "phi" => ('\u{03D5}', Ord),
        "varphi" => ('\u{03C6}', Ord),
        "chi" => ('\u{03C7}', Ord),
        "psi" => ('\u{03C8}', Ord),
        "omega" => ('\u{03C9}', Ord),
        // Greek, uppercase (TeX-defined capitals only).
        "Gamma" => ('\u{0393}', Ord),
        "Delta" => ('\u{0394}', Ord),
        "Theta" => ('\u{0398}', Ord),
        "Lambda" => ('\u{039B}', Ord),
        "Xi" => ('\u{039E}', Ord),
        "Pi" => ('\u{03A0}', Ord),
        "Sigma" => ('\u{03A3}', Ord),
        "Upsilon" => ('\u{03A5}', Ord),
        "Phi" => ('\u{03A6}', Ord),
        "Psi" => ('\u{03A8}', Ord),
        "Omega" => ('\u{03A9}', Ord),
        // Ordinary symbols.
        "infty" => ('\u{221E}', Ord),
        "partial" => ('\u{2202}', Ord),
        "nabla" => ('\u{2207}', Ord),
        "hbar" => ('\u{210F}', Ord),
        "ell" => ('\u{2113}', Ord),
        "emptyset" => ('\u{2205}', Ord),
        "forall" => ('\u{2200}', Ord),
        "exists" => ('\u{2203}', Ord),
        "prime" => ('\u{2032}', Ord),
        "ldots" => ('\u{2026}', Inner),
        "cdots" => ('\u{22EF}', Inner),
        // Delimiters also usable without `\left`/`\right`.
        "langle" => ('\u{27E8}', Open),
        "rangle" => ('\u{27E9}', Close),
        "lfloor" => ('\u{230A}', Open),
        "rfloor" => ('\u{230B}', Close),
        // Binary operators.
        "pm" => ('\u{00B1}', Bin),
        "mp" => ('\u{2213}', Bin),
        "times" => ('\u{00D7}', Bin),
        "cdot" => ('\u{22C5}', Bin),
        "div" => ('\u{00F7}', Bin),
        "cup" => ('\u{222A}', Bin),
        "cap" => ('\u{2229}', Bin),
        "otimes" => ('\u{2297}', Bin),
        "oplus" => ('\u{2295}', Bin),
        "wedge" => ('\u{2227}', Bin),
        "vee" => ('\u{2228}', Bin),
        // Relations.
        "mid" => ('\u{2223}', Rel),
        "parallel" => ('\u{2225}', Rel),
        "perp" => ('\u{22A5}', Rel),
        "sim" => ('\u{223C}', Rel),
        "propto" => ('\u{221D}', Rel),
        "leq" | "le" => ('\u{2264}', Rel),
        "geq" | "ge" => ('\u{2265}', Rel),
        "neq" | "ne" => ('\u{2260}', Rel),
        "approx" => ('\u{2248}', Rel),
        "equiv" => ('\u{2261}', Rel),
        "in" => ('\u{2208}', Rel),
        "notin" => ('\u{2209}', Rel),
        "subset" => ('\u{2282}', Rel),
        "supset" => ('\u{2283}', Rel),
        "subseteq" => ('\u{2286}', Rel),
        "supseteq" => ('\u{2287}', Rel),
        "mapsto" => ('\u{21A6}', Rel),
        "rightarrow" | "to" => ('\u{2192}', Rel),
        "leftarrow" => ('\u{2190}', Rel),
        "Rightarrow" => ('\u{21D2}', Rel),
        "Leftrightarrow" => ('\u{21D4}', Rel),
        // Stretchy `\left`/`\right` assemble from Unicode pieces in k2f_math.
        "{" => ('{', Open),
        "}" => ('}', Close),
        _ => return None,
    };
    Some(entry)
}

/// Big operators: glyph plus whether limits go above/below in display style.
pub(crate) fn big_operator(name: &str) -> Option<(char, bool)> {
    match name {
        "sum" => Some(('\u{2211}', true)),
        "prod" => Some(('\u{220F}', true)),
        "int" => Some(('\u{222B}', false)),
        _ => None,
    }
}

/// Function names set upright: glyph string plus whether limits go under.
pub(crate) fn operator_name(name: &str) -> Option<(&'static str, bool)> {
    match name {
        "sin" => Some(("sin", false)),
        "cos" => Some(("cos", false)),
        "tan" => Some(("tan", false)),
        "log" => Some(("log", false)),
        "ln" => Some(("ln", false)),
        "exp" => Some(("exp", false)),
        "lim" => Some(("lim", true)),
        "max" => Some(("max", true)),
        "min" => Some(("min", true)),
        _ => None,
    }
}

/// Explicit spacing commands, in math units (1mu = 1/18 em).
pub(crate) fn space_command(name: &str) -> Option<i128> {
    match name {
        "," => Some(3),
        ";" => Some(5),
        "quad" => Some(18),
        "qquad" => Some(36),
        _ => None,
    }
}

/// TeX's inter-atom spacing table, in math units. The flag marks the entries
/// that TeX suppresses in script styles.
fn pair_space(left: AtomClass, right: AtomClass) -> (i128, bool) {
    use AtomClass::*;
    const NONE: (i128, bool) = (0, false);
    const THIN: (i128, bool) = (3, false);
    const THIN_S: (i128, bool) = (3, true);
    const MED: (i128, bool) = (4, true);
    const THICK: (i128, bool) = (5, true);
    match (left, right) {
        (Space, _) | (_, Space) => NONE,
        (Ord, Op) | (Op, Ord) | (Op, Op) | (Close, Op) | (Inner, Op) => THIN,
        (Ord, Bin) | (Bin, Ord) | (Bin, Open) | (Bin, Inner) | (Close, Bin) | (Inner, Bin) => MED,
        (Ord, Rel)
        | (Rel, Ord)
        | (Op, Rel)
        | (Rel, Op)
        | (Close, Rel)
        | (Rel, Close)
        | (Rel, Open)
        | (Inner, Rel)
        | (Rel, Inner) => THICK,
        (Ord, Inner)
        | (Op, Inner)
        | (Close, Inner)
        | (Inner, Ord)
        | (Inner, Open)
        | (Inner, Punct)
        | (Inner, Inner) => THIN_S,
        (Punct, _) => THIN_S,
        _ => NONE,
    }
}

/// Map a source letter/digit to Mathematical Double-Struck (`\mathbb`).
pub(crate) fn mathbb_char(ch: char) -> Option<char> {
    Some(match ch {
        'C' => '\u{2102}',
        'H' => '\u{210D}',
        'N' => '\u{2115}',
        'P' => '\u{2119}',
        'Q' => '\u{211A}',
        'R' => '\u{211D}',
        'Z' => '\u{2124}',
        'A'..='Z' => char::from_u32(0x1D538 + (ch as u32 - 'A' as u32))?,
        'a'..='z' => char::from_u32(0x1D552 + (ch as u32 - 'a' as u32))?,
        '0'..='9' => char::from_u32(0x1D7D8 + (ch as u32 - '0' as u32))?,
        _ => return None,
    })
}

/// Map a source letter to Mathematical Script (`\mathcal`). Digits are unchanged.
pub(crate) fn mathcal_char(ch: char) -> Option<char> {
    Some(match ch {
        'B' => '\u{212C}',
        'E' => '\u{2130}',
        'F' => '\u{2131}',
        'H' => '\u{210B}',
        'I' => '\u{2110}',
        'L' => '\u{2112}',
        'M' => '\u{2133}',
        'R' => '\u{211B}',
        'A'..='Z' => char::from_u32(0x1D49C + (ch as u32 - 'A' as u32))?,
        _ => return None,
    })
}

/// Space to insert between two adjacent atoms, in math units.
pub(crate) fn space_mu(left: AtomClass, right: AtomClass, style: MathStyle) -> i128 {
    let (amount, script_suppressed) = pair_space(left, right);
    if script_suppressed && !style.is_display() {
        0
    } else {
        amount
    }
}

/// A `Bin` atom is only binary when it sits between two operands; otherwise TeX
/// demotes it to `Ord` (the minus in `-x`, or in `x = -y`).
pub(crate) fn demote_binaries(classes: &mut [AtomClass]) {
    use AtomClass::*;
    for i in 0..classes.len() {
        if classes[i] != Bin {
            continue;
        }
        let leading = match i.checked_sub(1).map(|p| classes[p]) {
            None => true,
            Some(Bin) | Some(Rel) | Some(Open) | Some(Punct) | Some(Op) | Some(Space) => true,
            Some(_) => false,
        };
        let trailing = match classes.get(i + 1) {
            None => true,
            Some(Rel) | Some(Close) | Some(Punct) => true,
            Some(_) => false,
        };
        if leading || trailing {
            classes[i] = Ord;
        }
    }
}
