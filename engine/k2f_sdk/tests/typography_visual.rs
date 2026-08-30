mod common;

use k2f_core::{LockFile, PaintOp};
use k2f_package::unpack_bytes;
use k2f_paint::{OpenedDocument, OFFICIAL_PNG_SCALE};

#[test]
fn report_package_embeds_markdown_scale_theme() {
    let mut ed = common::open("report");
    common::insert_heading(&mut ed, "root", "doc.h1", 1, "Title");
    common::insert_heading(&mut ed, "root", "doc.h2", 2, "Section");
    common::insert_heading(&mut ed, "root", "doc.h3", 3, "Subsection");
    common::insert_text(
        &mut ed,
        "root",
        "doc.p1",
        "body",
        "Body text should be ink-colored at 12pt, not gray.",
    );
    let bytes = ed.save_bytes().unwrap();
    assert!(bytes.len() > 500);

    let pkg = unpack_bytes(&bytes).unwrap();
    let theme: serde_json::Value = serde_json::from_str(&pkg.theme_json).unwrap();
    assert_eq!(theme["roles"]["h1"]["bold"], true);
    assert_eq!(theme["roles"]["h2"]["bold"], true);
    assert_eq!(theme["roles"]["h3"]["bold"], true);
    assert_eq!(theme["roles"]["body"]["color"], "ink");
    assert_eq!(theme["palette"]["ink"], "#24292f");

    let lock: LockFile = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    let h1_runs: Vec<_> = lock.render_plan.pages[0]
        .ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::DrawText { node_id, runs, .. } if node_id == "doc.h1" => Some(runs),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!h1_runs.is_empty(), "h1 must be painted");
    assert!(
        h1_runs.iter().all(|r| r.style.bold),
        "h1 lock runs must be bold"
    );
    assert_eq!(
        h1_runs[0].style.color.to_ascii_lowercase(),
        "#0969da",
        "h1 accent must resolve to GitHub blue, not a palette token"
    );
    let body_runs: Vec<_> = lock.render_plan.pages[0]
        .ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::DrawText { node_id, runs, .. } if node_id == "doc.p1" => Some(runs),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(
        body_runs.iter().all(|r| !r.style.bold),
        "body must not be bold"
    );
    assert_eq!(
        body_runs[0].style.color.to_ascii_lowercase(),
        "#24292f",
        "body ink must resolve to slate, not a palette token"
    );
}

#[test]
fn report_heading_raster_is_heavier_than_unbolded_lock() {
    let mut ed = common::open("report");
    common::insert_heading(&mut ed, "root", "doc.h1", 1, "Title");
    let bytes = ed.save_bytes().unwrap();
    let bold_png = OpenedDocument::open(&bytes)
        .unwrap()
        .render_page(0, OFFICIAL_PNG_SCALE)
        .unwrap();

    let mut pkg = unpack_bytes(&bytes).unwrap();
    let mut lock: LockFile = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    for page in &mut lock.render_plan.pages {
        for op in &mut page.ops {
            if let PaintOp::DrawText { runs, .. } = op {
                for run in runs {
                    run.style.bold = false;
                }
            }
        }
    }
    pkg.set_lock(&lock).unwrap();
    let regular_png = OpenedDocument::from_package(pkg)
        .unwrap()
        .render_page(0, OFFICIAL_PNG_SCALE)
        .unwrap();
    assert!(
        bold_png != regular_png,
        "report h1 must paint heavier than the same glyphs without bold"
    );
}
