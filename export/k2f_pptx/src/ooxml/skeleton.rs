use crate::ir::{DeckIR, SlideElement};
use crate::ooxml::parts;
use crate::ooxml::slide;
use crate::ooxml::xml_master;
use crate::ooxml::xml_theme::THEME_XML;
use std::collections::{BTreeMap, BTreeSet};

pub fn build_package(deck: &DeckIR) -> BTreeMap<String, Vec<u8>> {
    let n = deck.slides.len();
    let slide_w_emu = deck.slides[0].width_emu;
    let slide_h_emu = deck.slides[0].height_emu;
    let mut files = BTreeMap::new();
    let mut media_exts = BTreeSet::new();
    for slide in &deck.slides {
        for el in &slide.elements {
            if let SlideElement::Picture(p) | SlideElement::Raster(p) = el {
                if let Some((_, ext)) = p.media_name.rsplit_once('.') {
                    media_exts.insert(ext.to_ascii_lowercase());
                }
                files.insert(format!("ppt/media/{}", p.media_name), p.bytes.clone());
            }
        }
    }
    files.insert(
        "[Content_Types].xml".into(),
        parts::content_types_xml(n, &media_exts).into_bytes(),
    );
    files.insert("_rels/.rels".into(), parts::root_rels().as_bytes().to_vec());
    files.insert(
        "docProps/core.xml".into(),
        parts::core_xml(&deck.title).into_bytes(),
    );
    files.insert("docProps/app.xml".into(), parts::app_xml(n).into_bytes());
    files.insert(
        "ppt/presentation.xml".into(),
        parts::presentation_xml(n, slide_w_emu, slide_h_emu).into_bytes(),
    );
    files.insert(
        "ppt/_rels/presentation.xml.rels".into(),
        parts::presentation_rels(n).into_bytes(),
    );
    files.insert(
        "ppt/slideMasters/slideMaster1.xml".into(),
        xml_master::slide_master_xml(slide_w_emu, slide_h_emu).into_bytes(),
    );
    files.insert(
        "ppt/slideMasters/_rels/slideMaster1.xml.rels".into(),
        xml_master::slide_master_rels().as_bytes().to_vec(),
    );
    files.insert(
        "ppt/slideLayouts/slideLayout1.xml".into(),
        xml_master::slide_layout_xml(slide_w_emu, slide_h_emu).into_bytes(),
    );
    files.insert(
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels".into(),
        xml_master::slide_layout_rels().as_bytes().to_vec(),
    );
    files.insert("ppt/theme/theme1.xml".into(), THEME_XML.as_bytes().to_vec());
    for (i, slide) in deck.slides.iter().enumerate() {
        let n = i + 1;
        let bindings = slide::bind_slide(slide);
        files.insert(
            format!("ppt/slides/slide{n}.xml"),
            slide::slide_xml(slide, &bindings).into_bytes(),
        );
        files.insert(
            format!("ppt/slides/_rels/slide{n}.xml.rels"),
            slide::slide_rels(&bindings).into_bytes(),
        );
    }
    files
}
