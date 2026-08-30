mod common;

use k2f_core::PaintOp;
use k2f_paint::{load_faces, placed_glyphs};
use k2f_pdf::{export_opened, parse_notes, source_line, PdfExportOptions, PdfScale};

#[test]
fn invoice_pdf_page_size_and_count_match_lock() {
    let doc = common::invoice();
    let lock = doc.lock().unwrap();
    let pdf = export_opened(&doc, PdfScale::DEFAULT).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    let parsed = lopdf::Document::load_mem(&pdf).unwrap();
    let pages = parsed.get_pages();
    assert_eq!(pages.len(), lock.geometry.pages.len() + 1);
    for (i, id) in pages.values().enumerate() {
        if i >= lock.geometry.pages.len() {
            break;
        }
        let dict = parsed.get_dictionary(*id).unwrap();
        let arr = dict.get(b"MediaBox").unwrap().as_array().unwrap();
        let w = num(&arr[2]);
        let h = num(&arr[3]);
        assert!(
            common::near(w, lock.geometry.pages[i].width.as_f64_pt()),
            "page {i} width {w} vs lock"
        );
        assert!(
            common::near(h, lock.geometry.pages[i].height.as_f64_pt()),
            "page {i} height {h} vs lock"
        );
    }
}

#[test]
fn invoice_pdf_glyph_origins_and_wrap_match_lock() {
    let doc = common::invoice();
    let lock = doc.lock().unwrap();
    let faces = load_faces(doc.fonts()).unwrap();
    let pdf = export_opened(&doc, PdfScale::DEFAULT).unwrap();
    let parsed = lopdf::Document::load_mem(&pdf).unwrap();
    let mut any_wrap = false;
    for (i, id) in parsed.get_pages().values().enumerate() {
        if i >= lock.geometry.pages.len() {
            break;
        }
        let content = parsed.get_page_content(*id).unwrap();
        assert!(
            String::from_utf8_lossy(&content)
                .lines()
                .any(|l| l.trim() == "f" || l.trim() == "f*"),
            "page {i} must draw glyph outlines, not only comments"
        );
        let (_, glyphs, _) = parse_notes(&content);
        let mut expected = Vec::new();
        let page = &lock.geometry.pages[i];
        let plan = lock
            .render_plan
            .pages
            .iter()
            .find(|p| p.index == page.index)
            .unwrap();
        let mut geo = std::collections::HashMap::new();
        common::index(&page.root, &mut geo);
        for op in &plan.ops {
            if let PaintOp::DrawText {
                node_id,
                rect,
                runs,
            } = op
            {
                let g = geo.get(node_id.as_str()).unwrap();
                for placed in placed_glyphs(&faces, g, rect, runs).unwrap() {
                    expected.push((placed.origin_x_pt, placed.origin_y_pt, placed.glyph_id));
                }
            }
        }
        assert_eq!(
            glyphs.len(),
            expected.len(),
            "page {i} glyph count pdf={} lock={}",
            glyphs.len(),
            expected.len()
        );
        for (a, b) in glyphs.iter().zip(expected.iter()) {
            assert!(
                common::near(a.0, b.0) && common::near(a.1, b.1) && a.2 == b.2,
                "{a:?} vs {b:?}"
            );
        }
        if unique_baselines(&expected).len() > 1 {
            any_wrap = true;
        }
    }
    assert!(any_wrap, "invoice must wrap across more than one baseline");
}

#[test]
fn invoice_pdf_embeds_fonts_and_appearance_hash() {
    let doc = common::invoice();
    let hash = doc.appearance_hash().unwrap().to_string();
    let font = doc.fonts().values().next().unwrap().clone();
    let pdf = export_opened(
        &doc,
        PdfExportOptions::new(PdfScale::DEFAULT).with_trust_pack(),
    )
    .unwrap();
    let parsed = lopdf::Document::load_mem(&pdf).unwrap();
    assert!(
        font_file_stream_contains(&parsed, &font[..32]),
        "FontFile2/3 must hold the package TTF"
    );
    let info = parsed.trailer.get(b"Info").unwrap();
    let info_id = info.as_reference().unwrap();
    let dict = parsed.get_dictionary(info_id).unwrap();
    let sub = dict.get(b"Subject").unwrap().as_str().unwrap();
    assert!(
        std::str::from_utf8(sub).unwrap().contains(&hash),
        "trust-pack subject must print the appearance hash"
    );
    assert!(source_line(&hash).contains(&hash));
}

fn font_file_stream_contains(doc: &lopdf::Document, magic: &[u8]) -> bool {
    for obj in doc.objects.values() {
        let Ok(stream) = obj.as_stream() else {
            continue;
        };
        let dict = &stream.dict;
        let is_font_file = dict.get(b"Length1").is_ok()
            || dict.get(b"Subtype").ok().and_then(|s| s.as_name().ok()) == Some(b"OpenType");
        if !is_font_file {
            continue;
        }
        if let Ok(plain) = stream.get_plain_content() {
            if plain.starts_with(magic) {
                return true;
            }
        }
    }
    false
}

fn unique_baselines(glyphs: &[(f64, f64, u16)]) -> Vec<i64> {
    let mut ys: Vec<i64> = glyphs.iter().map(|g| (g.1 * 2.0).round() as i64).collect();
    ys.sort_unstable();
    ys.dedup();
    ys
}

fn num(obj: &lopdf::Object) -> f64 {
    match obj {
        lopdf::Object::Integer(v) => *v as f64,
        lopdf::Object::Real(v) => *v as f64,
        _ => panic!("not a number: {obj:?}"),
    }
}
