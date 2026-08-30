mod common;

use k2f_core::{for_each_node, NodeContent, SemanticNode};
use k2f_package::unpack_bytes;
use k2f_sdk::{k2f_to_markdown, markdown_to_k2f, MarkdownOptions, PageSize};
use std::fs;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/markdown")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn import(md: &str) -> k2f_sdk::MarkdownResult {
    markdown_to_k2f(md, MarkdownOptions::default()).unwrap()
}

fn root_of(bytes: &[u8]) -> SemanticNode {
    unpack_bytes(bytes).unwrap().root
}

fn texts(root: &SemanticNode, role: &str) -> Vec<(String, String, Vec<k2f_core::Modifier>)> {
    let mut out = Vec::new();
    for_each_node(root, &mut |n| {
        if n.role == role {
            if let NodeContent::Text(t) = &n.content {
                out.push((n.id.clone(), t.clone(), n.modifiers.clone()));
            }
        }
    });
    out
}

fn invoice_logo_png() -> Vec<u8> {
    let bytes = fs::read(repo_root().join("examples/published/invoice.K2F")).unwrap();
    unpack_bytes(&bytes)
        .unwrap()
        .assets
        .get("assets/images/logo.png")
        .cloned()
        .expect("example invoice embeds a logo")
}

#[test]
fn invoice_table_header_is_item_qty_amount() {
    let md = fs::read_to_string(fixtures_dir().join("invoice_table.md")).unwrap();
    let headers: Vec<String> = texts(&root_of(&import(&md).bytes), "table_header_cell")
        .into_iter()
        .map(|(_, t, _)| t)
        .collect();
    assert_eq!(headers, vec!["Item", "Qty", "Amount"]);
}

#[test]
fn table_cell_keeps_strong() {
    let md = "| Item | Note |\n| --- | --- |\n| **Lock** | Keep ids |\n";
    let cells: Vec<_> = texts(&root_of(&import(md).bytes), "table_row_cell")
        .into_iter()
        .filter(|(_, t, _)| t == "Lock")
        .collect();
    assert_eq!(cells.len(), 1);
    assert!(
        cells[0]
            .2
            .iter()
            .any(|m| m.mod_type == "emphasis" && m.intent == "strong"),
        "expected strong on Lock, got {:?}",
        cells[0].2
    );
}

#[test]
fn escaped_markers_stay_literal() {
    let md = fs::read_to_string(fixtures_dir().join("escaped_markers.md")).unwrap();
    let first = import(&md);
    let body = texts(&root_of(&first.bytes), "body");
    assert!(
        body.iter().any(|(_, t, mods)| {
            t.contains("_underscores_") && t.contains("[brackets]") && mods.is_empty()
        }),
        "expected literal markers, got {body:?}"
    );
    let out = k2f_to_markdown(&first.bytes).unwrap();
    let second = import(&out);
    assert_eq!(
        serde_json::to_value(&root_of(&first.bytes)).unwrap(),
        serde_json::to_value(&root_of(&second.bytes)).unwrap()
    );
}

#[test]
fn inline_math_imports_as_modifier() {
    let result = import("The ratio is $a/b$ in the body.");
    assert!(
        !result.report.warnings.iter().any(|w| w.contains("math")),
        "{:?}",
        result.report.warnings
    );
    let body = texts(&root_of(&result.bytes), "body");
    assert!(
        body.iter().any(|(_, t, mods)| {
            t.contains('\u{FFFC}')
                && mods
                    .iter()
                    .any(|m| m.mod_type == "math" && m.intent.contains("a/b"))
        }),
        "expected inline math modifier, got {body:?}"
    );
}

#[test]
fn inline_math_roundtrips_through_markdown() {
    let first = import("The ratio is $a/b$ in the body.");
    let out = k2f_to_markdown(&first.bytes).unwrap();
    assert!(out.contains("$a/b$"), "{out}");
    assert!(
        !out.contains('\u{FFFC}'),
        "export must not emit U+FFFC: {out}"
    );
    let second = import(&out);
    assert_eq!(
        serde_json::to_value(&root_of(&first.bytes)).unwrap(),
        serde_json::to_value(&root_of(&second.bytes)).unwrap()
    );
}

#[test]
fn table_cell_inline_math_imports_as_modifier() {
    let md = "| Qty | Note |\n| --- | --- |\n| 1 | $a/b$ |\n";
    let result = import(md);
    let cells = texts(&root_of(&result.bytes), "table_row_cell");
    assert!(
        cells.iter().any(|(_, t, mods)| {
            t.contains('\u{FFFC}')
                && mods
                    .iter()
                    .any(|m| m.mod_type == "math" && m.intent.contains("a/b"))
        }),
        "expected math modifier in table cell, got {cells:?}"
    );
}

#[test]
fn display_math_imports_as_node() {
    let result = import("$$\n a^2+b^2=c^2 \n$$");
    assert!(
        !result.report.warnings.iter().any(|w| w.contains("math")),
        "display math should not warn, got {:?}",
        result.report.warnings
    );
    let mut tex = None;
    for_each_node(&root_of(&result.bytes), &mut |n| {
        if let NodeContent::Math(s) = &n.content {
            tex = Some(s.clone());
            assert_eq!(n.role, "math");
        }
    });
    assert!(
        tex.as_deref().is_some_and(|s| s.contains("a^2+b^2=c^2")),
        "expected display math node, got {tex:?}"
    );
}

#[test]
fn display_math_roundtrips_through_markdown() {
    let first = import("$$E=mc^2$$");
    let out = k2f_to_markdown(&first.bytes).unwrap();
    assert!(out.contains("$$"), "{out}");
    assert!(out.contains("E=mc^2"), "{out}");
    let second = import(&out);
    let mut first_tex = None;
    let mut second_tex = None;
    for_each_node(&root_of(&first.bytes), &mut |n| {
        if let NodeContent::Math(s) = &n.content {
            first_tex = Some(s.clone());
        }
    });
    for_each_node(&root_of(&second.bytes), &mut |n| {
        if let NodeContent::Math(s) = &n.content {
            second_tex = Some(s.clone());
        }
    });
    assert_eq!(first_tex, second_tex);
    assert_eq!(first_tex.as_deref(), Some("E=mc^2"));
}

#[test]
fn import_local_png_from_invoice_example() {
    let dir = std::env::temp_dir().join("k2f-md-bridge-image");
    fs::create_dir_all(&dir).unwrap();
    let png = invoice_logo_png();
    fs::write(dir.join("logo.png"), &png).unwrap();
    let result = markdown_to_k2f(
        "# Logo\n\n![Company logo](logo.png)\n",
        MarkdownOptions {
            image_base: dir,
            ..MarkdownOptions::default()
        },
    )
    .unwrap();
    assert!(
        result.report.warnings.is_empty(),
        "{:?}",
        result.report.warnings
    );
    let mut images = 0;
    for_each_node(&root_of(&result.bytes), &mut |n| {
        if matches!(n.content, NodeContent::Image { .. }) {
            images += 1;
        }
    });
    assert_eq!(images, 1);
}

#[test]
fn keep_with_next_comment_roundtrips() {
    let md = "# Title\n\n<!-- k2f: keep_with_next=true -->\nStay with the next block.\n\nNext.\n";
    let first = import(md);
    let bodies: Vec<_> = {
        let mut out = Vec::new();
        for_each_node(&root_of(&first.bytes), &mut |n| {
            if n.role == "body" {
                if let NodeContent::Text(t) = &n.content {
                    out.push((t.clone(), n.keep_with_next));
                }
            }
        });
        out
    };
    assert!(
        bodies.iter().any(|(t, k)| t.contains("Stay with") && *k),
        "{bodies:?}"
    );
    let out = k2f_to_markdown(&first.bytes).unwrap();
    let second = import(&out);
    assert_eq!(
        serde_json::to_value(&root_of(&first.bytes)).unwrap(),
        serde_json::to_value(&root_of(&second.bytes)).unwrap()
    );
}

#[test]
fn h4_stays_h4_without_flatten_warning() {
    let md = fs::read_to_string(fixtures_dir().join("h4_stays_h4.md")).unwrap();
    let result = import(&md);
    let h4 = texts(&root_of(&result.bytes), "h4");
    assert_eq!(h4.len(), 1, "expected one h4, got {h4:?}");
    assert_eq!(h4[0].1, "Deep heading");
    assert!(
        !result
            .report
            .warnings
            .iter()
            .any(|w| w.contains("mapped to h3")),
        "{:?}",
        result.report.warnings
    );
}

#[test]
fn thematic_break_becomes_rule() {
    let result = import("# Title\n\nBefore.\n\n---\n\nAfter.\n");
    let rules = texts(&root_of(&result.bytes), "rule");
    assert_eq!(rules.len(), 1, "expected one rule, got {rules:?}");
    assert!(
        !result
            .report
            .warnings
            .iter()
            .any(|w| w.contains("thematic")),
        "{:?}",
        result.report.warnings
    );
    let out = k2f_to_markdown(&result.bytes).unwrap();
    assert!(out.contains("---"), "export must emit ---: {out}");
}

#[test]
fn signature_role_comment_reimports() {
    let mut ed = common::open("report");
    common::insert_heading(&mut ed, "root", "doc.h1_sign", 1, "Sign");
    common::insert_text(&mut ed, "root", "doc.sig", "body", "Alice Chen");
    ed.set_role("doc.sig", "signature_block", None).unwrap();
    let bytes = ed.save_bytes().unwrap();
    let md = k2f_to_markdown(&bytes).unwrap();
    assert!(md.contains("role=signature_block"), "{md}");
    let back = import(&md);
    let sigs = texts(&root_of(&back.bytes), "signature_block");
    assert_eq!(sigs.len(), 1);
    assert_eq!(sigs[0].1, "Alice Chen");
}

#[test]
fn running_header_with_spaces_roundtrips() {
    let mut ed = common::open("report");
    common::insert_heading(&mut ed, "root", "doc.h1_hdr", 1, "Hdr");
    common::insert_text(&mut ed, "root", "doc.p_001", "body", "Body.");
    ed.set_running_header("Page {{page_current}} of {{page_total}}")
        .unwrap();
    let bytes = ed.save_bytes().unwrap();
    let md = k2f_to_markdown(&bytes).unwrap();
    let back = unpack_bytes(&import(&md).bytes).unwrap();
    let headers: Vec<String> = back
        .manifest
        .running_blocks
        .iter()
        .filter(|rb| rb.position == k2f_core::RunningBlockPosition::Header)
        .filter_map(|rb| match &rb.node.content {
            NodeContent::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(headers, vec!["Page {{page_current}} of {{page_total}}"]);
}
