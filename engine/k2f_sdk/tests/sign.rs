mod common;

use k2f_package::{inspect_package, pack_bytes, unpack_bytes, IntegrityStatus};
use k2f_paint::{Banner, OpenedDocument};
use k2f_sdk::{generate_key, sign};

#[test]
fn save_is_unsigned_sign_is_a_separate_step() {
    let mut ed = common::open("invoice");
    ed.set_generated_by("agent.invoice-bot");
    common::insert_text(&mut ed, "root", "invoice.total", "body", "100.00");
    let mut pkg = unpack_bytes(&ed.save_bytes().unwrap()).unwrap();
    pkg.manifest.generated_by = Some("agent.invoice-bot".into());
    let bytes = pack_bytes(&pkg).unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert!(pkg.signatures_json.is_none());
    assert_eq!(
        pkg.manifest.generated_by.as_deref(),
        Some("agent.invoice-bot")
    );
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::Unsigned
    );

    let key = generate_key().unwrap();
    let signed = sign(
        &bytes,
        &key.secret_hex,
        Some("Jane Doe"),
        Some(1_704_067_200),
    )
    .unwrap();
    let opened = OpenedDocument::open(&signed).unwrap();
    assert_eq!(opened.banner(), Banner::Signed);
    assert_eq!(opened.signed_by(), Some("Jane Doe"));
    assert_eq!(opened.generated_by(), Some("agent.invoice-bot"));
    assert_eq!(opened.fingerprint(), Some(key.fingerprint.as_str()));
    assert_eq!(opened.status_code(), "SIGNED");
}

#[test]
fn omitted_signed_at_is_current_utc_not_epoch() {
    let mut ed = common::open("invoice");
    common::insert_text(&mut ed, "root", "invoice.total", "body", "1");
    let bytes = ed.save_bytes().unwrap();
    let key = generate_key().unwrap();
    let signed = sign(&bytes, &key.secret_hex, None, None).unwrap();
    let opened = OpenedDocument::open(&signed).unwrap();
    assert_eq!(opened.banner(), Banner::Signed);
    assert!(opened.signed_at().unwrap() > 1_700_000_000);
}
