mod common;

use common::{
    invoice_bytes, pack_with_rebound_engine, pack_with_tampered_lock, published_invoice_bytes,
};
use k2f_paint::Banner;
use k2f_reader::{AppState, VerifyStatus};

fn assert_quiet_mismatch_paints_same_lock(
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

    let opened = AppState::open(&pack_with_rebound_engine(bytes, tamper)).unwrap();
    assert_eq!(opened.banner(), Banner::Unsigned);
    assert_eq!(opened.banner_str(), "UNSIGNED");
    assert_eq!(opened.status(), VerifyStatus::EngineMismatch);
    assert_eq!(opened.status_code(), "UNSIGNED");
    assert_eq!(opened.hash_code(), "ENGINE_MISMATCH");
    assert_eq!(opened.page_count(), intact.page_count());
    assert_eq!(opened.title(), intact.title());
    assert_eq!(
        opened.render_current_png().unwrap(),
        old_png,
        "foreign-engine files must still show the published lock, not a reflow"
    );
    assert_eq!(
        opened.text_layer().len(),
        old_layer.len(),
        "text layer comes from the lock, not a recompile"
    );
    let pdf = opened.export_pdf_bytes().unwrap();
    assert!(
        pdf.starts_with(b"%PDF-"),
        "PDF export must still draw the published lock"
    );
}

/// Self-consistent lock compiled by another engine: quiet read, pixels stay the published lock.
#[test]
fn engine_version_mismatch_is_quiet_and_paints_lock() {
    assert_quiet_mismatch_paints_same_lock(&invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    });
}

#[test]
fn engine_commit_sha_mismatch_is_quiet_and_paints_lock() {
    assert_quiet_mismatch_paints_same_lock(&invoice_bytes(), |lock| {
        lock.engine_commit_sha = "deadbeef".into();
    });
}

#[test]
fn published_invoice_foreign_engine_paints_every_lock_page() {
    let bytes = published_invoice_bytes();
    let mut intact = AppState::open(&bytes).unwrap();
    assert!(
        intact.page_count() >= 2,
        "published invoice must paginate so mismatch cannot hide on page 0"
    );

    let rebound = pack_with_rebound_engine(&bytes, |lock| {
        lock.engine_version = "9.9.9".into();
    });
    let mut opened = AppState::open(&rebound).unwrap();
    assert_eq!(opened.banner(), Banner::Unsigned);
    assert_eq!(opened.status(), VerifyStatus::EngineMismatch);
    assert_eq!(opened.hash_code(), "ENGINE_MISMATCH");
    assert_eq!(opened.page_count(), intact.page_count());

    for _ in 0..intact.page_count() {
        assert_eq!(
            opened.render_current_png().unwrap(),
            intact.render_current_png().unwrap(),
            "page {} must stay the published lock",
            opened.page()
        );
        opened.next_page();
        intact.next_page();
    }

    let pdf = opened.export_pdf_bytes().unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}

#[test]
fn engine_field_tamper_without_rebind_is_broken_but_paints_lock() {
    let bytes = invoice_bytes();
    let intact = AppState::open(bytes.as_slice()).unwrap();
    let old_png = intact.render_current_png().unwrap();
    let broken = AppState::open(&pack_with_tampered_lock(&bytes, |lock| {
        lock.engine_version = "9.9.9".into();
    }))
    .unwrap();
    assert_eq!(broken.banner(), Banner::BrokenIntegrity);
    assert_eq!(broken.status(), VerifyStatus::AppearanceChanged);
    assert_eq!(broken.status_code(), "APPEARANCE_CHANGED");
    assert_eq!(broken.render_current_png().unwrap(), old_png);
}
