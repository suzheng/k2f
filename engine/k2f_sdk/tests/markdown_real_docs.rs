use k2f_core::{for_each_node, NodeContent, SemanticNode};
use k2f_package::unpack_bytes;
use k2f_sdk::{k2f_to_markdown, markdown_to_k2f, MarkdownOptions};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn report_opts() -> MarkdownOptions {
    MarkdownOptions::new("Document", repo_root().join("templates/report")).unwrap()
}

fn import_file(path: PathBuf) -> k2f_sdk::MarkdownResult {
    let md = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    markdown_to_k2f(&md, report_opts())
        .unwrap_or_else(|e| panic!("{}: import failed: {e}", path.display()))
}

fn root_of(bytes: &[u8]) -> SemanticNode {
    unpack_bytes(bytes).unwrap().root
}

fn role_texts(root: &SemanticNode) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for_each_node(root, &mut |n| {
        if let NodeContent::Text(t) = &n.content {
            out.entry(n.role.clone()).or_default().push(t.clone());
        }
    });
    out
}

fn joined(map: &BTreeMap<String, Vec<String>>, role: &str) -> String {
    map.get(role).map(|v| v.join("\n")).unwrap_or_default()
}

fn has_role(map: &BTreeMap<String, Vec<String>>, role: &str) -> bool {
    map.get(role).is_some_and(|v| !v.is_empty())
}

fn table_count(root: &SemanticNode) -> usize {
    let mut n = 0;
    for_each_node(root, &mut |node| {
        if matches!(node.content, NodeContent::Table(_)) {
            n += 1;
        }
    });
    n
}

fn inventory(root: &SemanticNode) -> BTreeMap<String, Vec<String>> {
    let mut map = role_texts(root);
    for texts in map.values_mut() {
        for t in texts {
            *t = t.split_whitespace().collect::<Vec<_>>().join(" ");
        }
    }
    map
}

fn assert_roundtrip(path: &PathBuf, first: &k2f_sdk::MarkdownResult) {
    let out = k2f_to_markdown(&first.bytes)
        .unwrap_or_else(|e| panic!("{}: export failed: {e}", path.display()));
    let second = markdown_to_k2f(&out, report_opts())
        .unwrap_or_else(|e| panic!("{}: re-import failed: {e}", path.display()));
    assert_eq!(
        inventory(&root_of(&first.bytes)),
        inventory(&root_of(&second.bytes)),
        "{}: role/text inventory differed after roundtrip",
        path.display()
    );
}

#[test]
fn noto_report_covers_ascii_and_sample_cjk() {
    let md = "七「正 * → % ≠ é `$code*`\n";
    markdown_to_k2f(md, report_opts()).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn font_bytes_override_embeds_a_face_that_covers_cjk() {
    let md = "合同\n";
    let roboto = fs::read(repo_root().join("assets/fonts/Roboto-Regular.ttf")).unwrap();
    let err = markdown_to_k2f(md, {
        let mut opts = MarkdownOptions::new("Document", repo_root().join("templates/invoice")).unwrap();
        opts.font_bytes = Some(roboto);
        opts
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("FONT_MISSING_GLYPH"),
        "Roboto override must fail closed on CJK, got {err}"
    );

    let noto = fs::read(repo_root().join("assets/fonts/NotoSansSC-Regular.otf")).unwrap();
    let ok = markdown_to_k2f(md, {
        let mut opts = MarkdownOptions::new("Document", repo_root().join("templates/invoice")).unwrap();
        opts.font_bytes = Some(noto);
        opts
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(ok.bytes[0], 0x50);
    assert_eq!(ok.bytes[1], 0x4b);
}

#[test]
fn phase_one_foundation_compiles_with_tables_code_and_arrows() {
    let path = repo_root().join("tests/fixtures/markdown/real/phase-1-foundation.md");
    let result = import_file(path.clone());
    let root = root_of(&result.bytes);
    let map = role_texts(&root);
    for role in ["h1", "h2", "h3", "body", "list_item", "code"] {
        assert!(has_role(&map, role), "phase-1 missing role {role}");
    }
    assert!(table_count(&root) > 0, "phase-1 missing GFM table");
    let blob = map
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        blob.contains('→'),
        "expected arrow in remaining text:\n{blob}"
    );
    assert!(
        blob.contains('*'),
        "expected * in remaining text (e.g. @/*)"
    );
    let cells = joined(&map, "table_row_cell") + &joined(&map, "table_header_cell");
    assert!(
        cells.contains("→") || blob.contains("→"),
        "arrow should survive table/body import"
    );
    let has_strong = {
        let mut found = false;
        for_each_node(&root_of(&result.bytes), &mut |n| {
            if n.modifiers
                .iter()
                .any(|m| m.mod_type == "emphasis" && m.intent == "strong")
            {
                found = true;
            }
        });
        found
    };
    let has_code_span = {
        let mut found = false;
        for_each_node(&root_of(&result.bytes), &mut |n| {
            if n.modifiers
                .iter()
                .any(|m| m.mod_type == "emphasis" && m.intent == "code")
            {
                found = true;
            }
        });
        found
    };
    assert!(has_strong, "expected strong modifiers");
    assert!(has_code_span, "expected inline code modifiers");
    assert_roundtrip(&path, &result);
}

#[test]
fn cjk_formal_compiles_with_h4_quote_rule_and_cjk() {
    let path = repo_root().join("tests/fixtures/markdown/real/cjk-formal.md");
    let result = import_file(path.clone());
    let root = root_of(&result.bytes);
    let map = role_texts(&root);
    assert!(has_role(&map, "h4"), "cjk-formal must keep #### as h4");
    assert!(has_role(&map, "quote"), "cjk-formal header blockquote");
    assert!(has_role(&map, "rule"), "cjk-formal --- must become rule");
    assert!(has_role(&map, "code"), "cjk-formal text diagram");
    assert!(table_count(&root) > 0, "cjk-formal missing table");
    assert!(has_role(&map, "list_item"), "cjk-formal lists");
    let title = joined(&map, "h1");
    assert!(title.contains('七'), "title missing 七: {title}");
    assert!(title.contains('「'), "title missing 「: {title}");
    assert!(
        !result
            .report
            .warnings
            .iter()
            .any(|w| w.contains("mapped to h3") || w.contains("thematic break")),
        "unexpected skip/flatten warnings: {:?}",
        result.report.warnings
    );
    assert_roundtrip(&path, &result);
}
