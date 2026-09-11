use crate::ir::{collect_hyperlink_urls, collect_pictures, has_lists, hyperlink_theme_hex, DocIR};
use crate::xml::escape_xml;
use std::collections::BTreeMap;

use super::document::{document_xml, hdrftr_xml};
use super::media::{
    collect_page_pictures, hyperlink_rel_xml, image_rel_xml, insert_media, media_default_xml,
    media_exts, part_rels_xml, picture_rids,
};
use super::numbering::numbering_xml;
use super::xml_theme;

pub fn build_package(ir: &DocIR) -> BTreeMap<String, Vec<u8>> {
    let n = ir.pages.len();
    let has_header = !ir.header.is_empty();
    let has_footer = !ir.footer.is_empty();
    let mut all_els = Vec::new();
    for p in &ir.pages {
        all_els.extend(p.elements.iter().cloned());
    }
    all_els.extend(ir.header.iter().cloned());
    all_els.extend(ir.footer.iter().cloned());
    let has_numbering = has_lists(&all_els);
    let mut urls = collect_hyperlink_urls(&all_els);
    urls.sort();
    let mut hyperlink_rids = BTreeMap::new();
    for (i, url) in urls.iter().enumerate() {
        hyperlink_rids.insert(url.clone(), format!("rIdL{}", i + 1));
    }

    let body_pics = collect_page_pictures(&ir.pages);
    let header_pics = collect_pictures(&ir.header);
    let footer_pics = collect_pictures(&ir.footer);
    let mut all_pics = Vec::new();
    all_pics.extend(body_pics.iter().copied());
    all_pics.extend(header_pics.iter().copied());
    all_pics.extend(footer_pics.iter().copied());
    let exts = media_exts(all_pics.iter().copied());
    let body_pic_rids = picture_rids(&body_pics);
    let header_pic_rids = picture_rids(&header_pics);
    let footer_pic_rids = picture_rids(&footer_pics);

    let mut files = BTreeMap::new();
    insert_media(&mut files, &all_pics);
    files.insert(
        "[Content_Types].xml".into(),
        content_types(has_header, has_footer, has_numbering, &exts).into_bytes(),
    );
    files.insert("_rels/.rels".into(), ROOT_RELS.as_bytes().to_vec());
    files.insert("docProps/core.xml".into(), core_xml(&ir.title).into_bytes());
    files.insert("docProps/app.xml".into(), app_xml(n).into_bytes());
    files.insert(
        "word/document.xml".into(),
        document_xml(ir, &hyperlink_rids, &body_pic_rids, has_header, has_footer).into_bytes(),
    );
    files.insert(
        "word/_rels/document.xml.rels".into(),
        document_rels(
            has_header,
            has_footer,
            has_numbering,
            &hyperlink_rids,
            &body_pic_rids,
        )
        .into_bytes(),
    );
    let hlink_hex = hyperlink_theme_hex(&all_els);
    files.insert(
        "word/styles.xml".into(),
        styles_xml(hlink_hex.as_deref()).into_bytes(),
    );
    files.insert(
        "word/theme/theme1.xml".into(),
        xml_theme::theme_xml(hlink_hex.as_deref()).into_bytes(),
    );
    files.insert("word/settings.xml".into(), SETTINGS.as_bytes().to_vec());
    files.insert("word/fontTable.xml".into(), FONT_TABLE.as_bytes().to_vec());
    files.insert(
        "word/webSettings.xml".into(),
        WEB_SETTINGS.as_bytes().to_vec(),
    );
    if has_numbering {
        files.insert("word/numbering.xml".into(), numbering_xml().into_bytes());
    }
    if has_header {
        files.insert(
            "word/header1.xml".into(),
            hdrftr_xml("hdr", &ir.header, &hyperlink_rids, &header_pic_rids).into_bytes(),
        );
        if let Some(xml) = part_rels_xml(&rids_used(&hyperlink_rids, &ir.header), &header_pic_rids)
        {
            files.insert("word/_rels/header1.xml.rels".into(), xml.into_bytes());
        }
    }
    if has_footer {
        files.insert(
            "word/footer1.xml".into(),
            hdrftr_xml("ftr", &ir.footer, &hyperlink_rids, &footer_pic_rids).into_bytes(),
        );
        if let Some(xml) = part_rels_xml(&rids_used(&hyperlink_rids, &ir.footer), &footer_pic_rids)
        {
            files.insert("word/_rels/footer1.xml.rels".into(), xml.into_bytes());
        }
    }
    files
}

fn rids_used(
    all: &BTreeMap<String, String>,
    els: &[crate::ir::PageElement],
) -> BTreeMap<String, String> {
    collect_hyperlink_urls(els)
        .into_iter()
        .filter_map(|u| all.get(&u).map(|rid| (u, rid.clone())))
        .collect()
}

fn core_xml(title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>{}</dc:title>
  <dc:creator>K2F</dc:creator>
  <cp:lastModifiedBy>K2F</cp:lastModifiedBy>
  <dcterms:created xsi:type="dcterms:W3CDTF">1980-01-01T00:00:00Z</dcterms:created>
  <dcterms:modified xsi:type="dcterms:W3CDTF">1980-01-01T00:00:00Z</dcterms:modified>
</cp:coreProperties>
"#,
        escape_xml(title)
    )
}

fn app_xml(page_count: usize) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
  <Application>K2F</Application>
  <Pages>{page_count}</Pages>
</Properties>
"#
    )
}

fn content_types(
    header: bool,
    footer: bool,
    numbering: bool,
    media_exts: &std::collections::BTreeSet<String>,
) -> String {
    let mut extra = String::new();
    if numbering {
        extra.push_str("  <Override PartName=\"/word/numbering.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/>\n");
    }
    if header {
        extra.push_str("  <Override PartName=\"/word/header1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml\"/>\n");
    }
    if footer {
        extra.push_str("  <Override PartName=\"/word/footer1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml\"/>\n");
    }
    let media = media_default_xml(media_exts);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
{media}  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
  <Override PartName="/word/settings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"/>
  <Override PartName="/word/webSettings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml"/>
  <Override PartName="/word/fontTable.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
{extra}</Types>
"#
    )
}

fn document_rels(
    header: bool,
    footer: bool,
    numbering: bool,
    hyperlink_rids: &BTreeMap<String, String>,
    picture_rids: &BTreeMap<String, String>,
) -> String {
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings" Target="webSettings.xml"/>
  <Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="fontTable.xml"/>
  <Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>
"#,
    );
    if numbering {
        rels.push_str("  <Relationship Id=\"rIdN1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering\" Target=\"numbering.xml\"/>\n");
    }
    if header {
        rels.push_str("  <Relationship Id=\"rIdH1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/header\" Target=\"header1.xml\"/>\n");
    }
    if footer {
        rels.push_str("  <Relationship Id=\"rIdF1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer\" Target=\"footer1.xml\"/>\n");
    }
    rels.push_str(&image_rel_xml(picture_rids));
    rels.push_str(&hyperlink_rel_xml(hyperlink_rids));
    rels.push_str("</Relationships>\n");
    rels
}

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>
"#;

fn styles_xml(hlink_hex: Option<&str>) -> String {
    // Hosts apply the Hyperlink character style to `w:hyperlink` even when the
    // run sets `w:color`. Pin that style to the lock color (or near-black).
    let color = hlink_hex.unwrap_or("000001");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:docDefaults>
    <w:rPrDefault>
      <w:rPr>
        <w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Calibri"/>
        <w:color w:val="000001"/>
        <w:sz w:val="22"/>
        <w:szCs w:val="22"/>
      </w:rPr>
    </w:rPrDefault>
    <w:pPrDefault>
      <w:pPr>
        <w:spacing w:after="0" w:line="240" w:lineRule="auto"/>
      </w:pPr>
    </w:pPrDefault>
  </w:docDefaults>
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/>
    <w:qFormat/>
  </w:style>
  <w:style w:type="character" w:styleId="Hyperlink">
    <w:name w:val="Hyperlink"/>
    <w:rPr>
      <w:color w:val="{color}"/>
      <w:u w:val="single" w:color="{color}"/>
    </w:rPr>
  </w:style>
  <w:style w:type="table" w:default="1" w:styleId="TableNormal">
    <w:name w:val="Normal Table"/>
    <w:uiPriority w:val="99"/>
    <w:semiHidden/>
    <w:unhideWhenUsed/>
    <w:tblPr>
      <w:tblInd w:w="0" w:type="dxa"/>
      <w:tblCellMar>
        <w:top w:w="0" w:type="dxa"/>
        <w:left w:w="0" w:type="dxa"/>
        <w:bottom w:w="0" w:type="dxa"/>
        <w:right w:w="0" w:type="dxa"/>
      </w:tblCellMar>
    </w:tblPr>
  </w:style>
</w:styles>
"#
    )
}

// w:background + displayBackgroundShape is the Office page-color slot.
// Word Dark Mode may hide it; the lock's full-page solid is the wash.
const SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:displayBackgroundShape/>
  <w:compat>
    <w:compatSetting w:name="compatibilityMode" w:uri="http://schemas.microsoft.com/office/word" w:val="15"/>
  </w:compat>
</w:settings>
"#;

const FONT_TABLE: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:font w:name="Calibri">
    <w:charset w:val="00"/>
    <w:family w:val="swiss"/>
    <w:pitch w:val="variable"/>
  </w:font>
</w:fonts>
"#;

const WEB_SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:webSettings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:optimizeForBrowser/>
</w:webSettings>
"#;
