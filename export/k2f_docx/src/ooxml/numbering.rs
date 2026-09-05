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
    <w:multiLevelType w:val="hybridMultilevel"/>
"#
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
        <w:ind w:left="{left}" w:hanging="360"/>
      </w:pPr>
    </w:lvl>
"#,
                left = 720 + ilvl * 360,
            ));
        } else {
            s.push_str(&format!(
                r#"    <w:lvl w:ilvl="{ilvl}">
      <w:start w:val="1"/>
      <w:numFmt w:val="bullet"/>
      <w:lvlText w:val="•"/>
      <w:lvlJc w:val="left"/>
      <w:pPr>
        <w:ind w:left="{left}" w:hanging="360"/>
      </w:pPr>
    </w:lvl>
"#,
                left = 720 + ilvl * 360,
            ));
        }
    }
    s.push_str("  </w:abstractNum>\n");
    s
}
