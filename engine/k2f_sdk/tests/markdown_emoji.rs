use k2f_core::{for_each_node, NodeContent, TextGlyphRun};
use k2f_package::unpack_bytes;
use k2f_sdk::{markdown_to_k2f, MarkdownOptions};

fn lock_runs(bytes: &[u8]) -> Vec<TextGlyphRun> {
    let pkg = unpack_bytes(bytes).unwrap();
    let lock: k2f_core::LockFile = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    let mut runs = Vec::new();
    fn walk(node: &k2f_core::GeometryNode, runs: &mut Vec<TextGlyphRun>) {
        runs.extend(node.text_runs.clone());
        for child in &node.children {
            walk(child, runs);
        }
    }
    for page in &lock.geometry.pages {
        walk(&page.root, &mut runs);
    }
    runs
}

#[test]
fn markdown_checkmark_compiles_with_sc_and_emoji_fonts() {
    let result =
        markdown_to_k2f("done ✅\n", MarkdownOptions::default()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(result.bytes[0], 0x50);
    assert_eq!(result.bytes[1], 0x4b);
    let pkg = unpack_bytes(&result.bytes).unwrap();
    assert!(pkg
        .fonts
        .contains_key("assets/fonts/NotoSansSC-Regular.otf"));
    assert!(pkg.fonts.contains_key("assets/fonts/NotoEmoji-Regular.ttf"));
    assert!(pkg
        .fonts
        .contains_key("assets/fonts/NotoSansMath-Regular.ttf"));
    let runs = lock_runs(&result.bytes);
    assert!(
        runs.iter()
            .any(|r| r.style.font_family == "NotoEmoji-Regular"),
        "expected emoji face in lock runs: {:?}",
        runs.iter()
            .map(|r| &r.style.font_family)
            .collect::<Vec<_>>()
    );
}

#[test]
fn pepkio_supabase_plan_compiles_with_checkmark() {
    let path = std::path::PathBuf::from(
        "/Users/mingyanfang/Documents/suzheng/apps/pepkio/docs/plan_docs/outdated/supabase_central_idp_subdomains_plan.md",
    );
    if !path.is_file() {
        eprintln!("skip pepkio plan: {}", path.display());
        return;
    }
    let md = std::fs::read_to_string(&path).unwrap();
    let result = markdown_to_k2f(
        &md,
        MarkdownOptions::new("Supabase plan", "report").unwrap(),
    )
    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut blob = String::new();
    let root = unpack_bytes(&result.bytes).unwrap().root;
    for_each_node(&root, &mut |n| {
        if let NodeContent::Text(t) = &n.content {
            blob.push_str(t);
            blob.push('\n');
        }
    });
    assert!(blob.contains('✅'), "body must retain checkmark emoji");
}
