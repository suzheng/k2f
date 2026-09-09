use k2f_package::unpack_bytes;
use k2f_pdf::export_bytes;

#[test]
fn export_and_unpack_refuse_pdf_as_source() {
    let pdf = export_bytes(
        &std::fs::read(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/published/invoice.K2F"),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    let err = unpack_bytes(&pdf).unwrap_err();
    assert!(err.to_string().contains("PDF_IS_NOT_A_SOURCE"), "got {err}");
    let err = export_bytes(&pdf).unwrap_err();
    assert!(err.to_string().contains("PDF_IS_NOT_A_SOURCE"), "got {err}");
}

#[test]
fn contract_backdrop_blur_exports_via_stamp() {
    let bytes = std::fs::read(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/published/contract.K2F"),
    )
    .unwrap();
    let pdf = export_bytes(&bytes).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}
