mod common;

use common::{assert_png_golden, compile_case, contract_k2f_bytes};
use k2f_paint::{render_lockfile_page_to_png, single_font_map, OpenedDocument, OFFICIAL_PNG_SCALE};

#[test]
fn contract_homepage_matches_golden_png() {
    let png = OpenedDocument::open(&contract_k2f_bytes())
        .unwrap()
        .render_page(0, OFFICIAL_PNG_SCALE)
        .unwrap();
    assert_png_golden("tests/fixtures/paint/contract/page_1.png", &png);
}

#[test]
fn contract_logo_is_not_a_gray_box() {
    let bytes = contract_k2f_bytes();
    let pkg = k2f_package::unpack_bytes(&bytes).unwrap();
    let lock: k2f_core::LockFile = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    let rect = lock.render_plan.pages[0]
        .ops
        .iter()
        .find_map(|op| match op {
            k2f_core::PaintOp::DrawImage { rect, .. } => Some(rect.clone()),
            _ => None,
        })
        .expect("contract page 1 must contain DrawImage");
    let png = OpenedDocument::open(&bytes)
        .unwrap()
        .render_page(0, OFFICIAL_PNG_SCALE)
        .unwrap();
    let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
    let scale = OFFICIAL_PNG_SCALE as f64;
    let left = (rect.x.as_f64_pt() * scale).round().max(0.0) as u32;
    let top = (rect.y.as_f64_pt() * scale).round().max(0.0) as u32;
    let right = ((rect.x + rect.width).as_f64_pt() * scale).round() as u32;
    let bottom = ((rect.y + rect.height).as_f64_pt() * scale).round() as u32;
    let mut red_marks = 0u32;
    for y in top..bottom.min(decoded.height()) {
        for x in left..right.min(decoded.width()) {
            let p = decoded.get_pixel(x, y);
            if p[0] > 150 && p[1] < 80 {
                red_marks += 1;
            }
        }
    }
    assert!(
        red_marks > 20,
        "expected the red logo mark inside DrawImage, found {red_marks} pixels"
    );
}

#[test]
fn elevation_shadows_homepage_matches_golden_png() {
    let lock = compile_case("elevation_shadows");
    let fonts = single_font_map(&common::font_bytes());
    let png =
        render_lockfile_page_to_png(&lock, 0, OFFICIAL_PNG_SCALE, &fonts, &Default::default())
            .unwrap();
    assert_png_golden("tests/fixtures/cases/elevation_shadows/page_1.png", &png);
}

#[test]
fn warning_card_matches_golden_png() {
    let lock = compile_case("card_variants");
    let fonts = single_font_map(&common::font_bytes());
    let png =
        render_lockfile_page_to_png(&lock, 0, OFFICIAL_PNG_SCALE, &fonts, &Default::default())
            .unwrap();
    assert_png_golden("tests/fixtures/cases/card_variants/page_1.png", &png);
}

#[test]
fn running_footer_paints_each_page() {
    let lock = compile_case("running_footer_page_numbers");
    assert!(
        lock.geometry.pages.len() >= 2,
        "expected multi-page fixture, got {}",
        lock.geometry.pages.len()
    );
    let fonts = single_font_map(&common::font_bytes());
    let images = common::case_images("running_footer_page_numbers");
    for i in 0..lock.geometry.pages.len() {
        let png =
            render_lockfile_page_to_png(&lock, i, OFFICIAL_PNG_SCALE, &fonts, &images).unwrap();
        assert_png_golden(
            &format!(
                "tests/fixtures/cases/running_footer_page_numbers/page_{}.png",
                i + 1
            ),
            &png,
        );
    }
}
