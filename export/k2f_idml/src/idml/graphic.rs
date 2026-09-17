use crate::coord::{DOM, NS};
use crate::xml::escape_xml;
use std::collections::BTreeSet;

const XML_DECL: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;

pub fn graphic_xml(extra_hex: &BTreeSet<String>) -> String {
    let mut group = String::from(
        r#"    <ColorGroupSwatch Self="kCgsNone" SwatchItemRef="Swatch/None"/>
    <ColorGroupSwatch Self="kCgsPaper" SwatchItemRef="Color/Paper"/>
    <ColorGroupSwatch Self="kCgsBlack" SwatchItemRef="Color/Black"/>
"#,
    );
    let mut colors = String::new();
    for hex in extra_hex {
        group.push_str(&format!(
            "    <ColorGroupSwatch Self=\"kCgs{hex}\" SwatchItemRef=\"Color/k2f_{hex}\"/>\n"
        ));
        let value = rgb_value(hex);
        colors.push_str(&format!(
            "  <Color Self=\"Color/k2f_{hex}\" Model=\"Process\" Space=\"RGB\" ColorValue=\"{value}\" Name=\"k2f_{hex}\"/>\n"
        ));
    }
    format!(
        r#"{XML_DECL}
<idPkg:Graphic xmlns:idPkg="{NS}" DOMVersion="{DOM}">
  <ColorGroup Self="ColorGroup/[$ID/]" Name="$ID/" IsRootColorGroup="true">
{group}  </ColorGroup>
  <Swatch Self="Swatch/None" Name="$ID/None" ColorValue=""/>
  <Color Self="Color/Paper" Model="Process" Space="RGB" ColorValue="255 255 255" Name="Paper"/>
  <Color Self="Color/Black" Model="Process" Space="RGB" ColorValue="0 0 0" Name="Black"/>
{colors}</idPkg:Graphic>
"#
    )
}

pub fn fonts_xml(faces: &[(String, String)]) -> String {
    let mut by_family: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for (family, style) in faces {
        let fam = family.trim();
        let sty = style.trim();
        if fam.is_empty() {
            continue;
        }
        let sty = if sty.is_empty() { "Regular" } else { sty };
        let styles = by_family.entry(fam.to_string()).or_default();
        if !styles.iter().any(|s| s == sty) {
            styles.push(sty.to_string());
        }
    }
    let mut inner = String::new();
    for (name, styles) in by_family {
        let esc = escape_xml(&name);
        inner.push_str(&format!(
            "  <FontFamily Self=\"FontFamily/$ID/{esc}\" Name=\"{esc}\">\n"
        ));
        for sty in styles {
            let es = escape_xml(&sty);
            inner.push_str(&format!(
                "    <Font Self=\"Font/$ID/{esc}-{es}\" FontFamily=\"{esc}\" Name=\"{es}\" Status=\"Installed\" FontStyleName=\"{es}\" FontType=\"OpenTypeTT\"/>\n"
            ));
        }
        inner.push_str("  </FontFamily>\n");
    }
    format!(
        r#"{XML_DECL}
<idPkg:Fonts xmlns:idPkg="{NS}" DOMVersion="{DOM}">
{inner}</idPkg:Fonts>
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_xml_registers_book_not_only_regular() {
        let xml = fonts_xml(&[
            ("DejaVu Serif".into(), "Book".into()),
            ("DejaVu Serif".into(), "Book".into()),
            ("Roboto".into(), "Regular".into()),
        ]);
        assert!(
            xml.contains("FontStyleName=\"Book\""),
            "DejaVu Serif Book must be registered, got {xml}"
        );
        assert!(
            xml.contains("Name=\"Book\""),
            "Font/@Name is the subfamily InDesign looks up, got {xml}"
        );
        assert!(
            xml.contains("DejaVu Serif-Book"),
            "got {xml}"
        );
        assert!(xml.contains("FontStyleName=\"Regular\""));
        assert_eq!(
            xml.matches("<FontFamily ").count(),
            2,
            "one family element per family, got {xml}"
        );
    }
}

fn rgb_value(hex: &str) -> String {
    if hex.len() != 6 {
        return "0 0 0".into();
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    format!("{r} {g} {b}")
}
