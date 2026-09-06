mod common;

use k2f_pptx::{export_bytes, export_opened, PptxError};
use std::io::Cursor;
use zip::ZipArchive;

#[test]
fn slides_keep_sptree_group() {
    let pptx = export_opened(&common::invoice()).unwrap();
    for name in common::unzip_names(&pptx) {
        if !(name.starts_with("ppt/slides/slide")
            && name.ends_with(".xml")
            && !name.contains("_rels"))
        {
            continue;
        }
        let xml = common::xml_in(&pptx, &name);
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        assert!(
            parsed.descendants().any(|n| n.has_tag_name("spTree")),
            "{name} missing spTree"
        );
        assert!(
            parsed.descendants().any(|n| n.has_tag_name("nvGrpSpPr")),
            "{name} missing nvGrpSpPr"
        );
        assert!(
            parsed.descendants().any(|n| n.has_tag_name("grpSpPr")),
            "{name} missing grpSpPr"
        );
    }
}

#[test]
fn zip_entries_use_fixed_1980_timestamp() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let mut zip = ZipArchive::new(Cursor::new(pptx)).unwrap();
    for i in 0..zip.len() {
        let f = zip.by_index(i).unwrap();
        let t = f.last_modified().expect("DOS timestamp");
        assert_eq!(
            (
                t.year(),
                t.month(),
                t.day(),
                t.hour(),
                t.minute(),
                t.second()
            ),
            (1980, 1, 1, 0, 0, 0),
            "{}",
            f.name()
        );
        assert!(
            f.extra_data().unwrap_or(&[]).is_empty(),
            "no extra timestamp fields on {}",
            f.name()
        );
    }
}

#[test]
fn core_xml_title_and_fixed_dates() {
    let doc = common::invoice();
    let pptx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&pptx, "docProps/core.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let title = parsed
        .descendants()
        .find(|n| n.has_tag_name("title"))
        .and_then(|n| n.text())
        .unwrap_or("");
    assert_eq!(title, doc.title());
    for tag in ["created", "modified"] {
        let t = parsed
            .descendants()
            .find(|n| n.has_tag_name(tag))
            .and_then(|n| n.text())
            .unwrap_or("");
        assert_eq!(t, "1980-01-01T00:00:00Z", "{tag}");
    }
}

#[test]
fn required_xml_parts_are_well_formed() {
    let pptx = export_opened(&common::invoice()).unwrap();
    for name in common::unzip_names(&pptx) {
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let xml = common::xml_in(&pptx, &name);
            roxmltree::Document::parse(&xml).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
}

#[test]
fn master_and_layout_xfrm_match_slide_size() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let pres_xml = common::xml_in(&pptx, "ppt/presentation.xml");
    let pres = roxmltree::Document::parse(&pres_xml).unwrap();
    let sz = pres
        .descendants()
        .find(|n| n.has_tag_name("sldSz"))
        .unwrap();
    let cx = sz.attribute("cx").unwrap();
    let cy = sz.attribute("cy").unwrap();
    for part in [
        "ppt/slideMasters/slideMaster1.xml",
        "ppt/slideLayouts/slideLayout1.xml",
    ] {
        let xml = common::xml_in(&pptx, part);
        let doc = roxmltree::Document::parse(&xml).unwrap();
        for tag in ["ext", "chExt"] {
            let n = doc
                .descendants()
                .find(|n| n.has_tag_name(tag))
                .unwrap_or_else(|| panic!("{part} missing {tag}"));
            assert_eq!(n.attribute("cx"), Some(cx), "{part} {tag} cx");
            assert_eq!(n.attribute("cy"), Some(cy), "{part} {tag} cy");
        }
    }
}

#[test]
fn lockless_invoice_returns_unlocked() {
    let src = common::invoice_bytes();
    let stripped = common::rewrite_zip(&src, |name, data| {
        if name == "document.K2F.lock" {
            None
        } else {
            Some(data)
        }
    });
    let err = export_bytes(&stripped).unwrap_err();
    assert!(matches!(err, PptxError::Unlocked), "got {err}");
}

#[test]
fn unknown_op_in_invoice_lock_returns_unknown_op() {
    let src = common::invoice_bytes();
    let lock = common::xml_in(&src, "document.K2F.lock");
    let rp = lock.find("\"render_plan\"").expect("render_plan");
    let needle = "\"ops\":[";
    let rel = lock[rp..].find(needle).expect("ops");
    let at = rp + rel + needle.len();
    let mut mutated = String::with_capacity(lock.len() + 32);
    mutated.push_str(&lock[..at]);
    mutated.push_str("{\"type\":\"draw_unicorn\"},");
    mutated.push_str(&lock[at..]);
    let k2f = common::rewrite_zip(&src, |name, data| {
        if name == "document.K2F.lock" {
            Some(mutated.as_bytes().to_vec())
        } else {
            Some(data)
        }
    });
    let err = export_bytes(&k2f).unwrap_err();
    assert!(matches!(err, PptxError::UnknownOp), "got {err}");
}

#[test]
fn cli_exports_invoice_zip() {
    let out = std::env::temp_dir().join(format!("k2f-pptx-cli-{}.pptx", std::process::id()));
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_k2f-pptx"))
        .args([
            "export",
            common::invoice_path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let bytes = std::fs::read(&out).unwrap();
    let _ = std::fs::remove_file(&out);
    assert!(bytes.starts_with(b"PK"));
    assert_eq!(bytes, export_opened(&common::invoice()).unwrap());
}
