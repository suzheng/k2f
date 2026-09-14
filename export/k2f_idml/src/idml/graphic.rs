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

pub fn fonts_xml(families: &[String]) -> String {
    let mut inner = String::new();
    for name in families {
        let esc = escape_xml(name);
        inner.push_str(&format!(
            "  <FontFamily Self=\"FontFamily/$ID/{esc}\" Name=\"{esc}\">\n    <Font Self=\"Font/$ID/{esc}-Regular\" FontFamily=\"{esc}\" Name=\"Regular\" Status=\"Installed\" FontStyleName=\"Regular\" FontType=\"OpenTypeTT\"/>\n  </FontFamily>\n"
        ));
    }
    format!(
        r#"{XML_DECL}
<idPkg:Fonts xmlns:idPkg="{NS}" DOMVersion="{DOM}">
{inner}</idPkg:Fonts>
"#
    )
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
