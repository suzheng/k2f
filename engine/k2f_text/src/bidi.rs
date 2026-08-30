use unicode_bidi::BidiInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BidiRun {
    pub start: usize,
    pub end: usize,
    pub rtl: bool,
}

/// Resolve embedding levels, then return visual runs for shaping.
/// Line breaking happens first; call this per wrapped line (or on a wrap token).
pub fn visual_runs(text: &str) -> Vec<BidiRun> {
    if text.is_empty() {
        return vec![];
    }
    let info = BidiInfo::new(text, None);
    let mut out = Vec::new();
    for para in &info.paragraphs {
        let line = para.range.clone();
        let (levels, runs) = info.visual_runs(para, line);
        for range in runs {
            if range.start >= range.end {
                continue;
            }
            let rtl = levels.get(range.start).map(|l| l.is_rtl()).unwrap_or(false);
            out.push(BidiRun {
                start: range.start,
                end: range.end,
                rtl,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_is_one_ltr_run() {
        let runs = visual_runs("Hello");
        assert_eq!(
            runs,
            vec![BidiRun {
                start: 0,
                end: 5,
                rtl: false
            }]
        );
    }

    #[test]
    fn hebrew_word_is_rtl() {
        let text = "שלום";
        let runs = visual_runs(text);
        assert_eq!(runs.len(), 1);
        assert!(runs[0].rtl);
        assert_eq!(&text[runs[0].start..runs[0].end], text);
    }

    #[test]
    fn mixed_english_hebrew_splits_before_shaping() {
        let text = "Hello שלום";
        let runs = visual_runs(text);
        assert!(
            runs.len() >= 2,
            "expected separate LTR and RTL runs, got {runs:?}"
        );
        let latin = runs.iter().find(|r| !r.rtl).unwrap();
        let hebrew = runs.iter().find(|r| r.rtl).unwrap();
        assert!(text[latin.start..latin.end].contains("Hello"));
        assert!(text[hebrew.start..hebrew.end].contains("שלום"));
        // Visual order: LTR paragraph keeps Hello on the left, so the first visual run is LTR.
        assert!(!runs[0].rtl);
    }
}
