mod common;

use k2f_core::{LockFile, PaintOp};
use k2f_layout::LayoutEngine;
use k2f_paint::{render_lockfile_page_to_png, single_font_map, OFFICIAL_PNG_SCALE};

fn compile_body_lock() -> LockFile {
    let json = LayoutEngine::compile_chunk(
        r#"{
            "id": "root",
            "role": "body",
            "content": { "type": "text", "value": "Heading" }
        }"#,
        r#"{
            "palette": {},
            "roles": {
                "body": {
                    "font_family": "default",
                    "font_size": 24000,
                    "line_height_mult": 1250,
                    "color": "black"
                }
            }
        }"#,
        &common::font_bytes(),
    )
    .unwrap();
    serde_json::from_str(&json).unwrap()
}

fn set_runs_bold(lock: &mut LockFile, bold: bool) {
    for page in &mut lock.render_plan.pages {
        for op in &mut page.ops {
            if let PaintOp::DrawText { runs, .. } = op {
                for run in runs {
                    run.style.bold = bold;
                }
            }
        }
    }
}

#[test]
fn bold_flag_changes_raster() {
    let fonts = single_font_map(&common::font_bytes());
    let mut lock = compile_body_lock();
    set_runs_bold(&mut lock, false);
    let regular =
        render_lockfile_page_to_png(&lock, 0, OFFICIAL_PNG_SCALE, &fonts, &Default::default())
            .unwrap();
    set_runs_bold(&mut lock, true);
    let bold =
        render_lockfile_page_to_png(&lock, 0, OFFICIAL_PNG_SCALE, &fonts, &Default::default())
            .unwrap();
    assert!(
        regular != bold,
        "paint must stroke Regular glyphs when TextPaintStyle.bold is true"
    );
}
