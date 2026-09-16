//! Step 4: real catalog/example authoring — compile, markdown emit, no `____` import.

use k2f_core::{for_each_form_field, for_each_node, NodeContent, SemanticNode};
use k2f_package::unpack_bytes;
use k2f_paint::OpenedDocument;
use k2f_sdk::{k2f_to_markdown, markdown_to_k2f, Editor, MarkdownOptions};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn open_dir(rel: &str) -> Editor {
    Editor::open_dir(&repo_root().join(rel)).unwrap_or_else(|e| panic!("open {rel}: {e}"))
}

fn pack(ed: &mut Editor) -> Vec<u8> {
    ed.save_bytes()
        .unwrap_or_else(|e| panic!("compile/relock: {e}"))
}

fn field_ids(root: &SemanticNode) -> Vec<String> {
    let mut ids = Vec::new();
    for_each_form_field(root, &mut |n, _| ids.push(n.id.clone()));
    ids
}

fn root_of(bytes: &[u8]) -> SemanticNode {
    unpack_bytes(bytes).unwrap().root
}

fn load_ex_form() -> SemanticNode {
    let path = repo_root().join("skills/k2f/catalog/content/ex_form.json");
    let json = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("ex_form.json: {e}"))
}

fn underscore_body_nodes(root: &SemanticNode) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for_each_node(root, &mut |n| {
        if let NodeContent::Text(t) = &n.content {
            if t.contains("____") {
                out.push((n.id.clone(), t.clone()));
            }
        }
    });
    out
}

#[test]
fn catalog_ex_form_field_json_is_a_real_application() {
    let node = load_ex_form();
    k2f_core::validate_semantic_tree(&node).unwrap();
    assert_eq!(node.id, "ex.form");
    let ids = field_ids(&node);
    assert!(ids.iter().any(|id| id == "ex.form.name"), "name: {ids:?}");
    assert!(
        ids.iter().any(|id| id == "ex.form.address"),
        "address: {ids:?}"
    );
    assert!(
        ids.iter().any(|id| id == "ex.form.agree"),
        "checkbox: {ids:?}"
    );
    assert_eq!(
        ids.iter()
            .filter(|id| id.starts_with("ex.form.sign."))
            .count(),
        2,
        "signature_block must hold name+date fields, got {ids:?}"
    );
}

#[test]
fn catalog_ex_form_field_compiles_inside_real_catalog() {
    let mut ed = open_dir("skills/k2f/catalog");
    let bytes = pack(&mut ed);
    let doc = OpenedDocument::open(&bytes).unwrap();
    let fields = doc.form_fields();
    let ids: Vec<_> = fields.iter().map(|f| f.id.as_str()).collect();
    for need in ["ex.form.name", "ex.form.address", "ex.form.agree"] {
        assert!(ids.contains(&need), "missing {need} in {ids:?}");
    }
    for f in &fields {
        let boxes = doc.boxes_for(&f.id);
        assert_eq!(
            boxes.len(),
            1,
            "{} split across {} boxes",
            f.id,
            boxes.len()
        );
    }
}

#[test]
fn contract_signatures_are_form_fields_on_one_page() {
    let mut ed = open_dir("examples/contract");
    let bytes = pack(&mut ed);
    let root = root_of(&bytes);
    let leftover = underscore_body_nodes(&root);
    assert!(
        leftover.is_empty(),
        "contract still uses underline glyphs: {leftover:?}"
    );
    let doc = OpenedDocument::open(&bytes).unwrap();
    let fields = doc.form_fields();
    assert!(
        fields.len() >= 4,
        "client/contractor name+date, got {:?}",
        fields.iter().map(|f| &f.id).collect::<Vec<_>>()
    );
    let pages: Vec<usize> = fields.iter().map(|f| f.page).collect();
    let first = pages[0];
    assert!(
        pages.iter().all(|p| *p == first),
        "signature fields must share one page, got {pages:?}"
    );
    for f in &fields {
        assert_eq!(doc.boxes_for(&f.id).len(), 1, "{} split", f.id);
    }
}

#[test]
fn k2f_to_markdown_from_filled_form_field_includes_id_and_value() {
    let mut ed = open_dir("skills/k2f/catalog");
    ed.replace_text("ex.form.name", "Alice").unwrap();
    let bytes = pack(&mut ed);
    let md = k2f_to_markdown(&bytes).unwrap();
    assert!(
        md.contains("<!-- k2f: form_field kind=text id=ex.form.name -->"),
        "missing emit hint: {md}"
    );
    assert!(md.contains("Alice"), "missing value: {md}");
}

#[test]
fn markdown_underscores_are_not_imported_as_form_fields() {
    let opts = MarkdownOptions::new("Doc", repo_root().join("templates/blank")).unwrap();
    let result = markdown_to_k2f(
        "# App\n\nName: [____]\n\nSign: ____________________\n",
        opts,
    )
    .unwrap();
    let root = root_of(&result.bytes);
    let ids = field_ids(&root);
    assert!(
        ids.is_empty(),
        "must not invent form_field from blanks: {ids:?}"
    );
    let bodies: Vec<String> = {
        let mut out = Vec::new();
        for_each_node(&root, &mut |n| {
            if let NodeContent::Text(t) = &n.content {
                if n.role == "body" {
                    out.push(t.clone());
                }
            }
        });
        out
    };
    assert!(
        bodies.iter().any(|t| t.contains("[____]")),
        "literal blanks must stay body text: {bodies:?}"
    );
}
