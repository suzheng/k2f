#![allow(dead_code)]

pub mod docs;

use k2f_core::{GridTrack, NodeContent, SemanticNode, TableDataSource, TableSpec};
use k2f_package::{pack_bytes, unpack_bytes};
use k2f_sdk::Editor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Debian/Ubuntu packages needed to compile winit/softbuffer on Linux.
/// CI and README must list the same set (X11 uses dlopen of xkbcommon-x11).
pub const LINUX_WINDOW_PACKAGES: &[&str] = &[
    "pkg-config",
    "libxkbcommon-dev",
    "libxkbcommon-x11-dev",
    "libwayland-dev",
    "libx11-dev",
    "libx11-xcb-dev",
    "libxcursor-dev",
    "libxrandr-dev",
    "libxi-dev",
];

pub fn reader_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_k2f-reader"))
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn run_reader(args: &[&str]) -> Output {
    reader_bin().args(args).output().expect("k2f-reader")
}

pub fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("k2f-reader-headless-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn write_k2f(dir: &Path, bytes: &[u8]) -> PathBuf {
    let path = dir.join("doc.K2F");
    std::fs::write(&path, bytes).unwrap();
    path
}

pub fn assert_ok(out: &Output) {
    assert!(
        out.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
}

/// Small Invoice package compiled in-process (engine-current, UNSIGNED).
pub fn invoice_bytes() -> Vec<u8> {
    let mut ed = Editor::open_template("invoice").unwrap();
    insert_heading(&mut ed, "invoice.title", 1, "STATEMENT");
    insert_table(
        &mut ed,
        "invoice.lines",
        &["Item".into(), "Qty".into()],
        &[vec!["A".into(), "1".into()]],
    );
    insert_text(&mut ed, "invoice.total", "Total: 1");
    ed.validate_package().unwrap();
    let mut pkg = unpack_bytes(&ed.save_bytes().unwrap()).unwrap();
    pkg.manifest.title = "STATEMENT".into();
    pack_bytes(&pkg).unwrap()
}

fn child_count(ed: &Editor) -> usize {
    let root: serde_json::Value = serde_json::from_str(&ed.get_node_json("root").unwrap()).unwrap();
    root["content"]["value"]["children"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0)
}

fn insert_node(ed: &mut Editor, node: &SemanticNode) {
    let json = serde_json::to_string(node).unwrap();
    ed.insert_node("root", child_count(ed), &json).unwrap();
}

fn insert_text(ed: &mut Editor, id: &str, text: &str) {
    insert_node(
        ed,
        &SemanticNode {
            id: id.to_string(),
            role: "body".to_string(),
            content: NodeContent::Text(text.to_string()),
            ..Default::default()
        },
    );
}

fn insert_heading(ed: &mut Editor, id: &str, level: u8, text: &str) {
    let role = match level {
        1 => "h1",
        2 => "h2",
        3 => "h3",
        _ => "h4",
    };
    let mut node = SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        content: NodeContent::Text(text.to_string()),
        keep_with_next: true,
        ..Default::default()
    };
    insert_node(ed, &node);
}

fn insert_table(ed: &mut Editor, id: &str, columns: &[String], rows: &[Vec<String>]) {
    let cols = columns.len();
    let widths = vec![GridTrack::Fr { fr: 1 }; cols];
    let mut all_rows = Vec::with_capacity(rows.len() + 1);
    all_rows.push(
        columns
            .iter()
            .enumerate()
            .map(|(ci, label)| SemanticNode {
                id: format!("{id}.h.c{ci}"),
                role: "table_header_cell".to_string(),
                content: NodeContent::Text(label.clone()),
                ..Default::default()
            })
            .collect(),
    );
    for (ri, row) in rows.iter().enumerate() {
        all_rows.push(
            row.iter()
                .enumerate()
                .map(|(ci, value)| SemanticNode {
                    id: format!("{id}.r{ri}.c{ci}"),
                    role: "table_row_cell".to_string(),
                    content: NodeContent::Text(value.clone()),
                    ..Default::default()
                })
                .collect(),
        );
    }
    insert_node(
        ed,
        &SemanticNode {
            id: id.to_string(),
            role: "table".to_string(),
            content: NodeContent::Table(TableSpec {
                column_widths: widths,
                header_rows: 1,
                gap: 4000,
                row_gap: None,
                column_gap: None,
                data: TableDataSource::Inline { rows: all_rows },
            }),
            ..Default::default()
        },
    );
}

/// Committed published invoice (multi-page lock). Not recompiled on open.
pub fn published_invoice_bytes() -> Vec<u8> {
    std::fs::read(repo_root().join("examples/published/invoice.K2F"))
        .expect("examples/published/invoice.K2F")
}

/// Repack after mutating the published lock. Does not recompile.
pub fn pack_with_tampered_lock(bytes: &[u8], f: impl FnOnce(&mut k2f_core::LockFile)) -> Vec<u8> {
    let mut pkg = unpack_bytes(bytes).expect("unpack");
    let mut lock: k2f_core::LockFile =
        serde_json::from_str(pkg.lock_json.as_ref().expect("lock")).expect("lock json");
    f(&mut lock);
    pkg.set_lock(&lock).expect("set_lock");
    pack_bytes(&pkg).expect("pack")
}

pub fn unlocked_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut pkg = unpack_bytes(bytes).expect("unpack");
    pkg.lock_json = None;
    pack_bytes(&pkg).expect("pack")
}

pub fn signed_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut pkg = unpack_bytes(bytes).expect("unpack");
    let key = k2f_package::generate_secret_key().expect("key");
    k2f_package::sign_package(&mut pkg, &key, Some("CI"), 1_704_067_200).expect("sign");
    pack_bytes(&pkg).expect("pack")
}
