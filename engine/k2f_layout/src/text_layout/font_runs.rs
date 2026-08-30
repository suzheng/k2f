use crate::LayoutContext;
use k2f_text::split_by_coverage;

use super::types::{push_or_merge_run, TextRun};

/// Split styled runs when the primary face lacks glyphs (emoji companion, etc.).
pub fn split_font_runs(runs: Vec<TextRun>, ctx: &LayoutContext) -> Result<Vec<TextRun>, String> {
    let mut out = Vec::new();
    for run in runs {
        if run.math_tex.is_some() {
            push_or_merge_run(&mut out, run);
            continue;
        }
        let primary = crate::style::resolve_font_family_key(&run.style.font_family, ctx.theme);
        let segments = split_by_coverage(&run.text, &primary, ctx.fonts)?;
        let mut byte = run.start;
        for segment in segments {
            let len = segment.text.len();
            let mut style = run.style.clone();
            style.font_family = segment.font_key;
            push_or_merge_run(
                &mut out,
                TextRun {
                    start: byte,
                    end: byte + len,
                    style,
                    text: segment.text,
                    math_tex: None,
                },
            );
            byte += len;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;
    use crate::{LayoutContext, Theme};
    use k2f_core::Pt;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn repo_font(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
    }

    fn report_ctx() -> LayoutContext<'static> {
        let mut fonts_map = BTreeMap::new();
        fonts_map.insert(
            "assets/fonts/NotoSansSC-Regular.otf".into(),
            repo_font("NotoSansSC-Regular.otf"),
        );
        fonts_map.insert(
            "assets/fonts/NotoEmoji-Regular.ttf".into(),
            repo_font("NotoEmoji-Regular.ttf"),
        );
        let lib = crate::fonts::load_font_library(&fonts_map).unwrap();
        let theme: Theme = serde_json::from_str(
            r#"{"palette":{},"roles":{"body":{"font_family":"NotoSansSC-Regular","font_size":12000,"line_height_mult":1600,"color":"ink"}}}"#,
        )
        .unwrap();
        let fonts = Box::leak(Box::new(lib));
        let theme = Box::leak(Box::new(theme));
        LayoutContext::new(fonts, theme)
    }

    #[test]
    fn body_run_with_checkmark_uses_emoji_face() {
        let ctx = report_ctx();
        let style = Style {
            font_family: "NotoSansSC-Regular".into(),
            font_size: Pt(12000),
            line_height_mult: 1600,
            color: "ink".into(),
            ..Style::default()
        };
        let runs = vec![TextRun {
            start: 0,
            end: "done ✅".len(),
            style,
            text: "done ✅".into(),
            math_tex: None,
        }];
        let split = split_font_runs(runs, &ctx).unwrap();
        assert_eq!(split.len(), 2);
        assert_eq!(split[0].style.font_family, "NotoSansSC-Regular");
        assert_eq!(split[1].style.font_family, "NotoEmoji-Regular");
    }
}
