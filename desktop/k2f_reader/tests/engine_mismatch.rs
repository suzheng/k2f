mod common;

use common::{invoice_bytes, pack_with_tampered_lock, published_invoice_bytes};
use k2f_paint::Banner;
use k2f_reader::{AppState, VerifyStatus};

fn assert_intact_then_broken_paints_same_lock(
    bytes: &[u8],
    tamper: impl FnOnce(&mut k2f_core::LockFile),
) {
    let intact = AppState::open(bytes).unwrap();
    assert!(
        matches!(intact.banner(), Banner::Unsigned | Banner::Signed),
        "fixture must start intact, got {:?}",
        intact.banner()
    );
    let old_png = intact.render_current_png().unwrap();
    assert!(old_png.starts_with(b"\x89PNG"));
    let old_layer = intact.text_layer();
    assert!(!old_layer.is_empty());

    let broken = AppState::open(&pack_with_tampered_lock(bytes, tamper)).unwrap();
    assert_eq!(broken.banner(), Banner::BrokenIntegrity);
    assert_eq!(
        broken.banner_str(),
        "BROKEN_INTEGRITY",
        "GUI chrome reads banner_str, not only Banner"
    );
    assert_eq!(broken.status(), VerifyStatus::EngineMismatch);
    assert_eq!(
        broken.status_code(),
        "ENGINE_MISMATCH",
        "web banner subtitle is status_code, not the coarse banner"
    );
    assert_eq!(broken.page_count(), intact.page_count());
    assert_eq!(broken.title(), intact.title());
    assert_eq!(
        broken.render_current_png().unwrap(),
        old_png,
        "broken files must still show the published lock, not a reflow"
    );
    assert_eq!(
        broken.text_layer().len(),
        old_layer.len(),
        "text layer comes from the lock, not a recompile"
    );
    let pdf = broken.export_pdf_bytes().unwrap();
    assert!(
        pdf.starts_with(b"%PDF-"),
        "PDF export must still draw the published lock"
    );
}

/// Current policy: a lock with a foreign engine version still opens and paints.
/// The banner is broken; pixels stay the published lock (no compile-on-open).
#[test]
fn engine_version_mismatch_shows_broken_but_paints_lock() {
    assert_intact_then_broken_paints_same_lock(&invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    });
}

#[test]
fn engine_commit_sha_mismatch_shows_broken_but_paints_lock() {
    assert_intact_then_broken_paints_same_lock(&invoice_bytes(), |lock| {
        lock.engine_commit_sha = "deadbeef".into();
    });
}

#[test]
fn published_invoice_engine_mismatch_paints_every_lock_page() {
    let bytes = published_invoice_bytes();
    let mut intact = AppState::open(&bytes).unwrap();
    assert!(
        intact.page_count() >= 2,
        "published invoice must paginate so mismatch cannot hide on page 0"
    );

    let tampered = pack_with_tampered_lock(&bytes, |lock| {
        lock.engine_version = "9.9.9".into();
    });
    let mut broken = AppState::open(&tampered).unwrap();
    assert_eq!(broken.banner(), Banner::BrokenIntegrity);
    assert_eq!(broken.status(), VerifyStatus::EngineMismatch);
    assert_eq!(broken.status_code(), "ENGINE_MISMATCH");
    assert_eq!(broken.page_count(), intact.page_count());

    for _ in 0..intact.page_count() {
        assert_eq!(
            broken.render_current_png().unwrap(),
            intact.render_current_png().unwrap(),
            "page {} must stay the published lock",
            broken.page()
        );
        broken.next_page();
        intact.next_page();
    }

    let pdf = broken.export_pdf_bytes().unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}
