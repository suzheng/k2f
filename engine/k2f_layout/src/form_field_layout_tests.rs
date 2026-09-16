//! Form-field compile tests: reserved box, in-box glyphs, no reflow.
//!
//! Uses official `templates/blank` theme and bundled Roboto/NotoSansMath faces.

use crate::compile_chunk_with_fonts;
use k2f_core::{GeometryNode, LockFile, PaintOp, Pt};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn official_theme() -> String {
    std::fs::read_to_string(repo_root().join("templates/blank/styles/theme.json"))
        .expect("official blank theme")
}

fn package_fonts() -> BTreeMap<String, Vec<u8>> {
    let fonts_dir = repo_root().join("assets/fonts");
    let mut fonts = BTreeMap::new();
    fonts.insert(
        "assets/fonts/Roboto-Regular.ttf".into(),
        std::fs::read(fonts_dir.join("Roboto-Regular.ttf")).expect("Roboto-Regular"),
    );
    fonts.insert(
        "assets/fonts/NotoSansSC-Regular.otf".into(),
        std::fs::read(fonts_dir.join("NotoSansSC-Regular.otf")).expect("NotoSansSC"),
    );
    fonts.insert(
        "assets/fonts/NotoSansMath-Regular.ttf".into(),
        std::fs::read(fonts_dir.join("NotoSansMath-Regular.ttf")).expect("NotoSansMath"),
    );
    fonts
}

fn compile(content: &str) -> LockFile {
    let json = compile_chunk_with_fonts(content, &official_theme(), &package_fonts(), None)
        .unwrap_or_else(|e| panic!("compile failed: {e}"));
    serde_json::from_str(&json).expect("lock json")
}

fn compile_err(content: &str) -> String {
    compile_chunk_with_fonts(content, &official_theme(), &package_fonts(), None)
        .expect_err("expected compile error")
}

fn collect_id<'a>(geo: &'a GeometryNode, id: &str, out: &mut Vec<&'a GeometryNode>) {
    if geo.id == id {
        out.push(geo);
    }
    for child in &geo.children {
        collect_id(child, id, out);
    }
}

fn boxes_for<'a>(lock: &'a LockFile, id: &str) -> Vec<&'a GeometryNode> {
    let mut out = Vec::new();
    for page in &lock.geometry.pages {
        collect_id(&page.root, id, &mut out);
    }
    out
}

fn only_box<'a>(lock: &'a LockFile, id: &str) -> &'a GeometryNode {
    let boxes = boxes_for(lock, id);
    assert_eq!(
        boxes.len(),
        1,
        "expected one box for {id}, got {}",
        boxes.len()
    );
    boxes[0]
}

fn field_ops<'a>(lock: &'a LockFile, id: &str) -> Vec<&'a PaintOp> {
    lock.render_plan
        .pages
        .iter()
        .flat_map(|p| p.ops.iter())
        .filter(|op| match op {
            PaintOp::DrawBox { node_id, .. } | PaintOp::DrawText { node_id, .. } => node_id == id,
            _ => false,
        })
        .collect()
}

fn a4_root(children: &str) -> String {
    format!(
        r#"{{
        "title": "form-field layout",
        "canvas_mode": "paged",
        "page_config": {{ "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] }},
        "root": {{
            "id": "doc",
            "role": "document",
            "layout": {{ "type": "stack", "direction": "vertical", "gap": 8000 }},
            "content": {{ "type": "container", "value": {{ "children": [ {children} ] }} }}
        }}
    }}"#
    )
}

fn stack_doc_json(value: &str) -> String {
    a4_root(&format!(
        r#"{{
            "id": "doc.title",
            "role": "h1",
            "content": {{ "type": "text", "value": "Application" }}
        }},
        {{
            "id": "doc.name",
            "role": "form_field",
            "variant": "underline",
            "break_inside": "avoid",
            "content": {{
                "type": "form_field",
                "value": {{ "kind": "text", "value": {value}, "placeholder": "Full legal name" }}
            }}
        }},
        {{
            "id": "doc.sign",
            "role": "body",
            "content": {{ "type": "text", "value": "签名" }}
        }}"#
    ))
}

#[test]
fn empty_and_long_value_same_box_and_sibling_y() {
    let empty = compile(&stack_doc_json("\"\""));
    let filled = compile(&stack_doc_json("\"Alexandria Catherine Whittaker-Jones\""));
    let e = only_box(&empty, "doc.name");
    let f = only_box(&filled, "doc.name");
    assert_eq!(e.width, f.width, "field width must not depend on value");
    assert_eq!(e.height, f.height, "field height must not depend on value");
    assert!(e.glyphs.is_empty(), "empty value must have no glyphs");
    assert!(!f.glyphs.is_empty(), "filled value must shape glyphs");

    let e_sign = only_box(&empty, "doc.sign");
    let f_sign = only_box(&filled, "doc.sign");
    assert_eq!(e_sign.y, f_sign.y, "following sibling y must not reflow");
    assert_eq!(e_sign.x, f_sign.x);

    let empty_ops = field_ops(&empty, "doc.name");
    assert!(
        empty_ops
            .iter()
            .any(|op| matches!(op, PaintOp::DrawBox { node_id, .. } if node_id == "doc.name")),
        "empty field must emit DrawBox, got {empty_ops:?}"
    );
    assert!(
        !empty_ops
            .iter()
            .any(|op| matches!(op, PaintOp::DrawText { node_id, .. } if node_id == "doc.name")),
        "empty field must not emit DrawText"
    );
    let filled_ops = field_ops(&filled, "doc.name");
    assert!(
        filled_ops
            .iter()
            .any(|op| matches!(op, PaintOp::DrawText { node_id, .. } if node_id == "doc.name")),
        "filled field must emit DrawText"
    );
}

#[test]
fn horizontal_stack_explicit_width_does_not_move() {
    let wrap = |value: &str| {
        a4_root(&format!(
            r#"{{
            "id": "doc.line",
            "role": "body",
            "layout": {{ "type": "stack", "direction": "horizontal", "gap": 4000, "align_items": "start" }},
            "content": {{ "type": "container", "value": {{ "children": [
                {{
                    "id": "doc.line.pre",
                    "role": "body",
                    "content": {{ "type": "text", "value": "我，" }}
                }},
                {{
                    "id": "doc.line.name",
                    "role": "form_field",
                    "variant": "underline",
                    "break_inside": "avoid",
                    "content": {{
                        "type": "form_field",
                        "value": {{ "kind": "text", "value": {value}, "width": 180000 }}
                    }}
                }},
                {{
                    "id": "doc.line.post",
                    "role": "body",
                    "content": {{ "type": "text", "value": "，同意" }}
                }}
            ] }} }}
        }}"#
        ))
    };
    let empty = compile(&wrap("\"\""));
    let filled = compile(&wrap("\"Alexandria Catherine Whittaker-Jones\""));
    for id in ["doc.line.pre", "doc.line.name", "doc.line.post"] {
        let a = only_box(&empty, id);
        let b = only_box(&filled, id);
        assert_eq!(a.x, b.x, "{id} x moved");
        assert_eq!(a.width, b.width, "{id} width changed");
    }
    assert_eq!(only_box(&empty, "doc.line.name").width, Pt(180_000));
}

#[test]
fn single_line_overflow_clips_to_declared_width() {
    let json = a4_root(
        r#"{
            "id": "doc.tiny",
            "role": "form_field",
            "variant": "underline",
            "break_inside": "avoid",
            "content": {
                "type": "form_field",
                "value": {
                    "kind": "text",
                    "value": "Alexandria Catherine Whittaker-Jones",
                    "width": 22000
                }
            }
        }"#,
    );
    let lock = compile(&json);
    let geo = only_box(&lock, "doc.tiny");
    assert_eq!(geo.width, Pt(22_000));
    assert!(!geo.glyphs.is_empty(), "some prefix glyphs should remain");
    for g in &geo.glyphs {
        assert!(
            g.x_offset + g.x_advance <= geo.width,
            "glyph x+advance {} exceeds box {}",
            (g.x_offset + g.x_advance).0,
            geo.width.0
        );
    }
}

#[test]
fn multiline_height_is_lines_times_metrics_and_clips_extra_line() {
    let json = a4_root(
        r#"{
            "id": "doc.addr",
            "role": "form_field",
            "variant": "box",
            "break_inside": "avoid",
            "content": {
                "type": "form_field",
                "value": {
                    "kind": "multiline",
                    "value": "one\ntwo\nthree\nFOURTH",
                    "lines": 3
                }
            }
        }"#,
    );
    let lock = compile(&json);
    let geo = only_box(&lock, "doc.addr");
    // Official form_field: font_size 11000, line_height_mult 1400, pad t/b 2000.
    let line_h = Pt(11_000 * 1400 / 1000);
    let pad_v = Pt(4_000);
    assert_eq!(geo.height, line_h * 3 + pad_v);

    let mut ys: Vec<i128> = geo.glyphs.iter().map(|g| g.y_offset.0).collect();
    ys.sort();
    ys.dedup();
    assert_eq!(ys.len(), 3, "fourth visual line must not appear, ys={ys:?}");
    assert!(
        !geo.glyphs.is_empty(),
        "first three lines should produce glyphs"
    );
}

#[test]
fn checkbox_square_and_mark() {
    let json_for = |value: &str| {
        a4_root(&format!(
            r#"{{
            "id": "doc.agree",
            "role": "form_field",
            "variant": "checkbox",
            "break_inside": "avoid",
            "content": {{
                "type": "form_field",
                "value": {{ "kind": "checkbox", "value": {value} }}
            }}
        }}"#
        ))
    };
    let off = compile(&json_for("\"\""));
    let on = compile(&json_for("\"true\""));
    let a = only_box(&off, "doc.agree");
    let b = only_box(&on, "doc.agree");
    assert_eq!(a.width, a.height, "checkbox must be square");
    assert_eq!(a.width, b.width);
    assert_eq!(a.height, b.height);
    let font_size = Pt(11_000);
    let delta = (a.width.0 - font_size.0).abs();
    assert!(
        delta <= font_size.0,
        "side {} should be the same order as font_size {}",
        a.width.0,
        font_size.0
    );
    assert!(a.glyphs.is_empty(), "unchecked checkbox has no mark");
    assert!(!b.glyphs.is_empty(), "checked checkbox must paint a mark");
}

#[test]
fn unbounded_width_without_width_fails() {
    let json = a4_root(
        r#"{
            "id": "doc.line",
            "role": "body",
            "layout": { "type": "stack", "direction": "horizontal", "gap": 4000 },
            "content": { "type": "container", "value": { "children": [
                {
                    "id": "doc.line.pre",
                    "role": "body",
                    "content": { "type": "text", "value": "我，" }
                },
                {
                    "id": "doc.line.name",
                    "role": "form_field",
                    "variant": "underline",
                    "break_inside": "avoid",
                    "content": {
                        "type": "form_field",
                        "value": { "kind": "text", "value": "" }
                    }
                }
            ] } }
        }"#,
    );
    let err = compile_err(&json);
    assert!(err.contains("FORM_FIELD_UNBOUNDED_WIDTH"), "got {err}");
}

#[test]
fn break_inside_avoid_keeps_field_on_one_page() {
    let json = format!(
        r#"{{
        "title": "keep field",
        "canvas_mode": "paged",
        "page_config": {{ "width": 200000, "height": 50000, "margin": [5000, 5000, 5000, 5000] }},
        "root": {{
            "id": "doc",
            "role": "document",
            "layout": {{ "type": "stack", "direction": "vertical", "gap": 2000 }},
            "content": {{ "type": "container", "value": {{ "children": [
                {{ "id": "doc.a", "role": "body", "content": {{ "type": "text", "value": "Line" }} }},
                {{ "id": "doc.b", "role": "body", "content": {{ "type": "text", "value": "Line" }} }},
                {{
                    "id": "doc.name",
                    "role": "form_field",
                    "variant": "underline",
                    "break_inside": "avoid",
                    "content": {{
                        "type": "form_field",
                        "value": {{ "kind": "text", "value": "Ada", "width": 80000 }}
                    }}
                }}
            ] }} }}
        }}
    }}"#
    );
    let lock = compile(&json);
    let boxes = boxes_for(&lock, "doc.name");
    assert_eq!(boxes.len(), 1, "field must not split across pages");
    assert_eq!(lock.geometry.pages.len(), 2, "field should move as a unit");
}
