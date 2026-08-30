use std::collections::HashSet;

/// ASCII slug heading id. Empty when `text` has no ASCII letters/digits (e.g. CJK).
pub fn slug_heading_id(level: u8, text: &str) -> String {
    let slug = ascii_slug(text);
    if slug.is_empty() {
        String::new()
    } else {
        format!("doc.h{level}_{slug}")
    }
}

pub fn seq_para_id(n: u32) -> String {
    format!("doc.p_{n:03}")
}

pub fn ascii_slug(text: &str) -> String {
    let mut out = String::new();
    let mut underscore = false;
    for c in text.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            underscore = false;
        } else if !out.is_empty() && !underscore {
            out.push('_');
            underscore = true;
        }
    }
    if out.ends_with('_') {
        out.pop();
    }
    out
}

#[derive(Debug, Default)]
pub struct IdGen {
    used: HashSet<String>,
    para: u32,
    h: [u32; 5],
    list: u32,
    table: u32,
    image: u32,
    code: u32,
    quote: u32,
    rule: u32,
    math: u32,
}

impl IdGen {
    pub fn heading(&mut self, level: u8, text: &str) -> String {
        let slug = slug_heading_id(level, text);
        if slug.is_empty() {
            let i = level.min(4) as usize;
            self.h[i] += 1;
            self.take(format!("doc.h{level}_{:03}", self.h[i]))
        } else {
            self.take(slug)
        }
    }

    pub fn para(&mut self) -> String {
        self.para += 1;
        self.take(seq_para_id(self.para))
    }

    pub fn list(&mut self, ordered: bool) -> String {
        self.list += 1;
        let kind = if ordered { "ol" } else { "ul" };
        self.take(format!("doc.{kind}_{:03}", self.list))
    }

    pub fn list_item(&mut self, list_id: &str, index: usize) -> String {
        self.take(format!("{list_id}.i{index}"))
    }

    pub fn table(&mut self) -> String {
        self.table += 1;
        self.take(format!("doc.table_{:03}", self.table))
    }

    pub fn image(&mut self) -> String {
        self.image += 1;
        self.take(format!("doc.img_{:03}", self.image))
    }

    pub fn code(&mut self) -> String {
        self.code += 1;
        self.take(format!("doc.code_{:03}", self.code))
    }

    pub fn quote(&mut self) -> String {
        self.quote += 1;
        self.take(format!("doc.quote_{:03}", self.quote))
    }

    pub fn rule(&mut self) -> String {
        self.rule += 1;
        self.take(format!("doc.rule_{:03}", self.rule))
    }

    pub fn math(&mut self) -> String {
        self.math += 1;
        self.take(format!("doc.eq_{:03}", self.math))
    }

    fn take(&mut self, id: String) -> String {
        if self.used.insert(id.clone()) {
            return id;
        }
        for n in 2.. {
            let cand = format!("{id}_{n}");
            if self.used.insert(cand.clone()) {
                return cand;
            }
        }
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::is_valid_node_id;

    #[test]
    fn heading_slug_id() {
        assert_eq!(slug_heading_id(1, "Overview"), "doc.h1_overview");
        assert!(is_valid_node_id("doc.h1_overview"));
    }

    #[test]
    fn paragraph_sequence() {
        assert_eq!(seq_para_id(3), "doc.p_003");
        assert!(is_valid_node_id("doc.p_003"));
    }

    #[test]
    fn cjk_heading_uses_sequence() {
        let mut ids = IdGen::default();
        assert_eq!(ids.heading(1, "概述"), "doc.h1_001");
        assert!(is_valid_node_id("doc.h1_001"));
    }
}
