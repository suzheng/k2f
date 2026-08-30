use k2f_paint::{Banner, OpenedDocument};
use k2f_pdf::{export_opened, PdfScale};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn open(name: &str) -> OpenedDocument {
    let bytes = std::fs::read(repo_root().join(name)).unwrap();
    OpenedDocument::open(&bytes).unwrap()
}

#[test]
fn invoice_goldens_cover_unsigned_signed_and_broken() {
    let ordinary = open("examples/published/invoice.K2F");
    assert_eq!(ordinary.banner(), Banner::Unsigned);
    assert!(ordinary.page_count() >= 2, "line items must paginate");
    assert!(export_opened(&ordinary, PdfScale::DEFAULT).unwrap().starts_with(b"%PDF-"));

    let signed = open("examples/invoice_signed.K2F");
    assert_eq!(signed.banner(), Banner::Signed);
    assert_eq!(signed.signed_by(), Some("Acme Billing"));
    assert_eq!(
        signed.appearance_hash(),
        ordinary.appearance_hash(),
        "signing must not change the lock"
    );

    let broken = open("examples/invoice_broken.K2F");
    assert_eq!(broken.banner(), Banner::BrokenIntegrity);
    assert_eq!(
        broken.render_page(0, k2f_paint::OFFICIAL_PNG_SCALE).unwrap(),
        ordinary
            .render_page(0, k2f_paint::OFFICIAL_PNG_SCALE)
            .unwrap(),
        "broken files still paint the published lock"
    );
}
