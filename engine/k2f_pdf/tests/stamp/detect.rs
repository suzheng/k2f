use k2f_core::LockFile;
use k2f_paint::OpenedDocument;
use k2f_pdf::page_needs_stamp;
use std::path::PathBuf;

mod helpers;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn open(name: &str) -> OpenedDocument {
    let bytes = std::fs::read(repo_root().join(name)).unwrap();
    OpenedDocument::open(&bytes).unwrap()
}

fn page_ops(lock: &LockFile, page_idx: usize) -> &[k2f_core::PaintOp] {
    let page = &lock.geometry.pages[page_idx];
    lock.render_plan
        .pages
        .iter()
        .find(|p| p.index == page.index)
        .map(|p| p.ops.as_slice())
        .unwrap()
}

#[test]
fn invoice_content_pages_are_vector() {
    let doc = open("examples/published/invoice.K2F");
    let lock = doc.lock().unwrap();
    for i in 0..lock.geometry.pages.len() {
        assert!(
            !page_needs_stamp(page_ops(lock, i)),
            "invoice page {i} must stay vector"
        );
    }
}

#[test]
fn glass_card_needs_stamp() {
    let doc = helpers::glass_card();
    let lock = doc.lock().unwrap();
    assert!(
        page_needs_stamp(page_ops(lock, 0)),
        "card.glass must trigger stamp path"
    );
}
