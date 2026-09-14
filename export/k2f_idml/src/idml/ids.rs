/// Deterministic XML Name: `k` plus a suffix. Never starts with a digit.
pub fn k(n: impl std::fmt::Display) -> String {
    format!("k{n}")
}

pub fn spread_self(i: usize) -> String {
    k(format!("Spread{i}"))
}

pub fn page_self(i: usize) -> String {
    k(format!("Page{i}"))
}

pub fn bg_self(i: usize) -> String {
    k(format!("Bg{i}"))
}
