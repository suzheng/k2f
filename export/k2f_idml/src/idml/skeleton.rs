use crate::coord::{fmt_pt, SpreadSpace, DOM, NS};
use crate::xml::escape_xml;

const XML_DECL: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;
/// InDesign refuses IDML without this PI (error 29441 / "format not supported").
const AID_PI: &str =
    r#"<?aid style="50" type="document" readerVersion="6.0" featureSet="257" product="16.0(0)" ?>"#;

pub const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="designmap.xml" media-type="application/vnd.adobe.indesign-idml-package"/>
  </rootfiles>
</container>
"#;

pub const STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<idPkg:Styles xmlns:idPkg="http://ns.adobe.com/AdobeInDesign/idml/1.0/packaging" DOMVersion="16.0">
  <RootParagraphStyleGroup Self="n">
    <ParagraphStyle Self="ParagraphStyle/$ID/[No paragraph style]" Name="$ID/[No paragraph style]"/>
  </RootParagraphStyleGroup>
  <RootCharacterStyleGroup Self="n">
    <CharacterStyle Self="CharacterStyle/$ID/[No character style]" Name="$ID/[No character style]"/>
  </RootCharacterStyleGroup>
  <RootTableStyleGroup Self="n">
    <TableStyle Self="TableStyle/$ID/[No table style]" Name="$ID/[No table style]"/>
  </RootTableStyleGroup>
  <RootCellStyleGroup Self="n">
    <CellStyle Self="CellStyle/$ID/[None]" Name="$ID/[None]"/>
  </RootCellStyleGroup>
  <RootObjectStyleGroup Self="n">
    <ObjectStyle Self="ObjectStyle/$ID/[None]" Name="$ID/[None]" AppliedParagraphStyle="ParagraphStyle/$ID/[No paragraph style]"/>
  </RootObjectStyleGroup>
</idPkg:Styles>
"#;

pub const TAGS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<idPkg:Tags xmlns:idPkg="http://ns.adobe.com/AdobeInDesign/idml/1.0/packaging" DOMVersion="16.0">
  <XMLTag Self="XMLTag/Root" Name="Root">
    <Properties>
      <TagColor type="enumeration">LightBlue</TagColor>
    </Properties>
  </XMLTag>
</idPkg:Tags>
"#;

pub const BACKING_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<idPkg:BackingStory xmlns:idPkg="http://ns.adobe.com/AdobeInDesign/idml/1.0/packaging" DOMVersion="16.0">
  <XmlStory Self="kXmlStory" AppliedTOCStyle="n" TrackChanges="false" StoryTitle="$ID/" AppliedNamedGrid="n">
    <StoryPreference OpticalMarginAlignment="false" OpticalMarginSize="12" FrameType="TextFrameType" StoryOrientation="Horizontal" StoryDirection="LeftToRightDirection"/>
  </XmlStory>
</idPkg:BackingStory>
"#;

pub fn designmap_xml(n_pages: usize, stories: &[String]) -> String {
    let mut spreads = String::new();
    for i in 0..n_pages {
        spreads.push_str(&format!(
            "  <idPkg:Spread src=\"Spreads/Spread_k{i}.xml\"/>\n"
        ));
    }
    let mut story_ents = String::new();
    for src in stories {
        story_ents.push_str(&format!("  <idPkg:Story src=\"{src}\"/>\n"));
    }
    format!(
        r#"{XML_DECL}
{AID_PI}
<Document xmlns:idPkg="{NS}" DOMVersion="{DOM}" Self="kDoc">
  <idPkg:Preferences src="Resources/Preferences.xml"/>
  <idPkg:Styles src="Resources/Styles.xml"/>
  <idPkg:Graphic src="Resources/Graphic.xml"/>
  <idPkg:Fonts src="Resources/Fonts.xml"/>
  <idPkg:Tags src="XML/Tags.xml"/>
  <Layer Self="kLayer" Name="Layer 1" Visible="true" Locked="false" IgnoreWrap="false" ShowGuides="true" LockGuides="false" UngroupWhenPrinting="false"/>
  <idPkg:MasterSpread src="MasterSpreads/MasterSpread_kMaster.xml"/>
{spreads}{story_ents}  <idPkg:BackingStory src="XML/BackingStory.xml"/>
</Document>
"#
    )
}

pub fn metadata_xml(title: &str) -> String {
    let title = escape_xml(title);
    format!(
        r#"{XML_DECL}
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about="" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:xmp="http://ns.adobe.com/xap/1.0/">
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">{title}</rdf:li>
        </rdf:Alt>
      </dc:title>
      <xmp:CreateDate>1980-01-01T00:00:00Z</xmp:CreateDate>
      <xmp:ModifyDate>1980-01-01T00:00:00Z</xmp:ModifyDate>
      <xmp:MetadataDate>1980-01-01T00:00:00Z</xmp:MetadataDate>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
"#
    )
}

pub fn preferences_xml(space: &SpreadSpace) -> String {
    let w = fmt_pt(space.page_w);
    let h = fmt_pt(space.page_h);
    let orient = if space.page_w >= space.page_h {
        "Landscape"
    } else {
        "Portrait"
    };
    format!(
        r#"{XML_DECL}
<idPkg:Preferences xmlns:idPkg="{NS}" DOMVersion="{DOM}">
  <DocumentPreference PageWidth="{w}" PageHeight="{h}" PageOrientation="{orient}" FacingPages="false" DocumentBleedTopOffset="0" DocumentBleedBottomOffset="0" DocumentBleedInsideOrLeftOffset="0" DocumentBleedOutsideOrRightOffset="0" DocumentSlugTopOffset="0" DocumentSlugBottomOffset="0" DocumentSlugInsideOrLeftOffset="0" DocumentSlugRightOrOutsideOffset="0" ColumnGuideCount="1" ColumnGuideGutter="12" Intent="PrintIntent" PageBinding="LeftToRight" MasterTextFrame="false"/>
  <ViewPreference HorizontalMeasurementUnits="Points" VerticalMeasurementUnits="Points"/>
</idPkg:Preferences>
"#
    )
}

pub fn master_xml(space: &SpreadSpace, frames: &str) -> String {
    let bounds = space.page_geometric_bounds();
    format!(
        r#"{XML_DECL}
<idPkg:MasterSpread xmlns:idPkg="{NS}" DOMVersion="{DOM}">
  <MasterSpread Self="kMaster" Name="A-Master" NamePrefix="A" BaseName="Master" ItemTransform="1 0 0 1 0 0" OverriddenPageItemProps="" PageCount="1" BindingLocation="0">
    <Page Self="kMasterPage" AppliedMaster="n" GeometricBounds="{bounds}" ItemTransform="1 0 0 1 0 0" Name="A"/>
{frames}  </MasterSpread>
</idPkg:MasterSpread>
"#
    )
}
