use serde::{Deserialize, Serialize};

/// Whether a node may split across pages. Agents pick an enum; they do not invent an algorithm.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BreakInside {
    /// Keep whole if it fits a page; split only at allowed points when taller than a page.
    #[default]
    Auto,
    /// Never split. Move to the next page; fail if taller than one page.
    Avoid,
}

impl BreakInside {
    pub fn is_auto(&self) -> bool {
        matches!(self, BreakInside::Auto)
    }
}

pub fn is_false(v: &bool) -> bool {
    !*v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_auto() {
        assert_eq!(BreakInside::default(), BreakInside::Auto);
        assert!(BreakInside::Auto.is_auto());
        assert!(!BreakInside::Avoid.is_auto());
    }

    #[test]
    fn json_omits_auto_when_using_skip() {
        #[derive(Serialize)]
        struct N {
            #[serde(skip_serializing_if = "BreakInside::is_auto")]
            break_inside: BreakInside,
            #[serde(skip_serializing_if = "is_false")]
            keep_with_next: bool,
        }
        let s = serde_json::to_string(&N {
            break_inside: BreakInside::Auto,
            keep_with_next: false,
        })
        .unwrap();
        assert_eq!(s, "{}");
        let s = serde_json::to_string(&N {
            break_inside: BreakInside::Avoid,
            keep_with_next: true,
        })
        .unwrap();
        assert!(s.contains("avoid"));
        assert!(s.contains("keep_with_next"));
    }
}
