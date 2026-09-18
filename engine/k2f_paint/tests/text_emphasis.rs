mod common;

use k2f_core::{LockFile, PaintOp};
use k2f_layout::LayoutEngine;
use k2f_paint::{
    decoration_lines, geo_for_op_str, index_geometry_multi_str, load_faces,
    render_lockfile_page_to_png, single_font_map, OFFICIAL_PNG_SCALE,
};

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

fn set_runs_underline(lock: &mut LockFile, underline: bool) {
    for page in &mut lock.render_plan.pages {
        for op in &mut page.ops {
            if let PaintOp::DrawText { runs, .. } = op {
                for run in runs {
                    run.style.underline = underline;
                }
            }
        }
    }
}

fn set_runs_strikethrough(lock: &mut LockFile, strikethrough: bool) {
    for page in &mut lock.render_plan.pages {
        for op in &mut page.ops {
            if let PaintOp::DrawText { runs, .. } = op {
                for run in runs {
                    run.style.strikethrough = strikethrough;
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

fn png_with_flags(underline: bool, strikethrough: bool) -> Vec<u8> {
    let fonts = single_font_map(&common::font_bytes());
    let mut lock = compile_body_lock();
    set_runs_underline(&mut lock, underline);
    set_runs_strikethrough(&mut lock, strikethrough);
    render_lockfile_page_to_png(&lock, 0, OFFICIAL_PNG_SCALE, &fonts, &Default::default()).unwrap()
}

#[test]
fn underline_flag_changes_raster() {
    assert!(
        png_with_flags(false, false) != png_with_flags(true, false),
        "paint must stroke a line when TextPaintStyle.underline is true"
    );
}

#[test]
fn strikethrough_flag_changes_raster() {
    assert!(
        png_with_flags(false, false) != png_with_flags(false, true),
        "paint must stroke a line when TextPaintStyle.strikethrough is true"
    );
}

#[test]
fn decoration_lines_follow_run_flags() {
    let fonts = single_font_map(&common::font_bytes());
    let faces = load_faces(&fonts).unwrap();
    let mut lock = compile_body_lock();
    set_runs_underline(&mut lock, true);
    let mut geo_by_id = std::collections::HashMap::new();
    index_geometry_multi_str(&lock.geometry.pages[0].root, &mut geo_by_id);
    let mut found = 0;
    for op in &lock.render_plan.pages[0].ops {
        let PaintOp::DrawText {
            node_id,
            rect,
            runs,
        } = op
        else {
            continue;
        };
        let geo = geo_for_op_str(&geo_by_id, node_id, rect).unwrap();
        let lines = decoration_lines(&faces, geo, rect, runs);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].x1_pt > lines[0].x0_pt);
        assert!(lines[0].thickness_pt > 0.0);
        found += 1;
    }
    assert!(found > 0, "compiled body lock must emit DrawText");
}
