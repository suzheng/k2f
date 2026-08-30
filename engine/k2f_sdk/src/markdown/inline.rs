use k2f_core::Modifier;

pub(crate) const MAX_MODIFIERS: usize = 50;

#[derive(Debug, Clone, Copy)]
enum MarkKind {
    Strong,
    Emphasis,
    Strikethrough,
    Link,
}

#[derive(Debug)]
struct Mark {
    kind: MarkKind,
    start: usize,
    url: String,
}

#[derive(Debug, Default)]
pub struct InlineBuf {
    pub text: String,
    pub modifiers: Vec<Modifier>,
    marks: Vec<Mark>,
}

impl InlineBuf {
    pub fn push_text(&mut self, s: &str) {
        self.text.push_str(s);
    }

    pub fn soft_break(&mut self) {
        self.text.push(' ');
    }

    pub fn hard_break(&mut self) {
        self.text.push('\n');
    }

    pub fn start_strong(&mut self) {
        self.open(MarkKind::Strong, String::new());
    }

    pub fn start_emphasis(&mut self) {
        self.open(MarkKind::Emphasis, String::new());
    }

    pub fn start_strikethrough(&mut self) {
        self.open(MarkKind::Strikethrough, String::new());
    }

    pub fn start_link(&mut self, url: String) {
        self.open(MarkKind::Link, url);
    }

    pub fn end_strong(&mut self) {
        self.close(MarkKind::Strong);
    }

    pub fn end_emphasis(&mut self) {
        self.close(MarkKind::Emphasis);
    }

    pub fn end_strikethrough(&mut self) {
        self.close(MarkKind::Strikethrough);
    }

    pub fn end_link(&mut self) {
        self.close(MarkKind::Link);
    }

    pub fn push_code(&mut self, code: &str) {
        let start = self.text.len();
        self.text.push_str(code);
        self.push_mod(start, self.text.len(), "emphasis", "code");
    }

    pub fn push_math(&mut self, tex: &str) {
        let start = self.text.len();
        self.text.push('\u{FFFC}');
        self.push_mod(start, self.text.len(), "math", tex.trim());
    }

    pub fn take(&mut self) -> (String, Vec<Modifier>) {
        self.marks.clear();
        (
            std::mem::take(&mut self.text),
            std::mem::take(&mut self.modifiers),
        )
    }

    pub fn take_trimmed(&mut self) -> (String, Vec<Modifier>) {
        let (text, mods) = self.take();
        trim_text_mods(text, mods)
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn open(&mut self, kind: MarkKind, url: String) {
        self.marks.push(Mark {
            kind,
            start: self.text.len(),
            url,
        });
    }

    fn close(&mut self, kind: MarkKind) {
        let Some(idx) = self.marks.iter().rposition(|m| matches_kind(m.kind, kind)) else {
            return;
        };
        let mark = self.marks.remove(idx);
        let end = self.text.len();
        if mark.start >= end {
            return;
        }
        let (ty, intent) = match mark.kind {
            MarkKind::Strong => ("emphasis", "strong".to_string()),
            MarkKind::Emphasis => ("emphasis", "emphasis".to_string()),
            MarkKind::Strikethrough => ("strikethrough", "default".to_string()),
            MarkKind::Link => ("link", mark.url),
        };
        self.push_mod(mark.start, end, ty, &intent);
    }

    fn push_mod(&mut self, start: usize, end: usize, ty: &str, intent: &str) {
        self.modifiers.push(Modifier {
            range: [start, end],
            mod_type: ty.into(),
            intent: intent.into(),
        });
    }
}

fn matches_kind(a: MarkKind, b: MarkKind) -> bool {
    std::mem::discriminant(&a) == std::mem::discriminant(&b)
}

fn trim_text_mods(text: String, mods: Vec<Modifier>) -> (String, Vec<Modifier>) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (String::new(), vec![]);
    }
    let start = text.len() - text.trim_start().len();
    let end = start + trimmed.len();
    let mods = mods
        .into_iter()
        .filter_map(|mut m| {
            let s = m.range[0].clamp(start, end);
            let e = m.range[1].clamp(start, end);
            if s >= e
                || !trimmed.is_char_boundary(s - start)
                || !trimmed.is_char_boundary(e - start)
            {
                return None;
            }
            m.range = [s - start, e - start];
            Some(m)
        })
        .collect();
    (trimmed.to_string(), mods)
}
