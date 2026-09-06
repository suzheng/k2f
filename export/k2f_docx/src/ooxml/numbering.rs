pub(crate) fn numbering_xml() -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
"#,
    );
    xml.push_str(&abstract_num(0, false));
    xml.push_str(&abstract_num(1, true));
    xml.push_str(
        r#"  <w:num w:numId="1">
    <w:abstractNumId w:val="0"/>
  </w:num>
  <w:num w:numId="2">
    <w:abstractNumId w:val="1"/>
  </w:num>
</w:numbering>
"#,
    );
    xml
}

fn abstract_num(id: u32, numbered: bool) -> String {
    let mut s = format!(
        r#"  <w:abstractNum w:abstractNumId="{id}">
    <w:nsid w:val="{nsid}"/>
    <w:multiLevelType w:val="hybridMultilevel"/>
"#,
        nsid = format!("{:08X}", 0x4B32_4630 + id),
    );
    for ilvl in 0..9u32 {
        if numbered {
            let lvl_text = format!("%{}.", ilvl + 1);
            s.push_str(&format!(
                r#"    <w:lvl w:ilvl="{ilvl}">
      <w:start w:val="1"/>
      <w:numFmt w:val="decimal"/>
      <w:lvlText w:val="{lvl_text}"/>
      <w:lvlJc w:val="left"/>
      <w:pPr>
        <w:ind w:left="0" w:hanging="0"/>
      </w:pPr>
      <w:rPr>
        <w:color w:val="000001"/>
      </w:rPr>
    </w:lvl>
"#
            ));
        } else {
            s.push_str(&format!(
                r#"    <w:lvl w:ilvl="{ilvl}">
      <w:start w:val="1"/>
      <w:numFmt w:val="bullet"/>
      <w:lvlText w:val="•"/>
      <w:lvlJc w:val="left"/>
      <w:pPr>
        <w:ind w:left="0" w:hanging="0"/>
      </w:pPr>
      <w:rPr>
        <w:color w:val="000001"/>
      </w:rPr>
    </w:lvl>
"#
            ));
        }
    }
    s.push_str("  </w:abstractNum>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbering_pins_rgb_marker_color() {
        let xml = numbering_xml();
        assert!(
            xml.contains(r#"<w:color w:val="000001"/>"#),
            "list markers must not use Automatic color, got {xml}"
        );
        assert!(
            !xml.contains(r#"w:hanging="360""#),
            "default numbering indent would shrink lock text boxes, got {xml}"
        );
    }
}
