use k2f_core::NodeContent;
use k2f_package::{generate_secret_key, pack_bytes, sign_package, unpack_bytes};
use k2f_paint::{Banner, OpenedDocument};
use k2f_pdf::{export_opened, PdfScale};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn published_invoice_bytes() -> Vec<u8> {
    std::fs::read(repo_root().join("examples/published/invoice.K2F")).unwrap()
}

#[test]
fn invoice_goldens_cover_unsigned_signed_and_broken() {
    let bytes = published_invoice_bytes();
    let ordinary = OpenedDocument::open(&bytes).unwrap();
    assert_eq!(ordinary.banner(), Banner::Unsigned);
    assert!(ordinary.page_count() >= 2, "line items must paginate");
    assert!(export_opened(&ordinary, PdfScale::DEFAULT)
        .unwrap()
        .starts_with(b"%PDF-"));

    let mut signed_pkg = unpack_bytes(&bytes).unwrap();
    let key = generate_secret_key().unwrap();
    sign_package(&mut signed_pkg, &key, Some("Acme Billing"), 1_704_067_200).unwrap();
    let signed = OpenedDocument::open(&pack_bytes(&signed_pkg).unwrap()).unwrap();
    assert_eq!(signed.banner(), Banner::Signed);
    assert_eq!(signed.signed_by(), Some("Acme Billing"));
    assert_eq!(
        signed.appearance_hash(),
        ordinary.appearance_hash(),
        "signing must not change the lock"
    );

    let mut broken_pkg = unpack_bytes(&bytes).unwrap();
    let node = k2f_core::find_node_mut(&mut broken_pkg.root, "invoice.header").unwrap();
    match &mut node.content {
        NodeContent::Text(t) => t.push_str(" TAMPERED"),
        other => panic!("expected invoice.header text, got {other:?}"),
    }
    let broken = OpenedDocument::from_package(broken_pkg).unwrap();
    assert_eq!(broken.banner(), Banner::BrokenIntegrity);
    assert_eq!(
        broken
            .render_page(0, k2f_paint::OFFICIAL_PNG_SCALE)
            .unwrap(),
        ordinary
            .render_page(0, k2f_paint::OFFICIAL_PNG_SCALE)
            .unwrap(),
        "broken files still paint the published lock"
    );
}
